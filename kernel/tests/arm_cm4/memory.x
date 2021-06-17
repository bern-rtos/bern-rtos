MEMORY {
    FLASH : ORIGIN = 0x08000000, LENGTH = 512K
    RAM : ORIGIN = 0x20000000, LENGTH = 127K
    SHARED : ORIGIN = 0x20000000 + 127K, LENGTH = 1K
}

/* Align stacks to double word see:
   https://community.arm.com/developer/ip-products/processors/f/cortex-m-forum/6344/what-is-the-meaning-of-a-64-bit-aligned-stack-pointer-address */
SECTIONS {
    .task_stack (NOLOAD) : ALIGN(8)
    {
        . = ALIGN(8);
        __stask_stack = .;
        *(.task_stack);
        . = ALIGN(8);
        __etask_stack = .;
    } > RAM
    __sitask_stack = LOADADDR(.task_stack);
} INSERT AFTER .bss;

SECTIONS {
    /*### .shared */
    .shared : ALIGN(4)
    {
        __sishared = LOADADDR(.shared);
        . = ALIGN(4);
        __sshared = .;
        *(.shared);
        . = ALIGN(4);
        __eshared = .;
    } > SHARED
}
