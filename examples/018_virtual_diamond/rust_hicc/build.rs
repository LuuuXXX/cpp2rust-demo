fn main() {
    let cpp_dir = std::path::PathBuf::from("../cpp");
    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.include(&cpp_dir).include(".").cpp(true)
            .file(cpp_dir.join("virtual_diamond.cpp"));

    build.rust_file("src/lib.rs").compile("virtual_diamond");

    println!("cargo::rustc-link-lib=virtual_diamond");
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/lib.rs");
    println!("cargo::rerun-if-changed=../cpp/virtual_diamond.cpp");
    println!("cargo::rerun-if-changed=../cpp/virtual_diamond.h");
}
