use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("to find the manifest dir");
    let root = PathBuf::from(manifest);

    let final_iso = root.join("bruh_os.iso");
    if final_iso.exists() {
        fs::remove_file(&final_iso).expect("to remove old iso");
    }

    println!("cargo:rerun-if-changed={}", root.join("grub.cfg").display());
    println!(
        "cargo:rerun-if-changed={}",
        root.join("linker.ld").display()
    );

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let iso_dir = out_dir.join("isodir");

    if iso_dir.exists() {
        if !iso_dir.is_dir() {
            panic!("iso dir {:?} is not a dir", iso_dir);
        }
        fs::remove_dir_all(&iso_dir).expect("to remove old iso dir");
    }

    let boot_dir = iso_dir.as_path().join("boot");
    fs::create_dir_all(&boot_dir).expect("to create boot dir");

    let grub_dir = boot_dir.as_path().join("grub");
    fs::create_dir_all(&grub_dir).expect("to create grub dir");

    let grub_cfg = root.join("grub.cfg");
    let grub_cfg_target = grub_dir.join("grub.cfg");

    if !grub_cfg.exists() {
        panic!("grub.cfg not found");
    }
    fs::copy(grub_cfg, grub_cfg_target).expect("to copy grub.cfg");

    // copy the kernel binary
    let kernel_lib = env::var("CARGO_STATICLIB_FILE_KERNEL_kernel").expect("to find kernel bin");

    let boot_asm = root.join("boot.asm");
    let boot_obj = boot_dir.join("boot.o");
    let mut cmd = Command::new("nasm");
    cmd.arg("-f").arg("elf64");
    cmd.arg("-o").arg(&boot_obj);
    cmd.arg(boot_asm);
    let output = cmd.output().expect("to run nasm");
    if !output.status.success() {
        panic!("nasm failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let boot_64_asm = root.join("boot64.asm");
    let boot_64_obj = boot_dir.join("boot64.o");
    let mut cmd = Command::new("nasm");
    cmd.arg("-f").arg("elf64");
    cmd.arg("-o").arg(&boot_64_obj);
    cmd.arg(boot_64_asm);
    let output = cmd.output().expect("to run nasm");
    if !output.status.success() {
        panic!("nasm failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let linker_script = root.join("linker.ld");
    let mut cmd = Command::new("ld");
    cmd.arg("-n");
    cmd.arg("-T").arg(linker_script);
    cmd.arg("-o").arg(boot_dir.join("kernel.bin"));
    cmd.arg(boot_obj);
    cmd.arg(boot_64_obj);
    cmd.arg(kernel_lib);

    let output = cmd.output().expect("to run ld");
    if !output.status.success() {
        panic!("ld failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let out = Command::new("grub-mkrescue")
        .arg("-o")
        .arg(final_iso)
        .arg(iso_dir)
        .status()
        .expect("to run grub-mkrescue");
    if !out.success() {
        panic!("grub-mkrescue failed");
    }
}
