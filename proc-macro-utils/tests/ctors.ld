PHDRS
{
    headers   PT_PHDR PHDRS;
    text      PT_LOAD FILEHDR PHDRS;
}

SECTIONS
{
    . = SIZEOF_HEADERS;

    .text : {
        __ctors_init_array_start = .;
        KEEP(*(.ctors_init_array))
        KEEP(*(.ctors_init_array.*))
        __ctors_init_array_end = .;

        *(.text .text.*)
    } :text
}