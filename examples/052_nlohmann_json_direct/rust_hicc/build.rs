fn main() {
    let cpp_dir = std::path::PathBuf::from("../cpp");
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap();
    let nlohmann_dir = repo_root.join("references/nlohmann-json/single_include");

    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.include(&cpp_dir);
    cc_build.include(&nlohmann_dir);
    cc_build.include(".");
    cc_build.cpp(true);
    cc_build.std("c++17");
    cc_build.file(cpp_dir.join("nlohmann_json_direct.cpp"));

    build.rust_file("src/lib.rs").compile("nlohmann_json_direct");

    println!("cargo::rustc-link-lib=nlohmann_json_direct");
    #[cfg(target_os = "macos")]
    println!("cargo::rustc-link-lib=c++");
    #[cfg(not(any(target_os = "macos", all(target_os = "windows", target_env = "msvc"))))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/lib.rs");
    println!("cargo::rerun-if-changed=../cpp/nlohmann_json_direct.cpp");
    println!("cargo::rerun-if-changed=../cpp/nlohmann_json_direct.h");
}
