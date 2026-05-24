use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let Ok(out) = env::var("OUT_DIR") else {
        panic!("not found $OUT_DIR");
    };
    let mut build = hicc_build::Build::new();

    // cast-funtion-type
    #[cfg(target_env = "msvc")]
    build.flag("/we4054").flag("/we4191");
    #[cfg(target_env = "gnu")]
    build.flag("-Werror=cast-function-type");
    
    // bigobj
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    build.flag("/bigobj");
    #[cfg(all(target_os = "windows", target_env = "gnu"))]
    build.flag("-Wa,-mbig-obj");

    let mut path = Path::new(&out).join("include");
    let _ = fs::create_dir_all(path.as_os_str());
    let include = hicc_build::normalize_windows_path(
        &path
            .canonicalize()
            .unwrap_or_else(|_| panic!("not found dir: $OUT_DIR/include")),
    );
    println!("cargo:include={}", include.display());
    build.include(include);
    path.push("hicc");
    path.push("std");

    let src = Path::new(".").join("src");
    let files = [
        ("std_string.rs", "string.hpp"),
        ("std_vector.rs", "vector.hpp"),
        ("std_array.rs", "array.hpp"),
        ("std_deque.rs", "deque.hpp"),
        ("std_queue.rs", "queue.hpp"),
        ("std_stack.rs", "stack.hpp"),
        ("std_list.rs", "list.hpp"),
        ("std_forward_list.rs", "forward_list.hpp"),
        ("std_set.rs", "set.hpp"),
        ("std_map.rs", "map.hpp"),
        ("std_unordered_set.rs", "unordered_set.hpp"),
        ("std_unordered_map.rs", "unordered_map.hpp"),
    ];

    for file in files {
        let mut src = src.clone();
        src.push(file.0);
        let mut hdr = path.clone();
        hdr.push(file.1);
        println!("cargo::rerun-if-changed={:?}", src.as_os_str());
        build.cpp_header(src, hdr);
    }

    build.rust_file("src/lib.rs");
    let std_test = Path::new("src").join("std_test");
    let test_files = [
        "std_vector.rs",
        "std_array.rs",
        "std_deque.rs",
        "std_queue.rs",
        "std_stack.rs",
        "std_list.rs",
        "std_forward_list.rs",
        "std_set.rs",
        "std_map.rs",
        "std_unordered_set.rs",
        "std_unordered_map.rs",
    ];
    test_files.iter().for_each(|f| {
        build.rust_file(std_test.join(f));
    });

    build.compile("hicc_std_cc");
    println!("cargo::rustc-link-lib=hicc_std_cc");
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/lib.rs");
}
