use clang::{Clang, EntityKind, Index};
use std::path::Path;

fn main() {
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);
    let file = Path::new("/tmp/bigintegertest_ffi.cpp2rust");
    let tu = index.parser(file).arguments(&["-xc++", "-std=c++17"]).parse().unwrap();
    let root = tu.get_entity();
    
    println!("=== Top-level entity locations ===");
    for entity in root.get_children() {
        if entity.get_location().map(|l| l.is_in_system_header()).unwrap_or(true) {
            continue;
        }
        let kind = entity.get_kind();
        let name = entity.get_name().unwrap_or_default();
        
        // spelling location (physical)
        let spelling_file = entity.get_location()
            .map(|l| {
                let sl = l.get_spelling_location();
                sl.file.map(|f| f.get_path().to_string_lossy().to_string()).unwrap_or("(none)".to_string())
            })
            .unwrap_or("(none)".to_string());
        
        // file location (presumed, follows #line markers)
        let file_loc = entity.get_location()
            .map(|l| {
                let fl = l.get_file_location();
                fl.file.map(|f| f.get_path().to_string_lossy().to_string()).unwrap_or("(none)".to_string())
            })
            .unwrap_or("(none)".to_string());
        
        println!("{:40} kind={:?}", name, kind);
        println!("  spelling_file: {}", spelling_file);
        println!("  file_location: {}", file_loc);
        
        // If it's a namespace, show its children too
        if kind == EntityKind::Namespace {
            for child in entity.get_children() {
                if child.get_location().map(|l| l.is_in_system_header()).unwrap_or(true) { continue; }
                let cname = child.get_name().unwrap_or_default();
                let ckind = child.get_kind();
                let cfile = child.get_location()
                    .map(|l| l.get_file_location().file.map(|f| f.get_path().to_string_lossy().to_string()).unwrap_or("(none)".to_string()))
                    .unwrap_or("(none)".to_string());
                let cspell = child.get_location()
                    .map(|l| l.get_spelling_location().file.map(|f| f.get_path().to_string_lossy().to_string()).unwrap_or("(none)".to_string()))
                    .unwrap_or("(none)".to_string());
                println!("  child {:30} kind={:?}", cname, ckind);
                println!("    spelling: {}", cspell);
                println!("    file_loc: {}", cfile);
            }
        }
    }
}
