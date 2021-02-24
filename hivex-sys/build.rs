use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=./build.rs");
    // println!("cargo:rerun-if-changed=./src/lib.rs");

    let dst = PathBuf::from(env::var("OUT_DIR").unwrap());
    // Clean Source
    Command::new("make")
        .current_dir("./hivex")
        .arg("distclean")
        .output()
        .expect("failed to clean ./hivex-sys/hivex during build process");
    // Run ./autogen.sh
    Command::new("./autogen.sh")
        .current_dir("./hivex")
        .output()
        .expect("an error occured while running ./autogen.sh during build process");
    // Run configure with the dst variable
    Command::new("./configure")
        .arg(format!("--prefix={}", dst.display()))
        .current_dir("./hivex")
        .output()
        .expect("an error occured while running ./configure during build process");
    // Run make to install the lib to the dst directory
    Command::new("make")
        .arg("install")
        .current_dir("./hivex")
        .output()
        .expect("an error occured while running 'make install' during build process");
    // Configure rustc
    println!("cargo:root={}", dst.display());
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=hivex");
    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("./hivex/lib/hivex.h")
        .rustified_enum("hive_type")
        .generate()
        .expect("an error occurred while generating bindings");

    // Write the bindings to dst
    bindings
        .write_to_file(dst.join("bindings.rs"))
        .expect("an error occurred while writing bindings");
}
