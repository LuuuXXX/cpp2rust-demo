fn main() {
    let cpp_dir = std::path::PathBuf::from("../cpp");

    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    // 与各示例 cpp/standalone.sh 的 `-std=c++17` 保持一致：示例普遍使用 C++17
    // 特性（折叠表达式、if constexpr 等）。g++/clang++ 默认即为 C++17，但 MSVC
    // cl.exe 默认仍是 C++14，会导致 hicc cpp! 块编译失败（如 028 折叠表达式）。
    // 显式固定标准，保证三大平台行为一致。
    cc_build.std("c++17");
    cc_build.include(&cpp_dir);
    cc_build.include(".");
    cc_build.cpp(true);
    cc_build.file(cpp_dir.join("template_specialization.cpp"));

    build.rust_file("src/lib.rs").compile("template_specialization");

    println!("cargo::rustc-link-lib=template_specialization");
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/lib.rs");
    println!("cargo::rerun-if-changed=../cpp/template_specialization.cpp");
    println!("cargo::rerun-if-changed=../cpp/template_specialization.h");
}
