ENTRY(start)
OUTPUT_FORMAT("binary")

SECTIONS
{
    /* Tell the linker that our code layout begins at physical address 0x7E00 */
    . = 0x7E00;

    /* Combine all execution code (.text sections) */
    .text :
    {
        /* Forces stage2.o's 'start' label to be at the very beginning of 0x7E00 */
        *(.text)
    }

    /* Combine all read-only data, like your string messages */
    .rodata : 
    { 
        *(.rodata*) 
    }

    /* Combine initialized variables and global data */
    .data : 
    { 
        *(.data*) 
    }

    /* Combine uninitialized global data (like stack allocations) */
    .bss : 
    { 
        *(.bss*) 
    }
}