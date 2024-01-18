use std::env::set_current_dir;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-arg=-Tlinker.ld");

    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("to find the manifest dir");
    let root = PathBuf::from(manifest);

    let iso_dir = root.join("isodir");

    if iso_dir.exists() {
        if !iso_dir.is_dir() {
            panic!("iso dir {:?} is not a dir", iso_dir);
        }
        fs::remove_dir_all(&iso_dir).expect("to remove old iso dir");
    }

    let boot_dir = iso_dir.as_path().join("boot");
    std::fs::create_dir_all(&boot_dir).expect("to create boot dir");

    let loader_target = boot_dir.join("loader.bin");

    let loader_root = root.join("loader");
    set_current_dir(loader_root).expect("to set current dir");

    let loader_bin = root.join("target/i686-bruh_os/debug/loader");
    if !loader_bin.exists() {
        panic!("loader.bin not found");
    }
    std::fs::copy(&loader_bin, loader_target).expect("to copy loader.bin");
    // std::fs::remove_file(&loader_bin).expect("to remove loader.bin");

    let grub_dir = boot_dir.as_path().join("grub");
    std::fs::create_dir_all(&grub_dir).expect("to create grub dir");

    let grub_cfg = root.join("grub.cfg");
    let grub_cfg_target = grub_dir.join("grub.cfg");

    if !grub_cfg.exists() {
        panic!("grub.cfg not found");
    }
    std::fs::copy(grub_cfg, grub_cfg_target).expect("to copy grub.cfg");
}
