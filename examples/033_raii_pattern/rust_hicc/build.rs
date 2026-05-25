fn main() {
    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.cpp(true);
    build.rust_file("src/main.rs").compile("raii_pattern");
    println!("cargo::rustc-link-lib=raii_pattern");
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
}