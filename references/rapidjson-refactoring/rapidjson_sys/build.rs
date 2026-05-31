fn main() {
    let include = "../rapidjson_legacy/include";

    // ── Compile all C++ shim files into a single archive ──────────────────────
    cc::Build::new()
        .cpp(true)
        .include(include)
        .file("shim/bigintegertest_ffi.cpp")
        .file("shim/document_ffi.cpp")
        .file("shim/writer_ffi.cpp")
        .file("shim/reader_ffi.cpp")
        .file("shim/pointer_ffi.cpp")
        .file("shim/schema_ffi.cpp")
        .file("shim/allocator_ffi.cpp")
        .file("shim/encoding_ffi.cpp")
        .file("shim/value_ffi.cpp")
        .file("shim/stringbuffer_ffi.cpp")
        .compile("rapidjson_shim");

    // ── Generate Rust bindings for each shim header ───────────────────────────
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let shim_headers: &[(&str, &str, &[&str], &[&str])] = &[
        (
            "shim/bigintegertest_ffi.h",
            "ffi_bigintegertest_bindings.rs",
            &["RapidJsonBigIntegerHandle"],
            &["rapidjson_biginteger_.*"],
        ),
        (
            "shim/document_ffi.h",
            "ffi_document_bindings.rs",
            &["RapidJsonDocumentHandle", "RapidJsonValueHandle", "RapidJsonAllocatorHandle"],
            &["rapidjson_document_.*", "rapidjson_value_.*"],
        ),
        (
            "shim/writer_ffi.h",
            "ffi_writer_bindings.rs",
            &[
                "RapidJsonStringBufferHandle",
                "RapidJsonWriterHandle",
                "RapidJsonPrettyWriterHandle",
            ],
            &[
                "rapidjson_stringbuffer_.*",
                "rapidjson_writer_.*",
                "rapidjson_prettywriter_.*",
            ],
        ),
        (
            "shim/reader_ffi.h",
            "ffi_reader_bindings.rs",
            &[
                "RapidJsonReaderHandle",
                "RapidJsonStringStreamHandle",
                "RapidJsonInsituStreamHandle",
                "RapidJsonHandlerCallbacks",
            ],
            &["rapidjson_reader_.*", "rapidjson_stringstream_.*", "rapidjson_insitustream_.*"],
        ),
        (
            "shim/pointer_ffi.h",
            "ffi_pointer_bindings.rs",
            &["RapidJsonPointerHandle"],
            &["rapidjson_pointer_.*"],
        ),
        (
            "shim/schema_ffi.h",
            "ffi_schema_bindings.rs",
            &["RapidJsonSchemaDocHandle", "RapidJsonSchemaValidHandle"],
            &["rapidjson_schemadoc_.*", "rapidjson_schemavalidator_.*"],
        ),
        (
            "shim/allocator_ffi.h",
            "ffi_allocator_bindings.rs",
            &["RapidJsonCrtAllocHandle", "RapidJsonMpAllocHandle"],
            &["rapidjson_crtallocator_.*", "rapidjson_mpallocator_.*"],
        ),
        (
            "shim/encoding_ffi.h",
            "ffi_encoding_bindings.rs",
            &[],
            &["rapidjson_utf8_.*", "rapidjson_utf16_.*", "rapidjson_utf32_.*",
              "rapidjson_transcode_.*"],
        ),
        (
            "shim/value_ffi.h",
            "ffi_value_bindings.rs",
            &[],
            &["rapidjson_value_get_member_at", "rapidjson_value_remove_member",
              "rapidjson_value_pop_back", "rapidjson_value_erase_index",
              "rapidjson_value_reserve", "rapidjson_value_is_lossless_.*",
              "rapidjson_value_get_float", "rapidjson_value_clear_.*"],
        ),
        (
            "shim/stringbuffer_ffi.h",
            "ffi_stringbuffer_bindings.rs",
            &[],
            &["rapidjson_stringbuffer_reserve", "rapidjson_stringbuffer_get_capacity",
              "rapidjson_stringbuffer_get_stack_top", "rapidjson_stringbuffer_push_n",
              "rapidjson_stringbuffer_flush"],
        ),
    ];

    for (header, out_file, types, funcs) in shim_headers {
        let mut builder = bindgen::Builder::default()
            .header(*header)
            .clang_arg(format!("-I{}", include));

        for t in *types {
            builder = builder.allowlist_type(t);
        }
        for f in *funcs {
            builder = builder.allowlist_function(f);
        }

        let bindings = builder
            .generate()
            .unwrap_or_else(|_| panic!("Unable to generate bindings for {}", header));

        bindings
            .write_to_file(out_path.join(out_file))
            .unwrap_or_else(|_| panic!("Couldn't write {}", out_file));
    }
}
