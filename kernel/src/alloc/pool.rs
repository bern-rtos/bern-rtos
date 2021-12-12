use core::alloc::Layout;
use core::cell::Cell;
use core::ptr;
use core::ptr::{NonNull, slice_from_raw_parts_mut};
use crate::alloc::allocator::{Allocator, AllocError};
use crate::mem::boxed::Box;
use crate::mem::queue::mpmc_linked::{Node, Queue};


pub enum PoolError {
    OutOfRange
}

struct Free { }
impl Free {
    const fn new() -> Self {
        Free {}
    }
}

struct Range {
    pub start: *const u8,
    pub end: *const u8,
}

pub struct Partition {
    free: Queue<Free>,
    block_size: Cell<usize>,
    capacity: Cell<usize>,
    range: Cell<Range>,
}

impl Partition {
    pub const fn empty(block_size: usize) -> Self {
        Partition {
            free: Queue::new(),
            block_size: Cell::new(block_size),
            capacity: Cell::new(0),
            range: Cell::new(Range {
                start: ptr::null(),
                end: ptr::null(),
            })
        }
    }

    pub fn init_from_slice(&self, memory: &'static mut [u8]) {
        let len = memory.len();
        let ptr = memory.as_mut_ptr();

        if memory.len() < self.block_size.get() {
            return;
        }

        // todo: align
        let capacity = len / self.block_size.get();

        for i in 0..capacity {
            // Note(unsafe): We should stay within the memory boundries.
            unsafe {
                self.push_free_block(ptr.add(i * self.block_size.get()));
            }
        }

        self.capacity.replace(capacity);
        // Note(unsafe): We just store the start and end address of the static
        // slice.
        unsafe {
            self.range.replace(Range {
                start: ptr,
                end: ptr.add(len)
            });
        }
    }

    fn try_allocate(&self, layout: Layout) -> Option<NonNull<[u8]>> {
        if layout.size() > self.block_size.get() {
            return None;
        }

        self.free.try_pop_front().map(| b | {
            unsafe {
                NonNull::new_unchecked(
                    slice_from_raw_parts_mut(
                        Box::leak(b).as_ptr() as *mut u8,
                        layout.size()
                    )
                )
            }
        })
    }

    unsafe fn try_deallocate(&self, ptr: NonNull<u8>, _layout: Layout) -> Result<(),PoolError> {
        let ptr_raw = ptr.as_ptr();
        let range = self.range.as_ptr();

        if (ptr_raw as usize) < ((*range).start as usize) ||
            (ptr_raw as usize) > ((*range).end as usize) {
            return Err(PoolError::OutOfRange);
        }

        self.push_free_block(ptr_raw);
        Ok(())
    }

    pub fn capacity(&self) -> usize {
        self.capacity.get()
    }

    pub fn free(&self) -> usize {
        self.free.len()
    }

    ///
    /// # Safety
    /// `ptr` must be valid.
    unsafe fn push_free_block(&self, ptr: *mut u8) {
        let free_block = ptr as *mut Node<Free>;
        let node = Node::new(Free::new());
        free_block.write(node);
        self.free.push_front(Box::from_raw(NonNull::new_unchecked(free_block)));
    }
}

unsafe impl Sync for Partition { }


pub struct Pool<const N: usize> {
    partitions: [Partition; N],
}

impl<const N: usize> Pool<{ N }> {
    pub fn partition(&self, index: usize) -> Option<&Partition> {
        if index >= self.partitions.len() {
            None
        } else {
            Some(&self.partitions[index])
        }
    }
}

impl<const N: usize> Allocator for Pool<{ N }> {
    fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        for partition in self.partitions.iter() {
            if let Some(block) = partition.try_allocate(layout) {
                return Ok(block);
            }
        }
        Err(AllocError::OutOfMemory)
    }

    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        for partition in self.partitions.iter() {
            match partition.try_deallocate(ptr, layout) {
                Ok(_) => return,
                Err(_) => continue,
            }
        }
    }

    fn capacity(&self) -> usize {
        todo!()
    }
}

unsafe impl<const N: usize> Sync for Pool<{ N }> { }

#[allow(unused)]
macro_rules! new {
    ($partitions:tt) => {
        Pool {
            partitions: $partitions
        }
    };
}


#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn one_partition() {
        static mut BUFFER: [u8; 1280] = [0; 1280];
        static POOL: Pool<1> = new!([
                Partition::empty(128),
        ]);

        let partion = POOL.partition(0).unwrap();
        unsafe { partion.init_from_slice(BUFFER.as_mut()); }

        assert_eq!(partion.capacity(), 10);
        assert_eq!(partion.free(), 10);
    }

    #[test]
    fn alloc_and_dealloc() {
        static mut BUFFER: [u8; 1280] = [0; 1280];
        static POOL: Pool<1> = new!([
                Partition::empty(128),
        ]);
        unsafe {
            POOL.partition(0).unwrap().init_from_slice(BUFFER.as_mut());
        }

        let layout = Layout::from_size_align(100, 4).unwrap();

        let mut vars: [Option<NonNull<[u8]>>; 10] = [None; 10];
        for var in vars.iter_mut() {
            *var = Some(POOL.alloc(layout.clone()).unwrap());
        }
        assert_eq!(POOL.partition(0).unwrap().free(), 0);

        for var in vars.iter_mut() {
            unsafe {
                POOL.dealloc(
                    NonNull::new_unchecked(var.take().unwrap().as_ptr() as *mut u8),
                    layout.clone()
                );
            }
        }
        assert_eq!(POOL.partition(0).unwrap().free(), 10);
    }

    #[test]
    fn multiple_partitions() {
        static mut BUFFER_1: [u8; 1024] = [0; 1024];
        static mut BUFFER_2: [u8; 1024] = [0; 1024];
        static mut BUFFER_3: [u8; 1024] = [0; 1024];

        static POOL: Pool<3> = new!([
            Partition::empty(128),
            Partition::empty(256),
            Partition::empty(512),
        ]);
        unsafe {
            POOL.partition(0).unwrap().init_from_slice(BUFFER_1.as_mut());
            POOL.partition(1).unwrap().init_from_slice(BUFFER_2.as_mut());
            POOL.partition(2).unwrap().init_from_slice(BUFFER_3.as_mut());
        }

        let layout_a = Layout::from_size_align(100, 4).unwrap();
        let layout_b = Layout::from_size_align(200, 4).unwrap();
        let layout_c = Layout::from_size_align(300, 4).unwrap();

        let a = POOL.alloc(layout_a.clone());
        assert_eq!(POOL.partition(0).unwrap().free(), 7);
        let b = POOL.alloc(layout_b.clone());
        assert_eq!(POOL.partition(1).unwrap().free(), 3);
        let c = POOL.alloc(layout_c.clone());
        assert_eq!(POOL.partition(2).unwrap().free(), 1);

        unsafe {
            POOL.dealloc(NonNull::new_unchecked(a.unwrap().as_ptr() as *mut u8), layout_a.clone());
            POOL.dealloc(NonNull::new_unchecked(b.unwrap().as_ptr() as *mut u8), layout_b.clone());
            POOL.dealloc(NonNull::new_unchecked(c.unwrap().as_ptr() as *mut u8), layout_c.clone());
        }
        assert_eq!(POOL.partition(0).unwrap().free(), 8);
        assert_eq!(POOL.partition(1).unwrap().free(), 4);
        assert_eq!(POOL.partition(2).unwrap().free(), 2);
    }
}