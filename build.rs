use autotools::Config;
use std::path::Path;
use std::process::{Command, Stdio};
// const LIBEXIF_LINK = "https://github.com/libexif/libexif/releases/download/v0.6.24/libexif-0.6.24.zip";
const LIBEXIF_GIT: &str = "https://github.com/libexif/libexif";
const LIBEXIF_TAG: &str = "v0.6.24";

pub fn clone<P: AsRef<Path>>(path: P) {
    std::env::set_current_dir(path.as_ref()).unwrap();
    if !Path::new("libexif").exists() {
        let clone = Command::new("git")
            .args([
                "clone",
                LIBEXIF_GIT,
                "--branch",
                LIBEXIF_TAG,
                "--depth",
                "1",
            ])
            .stdout(Stdio::inherit())
            .status()
            .expect("Failed to git clone");
        if !clone.success() {
            panic!("Failed to run git clone {}", LIBEXIF_GIT);
        };
    }
}

/// Call after dst.build
pub fn generate_bindings<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    std::env::set_current_dir(path.join("include")).unwrap();
    let bindings = bindgen::builder()
        .header("libexif/exif-byte-order.h")
        .header("libexif/exif-content.h")
        .header("libexif/exif-data-type.h")
        .header("libexif/exif-data.h")
        .header("libexif/exif-entry.h")
        .header("libexif/exif-format.h")
        .header("libexif/exif-ifd.h")
        .header("libexif/exif-loader.h")
        .header("libexif/exif-log.h")
        .header("libexif/exif-mem.h")
        .header("libexif/exif-mnote-data.h")
        .header("libexif/exif-tag.h")
        .header("libexif/exif-utils.h")
        .rustified_enum("Exif.*")
        .clang_arg(format!("-I{}", path.join("include").display()))
        .generate()
        .unwrap();
    bindings.write_to_file(path.join("bindings.rs")).unwrap();
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    clone(out_dir);
    #[cfg(feature = "static")]
    let dst = Config::new("libexif")
        .reconf("--install")
        .enable("static", None)
        .build();
    #[cfg(not(feature = "static"))]
    let dst = Config::new("libexif").reconf("--install").build();

    generate_bindings(out_dir);
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
    #[cfg(feature = "static")]
    println!("cargo:rustc-link-lib=static=exif");
    #[cfg(not(feature = "static"))]
    println!("cargo:rustc-link-lib=exif");
    println!("cargo:rerun-if-changed=build.rs");
}
