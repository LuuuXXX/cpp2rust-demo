fn main() {
    let cpp_dir = std::path::PathBuf::from("../cpp");

    cc::Build::new()
        .include(&cpp_dir)
        .include(".")
        .cpp(true)
        .file(cpp_dir.join("variadic_functions.cpp"))
        .compile("variadic_functions");

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=../cpp/variadic_functions.cpp");
    println!("cargo::rerun-if-changed=../cpp/variadic_functions.h");
}
