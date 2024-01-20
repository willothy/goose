use std::process::Command;

fn main() {
    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.args(&["-cdrom", "bruh_os.iso"]);
    if !cmd.status().expect("failed to execute qemu").success() {
        panic!("qemu failed");
    }
}
