ARCH := "i686"
QEMU_ARCH := "i386"
TARGET := ARCH + "-bruh_os"

[private]
list:
  @just --list

clean: distclean
  cargo clean

distclean:
  rm -rf isodir
  rm -f bruh_os.iso

build: distclean
  cargo build --target {{TARGET}}.json
  mkdir -p isodir/boot/grub
  cp target/{{TARGET}}/debug/bruh_os isodir/boot/bruh_os.bin
  cp grub.cfg isodir/boot/grub/grub.cfg
  grub-mkrescue -o bruh_os.iso isodir

run: build
  qemu-system-{{QEMU_ARCH}} bruh_os.iso

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
