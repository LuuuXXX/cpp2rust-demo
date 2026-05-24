use std::path::Path;

fn main() {
    let include = Path::new("./include").canonicalize().unwrap();
    println!("cargo:include={}", hicc_build::normalize_windows_path(&include).display());
    println!("cargo::rerun-if-changed=build.rs");
}
