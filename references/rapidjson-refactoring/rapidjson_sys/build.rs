fn main() {
    // Ensure the C shim is compiled and linked.
    cc::Build::new()
        .cpp(true)
        .file("shim/bigintegertest_ffi.cpp")
        .include("../rapidjson_legacy/include")
        .compile("bigintegertest_ffi");

    // Generate bindings for the C shim header.
    let header = "shim/bigintegertest_ffi.h";

    let bindings = bindgen::Builder::default()
        .header(header)
        .allowlist_type("RapidJsonBigIntegerHandle")
        .allowlist_function("rapidjson_biginteger_.*")
        .generate()
        .expect("Unable to generate bindings for bigintegertest_ffi.h");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ffi_bigintegertest_bindings.rs"))
        .expect("Couldn't write bindings");
}
