use cpp2rust_demo::ast_parser;
use cpp2rust_demo::extractor;
use cpp2rust_demo::generator::hicc_codegen;
use std::path::Path;

fn main() {
    let cpp2rust_file = Path::new("/tmp/bigintegertest_ffi.cpp2rust");
    let (system_includes, project_header) = extractor::read_source_includes(
        Path::new("/tmp/workspace/LuuuXXX/cpp2rust-demo/references/rapidjson-refactoring/rapidjson_sys/shim/bigintegertest_ffi.cpp")
    );
    let ast = ast_parser::parse_preprocessed(cpp2rust_file).expect("parse failed");
    let spec = extractor::extract(
        &ast,
        "bigintegertest_ffi",
        &system_includes,
        project_header.as_deref(),
    );
    let code = hicc_codegen::generate(&spec);
    println!("{}", code);
}
