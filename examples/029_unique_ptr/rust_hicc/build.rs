fn main() {
    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.cpp(true);
    build.rust_file("src/main.rs").compile("unique_ptr");
    println!("cargo::rustc-link-lib=unique_ptr");
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
}