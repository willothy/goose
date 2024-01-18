ARCH := "x86_64"
#ARCH := "i686"
QEMU_ARCH := if ARCH == "x86_64" { "x86_64" } else { "i386" }
TARGET := ARCH + "-bruh_os"

[private]
list:
  @just --list

distclean:
  rm -rf isodir
  rm -f bruh_os.iso

clean: distclean
  cargo clean

build: distclean
  cargo build --package loader --target i686-bruh_os.json
  cargo build --package bruh_os --target x86_64-bruh_os.json
  cp target/x86_64-bruh_os/debug/bruh_os isodir/boot/kernel.bin
  grub-mkrescue -o bruh_os.iso isodir

run: build
  qemu-system-x86_64 -cdrom bruh_os.iso

dump-header:
  rust-objdump -f target/{{TARGET}}/debug/bruh_os

dump-sections:
  rust-objdump -f target/{{TARGET}}/debug/bruh_os

dump-asm:
  rust-objdump -d target/{{TARGET}}/debug/bruh_os

strip-symbols:
  rust-strip target/{{TARGET}}/debug/bruh_os

nm:
  rust-nm target/{{TARGET}}/debug/bruh_os
