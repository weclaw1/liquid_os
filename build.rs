use std::process::Command;
use std::env;
use std::path::Path;

fn nasm_compile(arch: &str) {
    match arch {
        "x86_64" => {
            
            assert!(Command::new("mkdir")
            .arg("-p")
            .arg(&format!("build/arch/{}/boot/", arch))
            .status()
            .expect("failed to execute mkdir")
            .success(),
            "mkdir failed");

            assert!(Command::new("nasm")
            .arg(&format!("src/arch/{}/boot/boot.asm", arch))
            .args(&["-felf64", "-o"])
            .arg(&format!("build/arch/{}/boot/boot.o", arch))
            .status()
            .expect("failed to execute nasm")
            .success(),
            "compilation of boot.asm failed");
        },
        _ => panic!("architecture not supported"),
    }

    println!("cargo:rerun-if-changed=src/arch/{}/boot/boot.asm", arch);
}

fn main() {
    match env::var("ARCH") {
        Ok(arch) => {
            nasm_compile(&arch);
        },
        Err(err) => {
            nasm_compile("x86_64");
        },
    }

}