MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* TODO Adjust these memory regions to match your device memory layout */
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  RAM : ORIGIN = 0x20000000, LENGTH = 64K
  CCRAM : ORIGIN = 0x10000000, LENGTH = 64K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* You may want to use this variable to locate the call stack and static
   variables in different memory regions. Below is shown the default value */
/* _stack_start = ORIGIN(RAM) + LENGTH(RAM); */

/* You can use this symbol to customize the location of the .text section */
/* If omitted the .text section will be placed right after the .vector_table
   section */
/* This is required only on microcontrollers that store some configuration right
   after the vector table */
/* _stext = ORIGIN(FLASH) + 0x400; */



/* Align stacks to double word see:
   https://community.arm.com/developer/ip-products/processors/f/cortex-m-forum/6344/what-is-the-meaning-of-a-64-bit-aligned-stack-pointer-address */
/* Note that the section will not be zero-initialized by the runtime! */
    SECTIONS {
        .task_stack (NOLOAD) : ALIGN(8) {
            *(.task_stack);
            . = ALIGN(8);
        } > RAM
    } INSERT AFTER .bss;

    SECTIONS {
        /*### .shared */
        .shared : ALIGN(4)
        {
            . = ALIGN(4);
            __sshared = .;
            *(.shared);
            . = ALIGN(4);
            __eshared = .;
        } > RAM
        __sishared = LOADADDR(.shared);
    } INSERT AFTER .task_stack
