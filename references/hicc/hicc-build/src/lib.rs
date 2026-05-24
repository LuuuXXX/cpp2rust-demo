use hicc_autogen::{Cpp, ExportClasses, ExportLib, ImportClass, ImportLib};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;

pub struct Build {
    build: cc::Build,
}

impl Default for Build {
    fn default() -> Self {
        Self::new()
    }
}

impl Build {
    pub fn new() -> Self {
        let mut this = Self {
            build: cc::Build::new(),
        };
        this.init();
        this
    }

    /// 基于输入的`rust`文件自动生成`c++`适配代码, 并添加到待编译的`cpp`文件列表中.
    ///
    /// 注: 生成的文件在环境变量`$OUT_DIR`指定的目录下.
    pub fn rust_file<P: AsRef<Path>>(&mut self, src: P) -> &mut Self {
        let src = src.as_ref().as_os_str().to_string_lossy().into_owned();
        let codes = self
            .generate_cpp_codes(&src)
            .unwrap_or_else(|e| self.map_err(e, &src));
        self.write_cpp_file(codes, &src)
            .unwrap_or_else(|e| panic!("write_cpp_file error: {e:?}"));
        self
    }

    /// 基于输入的`rust`文件自动生成`c++`适配代码，并写入指定的头文件.
    ///
    /// 头文件并不会加入到待编译的`cpp`源文件列表. 使用时需要在其他文件中显式包含生成的头文件.
    pub fn cpp_header<P1: AsRef<Path>, P2: AsRef<Path>>(&mut self, src: P1, hdr: P2) -> &mut Self {
        let src = src.as_ref().as_os_str().to_string_lossy().into_owned();
        let hdr = hdr.as_ref().as_os_str().to_string_lossy().into_owned();
        let codes = self
            .generate_cpp_codes(&src)
            .unwrap_or_else(|e| self.map_err(e, &src));
        self.write_cpp_header(&src, codes, &hdr)
            .unwrap_or_else(|e| panic!("write_header error: {e:?}"));
        self
    }

    fn generate_cpp_codes(&self, src: &str) -> syn::Result<String> {
        let mut codes = String::new();
        codes.push_str("#include <hicc/hicc.hpp>\n");
        codes.push_str(&format!("#line 0 R\"({src})\"\n"));

        let content =
            fs::read_to_string(src).unwrap_or_else(|_| panic!("failed to read rust file `{src}`"));
        let file: syn::File = syn::parse_str(&content)?;
        for item in file.items.iter() {
            let syn::Item::Macro(mac) = item else {
                continue;
            };
            match hicc_macro(&mac.mac.path) {
                HiccMacro::Class => {
                    let c = syn::parse2::<ImportClass>(mac.mac.tokens.clone())?;
                    codes.push_str(&ExportClasses::new(c).export()?)
                }
                HiccMacro::Lib => {
                    let lib = syn::parse2::<ImportLib>(mac.mac.tokens.clone())?;
                    let lib = ExportLib::try_from(lib)?;
                    codes.push_str(&lib.export()?);
                }
                HiccMacro::Cpp => {
                    let cpp = syn::parse2::<Cpp>(mac.mac.tokens.clone())?;
                    codes.push_str(&cpp.export());
                }
                HiccMacro::Error => {
                    return Err(syn::Error::new(mac.span(), "undefined macro!"));
                }
                _ => {}
            }
        }
        Ok(codes)
    }

    fn map_err(&self, e: syn::Error, file: &str) -> ! {
        let pos = e.span().start();
        panic!("+++ {file} {}:{}\n{e:?}\n--- End", pos.line, pos.column);
    }

    fn write_cpp_header(&self, src: &str, codes: String, hdr: &str) -> Result<(), cc::Error> {
        let hdr_macro = src
            .replace(|c: char| !c.is_ascii_alphanumeric(), "_")
            .to_ascii_uppercase();

        if let Some(parent) = <str as AsRef<Path>>::as_ref(hdr).parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut file = fs::File::create(hdr).map_err(cc::Error::from)?;
        writeln!(file, "#ifndef {hdr_macro}").map_err(cc::Error::from)?;
        writeln!(file, "#define {hdr_macro}\n").map_err(cc::Error::from)?;
        writeln!(file, "{codes}").map_err(cc::Error::from)?;
        writeln!(file, "#endif").map_err(cc::Error::from)?;
        Ok(())
    }

    fn write_cpp_file(&mut self, codes: String, src: &str) -> Result<(), cc::Error> {
        let Ok(out_dir) = env::var("OUT_DIR") else {
            return Err(io::Error::new(io::ErrorKind::NotFound, "not found $OUT_DIR").into());
        };
        let mut dst = src.replace(['/', '\\'], "-");
        dst.push_str(".cpp");
        let path = Path::new(&out_dir).join(dst);
        let mut file = fs::File::create(path.clone()).map_err(cc::Error::from)?;
        writeln!(file, "{codes}").map_err(cc::Error::from)?;
        self.file(path.as_path());
        Ok(())
    }

    fn init(&mut self) {
        if let Some(include) = env::var_os("DEP_HICC_INCLUDE") {
            self.build.include(include);
        }
        if let Some(include) = env::var_os("DEP_HICC_STD_INCLUDE") {
            self.build.include(include);
        }
        self.build.include(".");
        self.build.cpp(true);
    }
}

impl Deref for Build {
    type Target = cc::Build;
    fn deref(&self) -> &Self::Target {
        &self.build
    }
}

impl DerefMut for Build {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.build
    }
}

enum HiccMacro {
    Class,
    Lib,
    Cpp,
    Ignore,
    Error,
}

fn hicc_macro(path: &syn::Path) -> HiccMacro {
    match (
        path.segments.get(0),
        path.segments.get(1),
        path.segments.get(2),
    ) {
        (Some(p1), Some(p2), None) if p1.ident == "hicc" && p2.ident == "import_class" => {
            HiccMacro::Class
        }
        (Some(p1), None, None) if p1.ident == "import_class" => HiccMacro::Class,
        (Some(p1), Some(p2), None) if p1.ident == "hicc" && p2.ident == "import_lib" => {
            HiccMacro::Lib
        }
        (Some(p1), None, None) if p1.ident == "import_lib" => HiccMacro::Lib,
        (Some(p1), Some(p2), None) if p1.ident == "hicc" && p2.ident == "cpp" => HiccMacro::Cpp,
        (Some(p1), None, None) if p1.ident == "cpp" => HiccMacro::Cpp,
        (Some(p1), _, _) if p1.ident == "hicc" => HiccMacro::Error,
        _ => HiccMacro::Ignore,
    }
}

/// 规范化路径，在Windows平台上去除扩展长度路径语法(\\?\前缀)
/// 对于其他平台，原样返回路径
/// 参考文档： <https://doc.rust-lang.org/std/fs/fn.canonicalize.html#platform-specific-behavior>
pub fn normalize_windows_path(path: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        path.display()
            .to_string()
            .strip_prefix(r"\\?\")
            .map(|s| Path::new(s).to_path_buf())
            .unwrap_or_else(|| path.to_path_buf())
    } else {
        path.to_path_buf()
    }
}
