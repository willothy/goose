use std::{env, fs, path::PathBuf, process::Command};

use walkdir::WalkDir;

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

    // let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    //
    // let loader_crate = root.join("loader");
    //
    // if !loader_crate.exists() {
    //     panic!("loader crate not found");
    // }
    // if !loader_crate.is_dir() {
    //     panic!("loader crate is not a dir");
    // }
    // for entry in WalkDir::new(loader_crate.join("src")) {
    //     if let Ok(entry) = entry {
    //         if entry.file_type().is_file() {
    //             println!("cargo:rerun-if-changed={}", entry.path().display());
    //         }
    //     }
    // }

    // cmd.arg("install")
    //     .arg("loader")
    //     .arg("--path")
    //     .arg(loader_crate)
    //     .arg("--root")
    //     .arg(&out_dir);
    // .arg("--target-dir")
    // .arg(&out_dir);

    // cmd.arg("--target").arg("i686-bruh_os.json");
    // cmd.arg("-Zbuild-std=core,compiler_builtins");
    // cmd.arg("-Zbuild-std-features=compiler-builtins-mem");
    //
    // cmd.env_remove("RUSTFLAGS");
    // cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
    // cmd.env_remove("RUSTC_WORKSPACE_WRAPPER");

    // let output = cmd.output().expect("to run cargo nasm boot.asm");
    // if !output.status.success() {
    //     panic!(
    //         "cargo install loader failed: {}",
    //         String::from_utf8_lossy(&output.stderr)
    //     );
    // }
    // let loader_lib = out_dir
    //     // .join("i686-bruh_os")
    //     // .join("release")
    //     // .join("libloader.a");
    //     // .join("bin")
    //     .join("loader");

    // let loader_bin = convert_elf_to_bin(loader_lib);

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
    } else {
        // println!("cargo:rustc-link-lib=static=loader");
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

fn convert_elf_to_bin(elf_path: PathBuf) -> PathBuf {
    let flat_binary_path = elf_path.with_extension("bin");

    let llvm_tools = llvm_tools::LlvmTools::new().expect("failed to get llvm tools");
    let objcopy = llvm_tools
        .tool(&llvm_tools::exe("llvm-objcopy"))
        .expect("LlvmObjcopyNotFound");

    // convert first stage to binary
    let mut cmd = Command::new(objcopy);
    cmd.arg("-I").arg("elf64-x86-64");
    cmd.arg("-O").arg("binary");
    cmd.arg("--binary-architecture=i386:x86-64");
    cmd.arg(&elf_path);
    cmd.arg(&flat_binary_path);
    let output = cmd
        .output()
        .expect("failed to execute llvm-objcopy command");
    if !output.status.success() {
        panic!(
            "objcopy failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    flat_binary_path
}
