# bruh OS

Hobby OS project. Probably very bad. I have no idea what I'm doing.

Inspired by [OSDev](https://wiki.osdev.org/Main_Page), [blog_os](https://os.phil-opp.com/),
and [StreamOS](https://github.com/sphaerophoria/stream-os).

## Goals

- 64-bit, x86_64
- Bootable by grub or similar (BIOS, UEFI is a non-goal for now)
  - Multiboot1 for now, probably 2 later
- Run on multiple CPU cores (SMP)
- Support for simple graphics such as text and shapes
- Preemptive, multicore scheduler (main goal, I want to learn about this)
- Userland processes
- Basic network stack (maybe a web server if I am lucky)

## About

I've followed the blog_os series before as well as the OSDev barebones tutorial, but I
am attempting to learn by doing with this project. I will be sometimes indirectly
following tutorials or courses I find, but for the most part
my goal here is to really figure out how OS dev works and how to do it myself.

I am writing it in Rust because I like Rust, but also because most of the available resources use
C so I will not have the temptation to copy-paste. Also, there will be crates available for some
tasks which is a nice improvement from c-land. I am already making use of the
[multiboot](https://github.com/gz/rust-multiboot) and [elf](https://github.com/cole14/rust-elf/) crates,
which have simplified the bootloader integration process massively.

While I will use crates for some things, I will be implementing as much as possible
of the kernel myself. I really just didn't want to deal with multiboot or writing an elf parser.

## Architecture

Multiboot1 compatible (maybe multiboot2 in the future).

Stage 1 (/loader, target: i686-unknown-none / i686-bruh_os.json):

- This is where grub puts us initially
- setup stack and do protected mode (32-bit) init stuff
- parse multiboot info (using multiboot crate)
- retrieve kernel module
- parse elf module
- get kernel entry point from elf module

TODO:

- setup basic paging so we can enter long mode
- setup basic gdt so CPU lets us into long mode
- setup long mode (64-bit)
- jump to kernel entry point in long mode

Stage 2: (/src, target: x86_64-unknown-none / x86_64-bruh_os.json):

- This currently does nothing because we cannot yet enter long mode.

TODO:

- Setup long mode 4-level paging
- Setup / reload GDT
- Handle interrupts
- Memory allocator so I can use the `alloc` crate and the heap
