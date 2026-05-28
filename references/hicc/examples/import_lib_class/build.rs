fn main() {
    hicc_build::Build::new()
        .rust_file("src/main.rs")
        .compile("example");
    println!("cargo::rustc-link-lib=example");
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/main.rs");
}
