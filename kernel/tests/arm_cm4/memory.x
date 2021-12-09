MEMORY {
    FLASH : ORIGIN = 0x08000000, LENGTH = 512K
    RAM : ORIGIN = 0x20000000, LENGTH = 127K
}

SECTIONS {
    /* By default static variabled will be placed in .data or .bss. These
     * sections are not accessible by default and do not meet memory protection
     * alignment requirements. Thus, we place a marker to give .data + .bss a
     * fixed size.
     */
    .shared_global ORIGIN(RAM) + 4K : {
        __eshared_global = .;
    } > RAM
} INSERT AFTER .uninit;

SECTIONS {
    _kernel_size = 2K;

    .kernel : ALIGN(4) {
        /* Kernel static memory */
        . = ALIGN(4);
        __smkernel = .;
        *(.kernel);
        *(.kernel.process);
        . = ALIGN(4);
        __emkernel = .;

        /* Kernel heap */
        . = ALIGN(4);
        __shkernel = .;
        . = __smkernel + _kernel_size;
        __ehkernel = .;

        ASSERT(__emkernel <= __ehkernel, "Error: No room left in bern kernel.");
    } > RAM
    __sikernel = LOADADDR(.kernel);
} INSERT AFTER .shared_global;