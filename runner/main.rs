use std::process::Command;

#[test]
fn test_main() {
    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.args(&[
        "-cdrom",
        "bruh_os.iso",
        "-device",
        "isa-debug-exit,iobase=0xf4,iosize=0x04",
    ]);

    if !cmd.status().expect("failed to execute qemu").success() {
        panic!("qemu failed");
    }
}

fn main() {
    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.args(&["-cdrom", "bruh_os.iso"]);

    if !cmd.status().expect("failed to execute qemu").success() {
        panic!("qemu failed");
    }
}
