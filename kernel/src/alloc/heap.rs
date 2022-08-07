//! Heap implementation based on a free linked list. The list is sorted by
//! memory address, so that adjecent free blocks can be found and merged easily.

use core::alloc::Layout;
use core::cell::Cell;
use core::mem::size_of;
use core::ptr;
use core::ptr::{NonNull, slice_from_raw_parts_mut};
use bern_units::memory_size::Byte;
use crate::alloc::allocator::{Allocator, AllocError};
use crate::mem::linked_list::{LinkedList, Node};
use crate::mem::boxed::Box;

struct Free {
    size: usize,
}

const MIN_FREE_SIZE: usize = size_of::<Node<Free>>();

pub struct Heap {
    free: LinkedList<Free>,
    size: Cell<usize>,
    end: Cell<*const u8>,
}

impl Heap {
    pub const fn empty() -> Self {
        Heap {
            free: LinkedList::new(),
            size: Cell::new(0),
            end: Cell::new(ptr::null()),
        }
    }

    pub fn init_from_slice(&self, memory: &'static mut [u8]) {
        // Note(unsafe): Size is checked to be large enough.
        unsafe {
            let size = memory.len();
            assert!(size >= MIN_FREE_SIZE);

            self.insert_free(NonNull::new_unchecked(memory as *mut _));
            self.size.replace(size);
            self.end.replace(memory.as_ptr());
        }
    }

    ///
    /// # Safety
    /// Free block must larger or equal than `Node<Free>`.
    unsafe fn insert_free(&self, block: NonNull<[u8]>) {
        let size = block.as_ref().len();

        let ptr = block.as_ptr() as *mut Node<Free>;

        let node = Node::new(Free {
            size
        });

        ptr.write(node);

        let boxed = Box::from_raw(NonNull::new_unchecked(ptr));
        let mut cursor = self.free.cursor_front_mut();
        // Insert the new free block by ascending memory address.
        for _i in 0..self.free.len() + 1 {
            if (cursor.node() as usize) > (ptr as usize) {
                // Is this the adjecent node?
                if (cursor.node() as usize) == (ptr as usize + size) {
                    // Join right
                    (*ptr).size += (*cursor.node()).size;
                    Box::leak(cursor.take().unwrap_unchecked());
                }

                if cursor.node().is_null() {
                    // We could have moved the cursor to the end of the list
                    // when when took a node.
                    self.free.push_back(boxed);
                } else {
                    self.free.insert(
                        NonNull::new_unchecked(cursor.node()),
                        boxed,
                    );
                }
                return;
            } else if cursor.node().is_null() {
                // End of list reached.
                self.free.push_back(boxed);
                return;
            }
            cursor.move_next();
        }
        panic!("`self.free_list` contains a loop.");
    }

    #[allow(unused)]
    fn align(ptr: *mut u8, align: usize) -> *mut u8 {
        unsafe {
            ptr.add(ptr.align_offset(align))
        }
    }

    ///
    /// # Safety
    /// - Pointer must not be `null`
    /// - Range must be checked
    unsafe fn slice_ptr_from_raw(ptr: *mut u8, len: usize) -> NonNull<[u8]> {
        NonNull::new_unchecked(slice_from_raw_parts_mut(
            ptr,
            len,
        ))
    }
}

impl Allocator for Heap {
    fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut size_requested = layout.size();
        if size_requested == 0 {
            return Err(AllocError::Other);
        }
        // Fulfill min size requirement to place a free node upon deallocation.
        if size_requested < MIN_FREE_SIZE {
            size_requested = MIN_FREE_SIZE;
        }

        // Find free block that is large enough.
        let mut cursor = self.free.cursor_front_mut();
        for i in 0..self.free.len() + 1{
            match cursor.inner() {
                None => return Err(AllocError::OutOfMemory),
                Some(node) => {
                    if node.size == size_requested {
                        break; // perfect fit
                    } else if node.size >= size_requested + MIN_FREE_SIZE {
                        break; // Free node must fit into remaing memory.
                    }
                }
            }
            cursor.move_next();
            if i == self.free.len() - 1 {
                panic!("`self.free_list` contains a loop.");
            }
        }
        let node = match cursor.take() {
            None => return Err(AllocError::Other),
            Some(n) => n,
        };

        let free_size = node.size;
        let free_ptr = Box::leak(node).as_ptr() as *mut u8;

        // Return remaining memory to the list.
        if free_size > size_requested {
            unsafe {
                // Create new free block.
                self.insert_free(Heap::slice_ptr_from_raw(
                    free_ptr.add(size_requested),
                    free_size - size_requested,
                ));
            }
        }

        unsafe {
            // We return the requested size, if there was padding due to min
            // size requirements we will recalculate it at deallocation.
            Ok(Heap::slice_ptr_from_raw(free_ptr, layout.size()))
        }
    }

    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        let mut size = layout.size();
        // The allocation process made sure that there is enough space to place
        // a free node.
        if size < MIN_FREE_SIZE {
            size = MIN_FREE_SIZE;
        }

        self.insert_free(Heap::slice_ptr_from_raw(
            ptr.as_ptr(),
            size,
        ));
    }

    fn capacity(&self) -> Byte {
        unimplemented!()
    }

    fn usage(&self) -> Byte {
        unimplemented!()
    }
}

unsafe impl Sync for Heap {}


#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn only_allocations() {
        static mut HEAP_BUFFER: [u8; 1000] = [0; 1000];
        static HEAP: Heap = Heap::empty();

        unsafe {
            HEAP.init_from_slice(&mut HEAP_BUFFER);
        }

        for _i in 0..10 {
            let _a = HEAP.alloc(Layout::from_size_align(100, 4).unwrap()).unwrap();
        }
    }

    #[test]
    #[should_panic]
    fn over_allocation() {
        static mut HEAP_BUFFER: [u8; 1024] = [0; 1024];
        static HEAP: Heap = Heap::empty();

        unsafe {
            HEAP.init_from_slice(&mut HEAP_BUFFER);
        }

        for _i in 0..11 {
            let _a = HEAP.alloc(Layout::from_size_align(100, 4).unwrap()).unwrap();
        }
    }

    #[test]
    fn alloc_and_dealloc() {
        static mut HEAP_BUFFER: [u8; 1050] = [0; 1050];
        static HEAP: Heap = Heap::empty();

        unsafe {
            HEAP.init_from_slice(&mut HEAP_BUFFER);
        }

        let layout = Layout::from_size_align(100, 4).unwrap();
        for _i in 0..10 {
            let mut vars: [Option<NonNull<[u8]>>; 10] = [None; 10];
            for e in vars.iter_mut() {
                *e = Some(HEAP.alloc(layout.clone()).unwrap());
            }
            for e in vars.iter_mut() {
                unsafe {
                    HEAP.dealloc(
                        NonNull::new_unchecked(e.take().unwrap().as_ptr() as *mut u8),
                        layout.clone()
                    );
                }
            }
        }

        //assert_eq!(HEAP.free_list.len(), 1); // defragmentation
    }
}