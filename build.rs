use autotools::Config;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const LIBEXIF_LINK: &str =
    "https://github.com/libexif/libexif/releases/download/v0.6.24/libexif-0.6.24.zip";
const LIBEXIF_HASH: &str = "12fa9fc73d3610f752f9a0ef5da1269e76b1caab7aca83f5174ca0c9565ca802";
const LIBEXIF_GIT: &str = "https://github.com/libexif/libexif";
const LIBEXIF_TAG: &str = "v0.6.24";
const LIBEXIF_VERSION: &str = "0.6.24";

pub fn clone<P: AsRef<Path>>(path: P) {
    // std::env::set_current_dir(path.as_ref()).unwrap();
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

pub fn wget(url: &str) -> Result<PathBuf, String> {
    let mut buffer = Vec::new();
    let mut response = http_req::request::get(url, &mut buffer).map_err(|e| e.to_string())?;

    while response.status_code().is_redirect() {
        buffer.clear();
        let url = response.headers().get("Location").unwrap();
        response = http_req::request::get(url, &mut buffer).map_err(|e| e.to_string())?;
    }

    if !response.status_code().is_success() {
        return Err(format!(
            "Download Error: HTTP status code {}",
            &response.status_code(),
        ));
    }
    assert_eq!(sha256::digest_bytes(&buffer), LIBEXIF_HASH);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let mut libexif_zip = std::fs::File::create(out_dir.join("libexif.zip")).unwrap();
    libexif_zip.write_all(&buffer).map_err(|e| e.to_string())?;

    Ok(out_dir.join("libexif.zip"))
}

pub fn unzip(zip: impl AsRef<Path>, dir: impl AsRef<Path>) -> Result<(), String> {
    let file = std::fs::File::open(zip).map_err(|e| e.to_string())?;
    let mut zip = zip::read::ZipArchive::new(file).map_err(|e| e.to_string())?;
    zip.extract(dir).map_err(|e| e.to_string())?;
    Ok(())
}

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
    std::env::set_current_dir(out_dir).unwrap();
    // clone(out_dir);
    unzip(wget(LIBEXIF_LINK).unwrap(), out_dir).unwrap();
    let libexif_dir = format!("libexif-{}", LIBEXIF_VERSION);

    #[cfg(feature = "static")]
    let dst = Config::new(libexif_dir)
        // .reconf("--install")
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
