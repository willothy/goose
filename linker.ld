/*
The loader will find this when parsing the elf image after reading its
location from multiboot.
*/
ENTRY(_start)

/*
Tell where the various sections of the object files will be put in the final
kernel image.

NOTE: I don't know if I need this stuff for the kernel now that the loader is separate...
*/
SECTIONS
{
	/*
	Begin putting sections at 1 MiB, a conventional place for kernels to be
	loaded at by the bootloader.
	*/
	. = 1M;

	.boot :
	{
		KEEP(*(.multiboot))
	}

	/* Read-only code. */
	.text :
	{
		*(.text)
	}

	/* Read-only data. */
	.rodata ALIGN(4K) : ALIGN(4K)
	{
		*(.rodata)
	}

	/* Read-write data (initialized) */
	.data ALIGN(4K) : ALIGN(4K)
	{
		*(.data)
	}

	/* Read-write data (uninitialized) and stack */
	.bss ALIGN(4K) : ALIGN(4K)
	{
		*(.bss)
		*(COMMON)
	}

	/* The compiler may produce other sections, by default it will put them in
	   a segment with the same name. Simply add stuff here as needed. */
}
