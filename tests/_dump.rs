use cpp2rust_demo::{ast_parser, extractor, generator::hicc_codegen};

#[test]
#[cfg_attr(not(feature = "full-test"), ignore)]
fn dump025() {
    let cpp_file = std::path::PathBuf::from("examples/025_template_class/cpp/template_class.cpp");
    let pre = std::path::PathBuf::from("/tmp/tc.cpp2rust");
    std::process::Command::new("g++")
        .args(["-E", "-C"])
        .arg(&cpp_file)
        .arg("-o")
        .arg(&pre)
        .status()
        .unwrap();
    let ast = ast_parser::parse_preprocessed(&pre).unwrap();
    eprintln!("=== template_classes ===");
    for tc in &ast.template_classes {
        eprintln!("TC {} params={:?}", tc.name, tc.type_params.iter().map(|p| &p.name).collect::<Vec<_>>());
    }
    eprintln!("=== classes (name: field types) ===");
    for c in &ast.classes {
        let fts: Vec<String> = c.fields.iter().map(|f| format!("{}:{}", f.name, f.type_name)).collect();
        eprintln!("CLS {} from_current={} fields={:?}", c.name, c.is_from_current_file, fts);
    }
    let (si, ph) = extractor::read_source_includes(&cpp_file);
    let spec = extractor::extract(&ast, "template_class", &si, ph.as_deref());
    std::env::set_var("CPP2RUST_GEN_TEMPLATES", "1");
    let raw = hicc_codegen::generate(&spec);
    std::env::remove_var("CPP2RUST_GEN_TEMPLATES");
    eprintln!("=== GENERATED ===\n{}", raw);
}
