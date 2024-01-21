use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const ASM: &[&str] = &[
    // add asm files here
    "boot.asm",
    "boot64.asm",
    "kernel/src/mem.asm",
];

fn build_assembly_files(files: &[&str], root: &Path, out_dir: &Path) -> Vec<PathBuf> {
    let mut errors = None;
    let objects = files
        .into_iter()
        .enumerate()
        .map(|(i, file)| {
            let asm_path = root.join(file);
            let object_path = out_dir.join(format!("asm_{i}.o"));

            println!("cargo:rerun-if-changed={}", asm_path.display());

            let output = Command::new("nasm")
                .args(&["-f", "elf64", "-o"])
                .args([&object_path, &asm_path])
                .output()
                .expect("to run nasm");
            if !output.status.success() {
                if let None = errors {
                    errors = Some(Vec::new());
                }
                errors.as_mut().unwrap().push(format!(
                    "nasm failed to compile {:?}: {}",
                    asm_path,
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            object_path
        })
        .collect::<Vec<_>>();
    if let Some(errors) = errors {
        for error in &errors {
            println!("{}", error);
        }
        panic!(
            "nasm failed to compile {} file{}",
            errors.len(),
            if errors.len() == 1 { "" } else { "s" }
        );
    }
    objects
}

fn build_boot_dir(root: &Path, iso_dir: &Path) -> PathBuf {
    if iso_dir.exists() {
        if !iso_dir.is_dir() {
            panic!("iso dir {:?} is not a dir", iso_dir);
        }
        fs::remove_dir_all(&iso_dir).expect("to remove old iso dir");
    }

    let boot_dir = iso_dir.join("boot");
    fs::create_dir_all(&boot_dir).expect("to create boot dir");

    let grub_dir = boot_dir.as_path().join("grub");
    fs::create_dir_all(&grub_dir).expect("to create grub dir");

    let grub_cfg = root.join("grub.cfg");
    println!("cargo:rerun-if-changed={}", grub_cfg.display());
    let grub_cfg_target = grub_dir.join("grub.cfg");

    if !grub_cfg.exists() {
        panic!("grub.cfg not found");
    }
    fs::copy(grub_cfg, grub_cfg_target).expect("to copy grub.cfg");

    boot_dir
}

fn build_kernel_elf(root: &Path, boot_dir: &Path, objects: Vec<PathBuf>) {
    let linker_script = root.join("linker.ld");
    println!("cargo:rerun-if-changed={}", linker_script.display());

    // copy the kernel binary
    let kernel_lib = env::var("CARGO_STATICLIB_FILE_KERNEL_kernel").expect("to find kernel bin");
    let kernel_bin = boot_dir.join("kernel.bin");

    let output = Command::new("ld")
        .arg("-n")
        .arg("-T")
        .arg(linker_script)
        .args(objects)
        .arg(kernel_lib)
        .arg("-o")
        .arg(&kernel_bin)
        .output()
        .expect("to run ld");
    if !output.status.success() {
        panic!("ld failed: {}", String::from_utf8_lossy(&output.stderr));
    }
}

fn build_kernel_iso(iso_out: &Path, iso_dir: &Path) {
    let out = Command::new("grub-mkrescue")
        .arg("-o")
        .arg(iso_out)
        .arg(iso_dir)
        .status()
        .expect("to run grub-mkrescue");
    if !out.success() {
        panic!("grub-mkrescue failed");
    }
}

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("to find the manifest dir");
    let root = PathBuf::from(manifest);

    let final_iso = root.join("bruh_os.iso");
    if final_iso.exists() {
        fs::remove_file(&final_iso).expect("to remove old iso");
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let iso_dir = out_dir.join("isodir");

    let objects = build_assembly_files(ASM, &root, &out_dir);
    let boot_dir = build_boot_dir(&root, &iso_dir);
    build_kernel_elf(&root, &boot_dir, objects);
    build_kernel_iso(&final_iso, &iso_dir);
}
