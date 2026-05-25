fn main() {
    let cpp_dir = std::path::PathBuf::from("../cpp");

    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.include(&cpp_dir);
    cc_build.include(".");
    cc_build.cpp(true);
    cc_build.file(cpp_dir.join("inheritance_single.cpp"));

    build.rust_file("src/main.rs").compile("inheritance_single");

    println!("cargo::rustc-link-lib=inheritance_single");
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/main.rs");
    println!("cargo::rerun-if-changed=../cpp/inheritance_single.cpp");
    println!("cargo::rerun-if-changed=../cpp/inheritance_single.h");
}
