fn main() {
    let cpp_dir = std::path::PathBuf::from("../cpp");

    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.include(&cpp_dir);
    cc_build.cpp(true);
    build.rust_file("src/main.rs").compile("array_basic");

    println!("cargo::rustc-link-lib=array_basic");
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/main.rs");
    println!("cargo::rerun-if-changed=../cpp/array_basic.cpp");
    println!("cargo::rerun-if-changed=../cpp/array_basic.h");
}
