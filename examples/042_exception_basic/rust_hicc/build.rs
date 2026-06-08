fn main() {
    let cpp_dir = std::path::PathBuf::from("../cpp");

    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.include(&cpp_dir);
    cc_build.include(".");
    cc_build.cpp(true);
    cc_build.file(cpp_dir.join("exception_basic.cpp"));

    build.rust_file("src/main.rs").compile("exception_basic");

    println!("cargo::rustc-link-lib=exception_basic");
    #[cfg(target_os = "macos")]
    println!("cargo::rustc-link-lib=c++");
    #[cfg(not(any(target_os = "macos", all(target_os = "windows", target_env = "msvc"))))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/main.rs");
    println!("cargo::rerun-if-changed=../cpp/exception_basic.cpp");
    println!("cargo::rerun-if-changed=../cpp/exception_basic.h");
}
