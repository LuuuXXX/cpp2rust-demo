use crate::{Attr, ImportClass, ImportLib};

mod token;
use token::*;

mod function;
use function::*;

mod class;
use class::*;

pub struct ExportClasses {
    imported: ImportClass,
}

impl ExportClasses {
    pub fn new(imported: ImportClass) -> Self {
        Self { imported }
    }

    pub fn export(&self) -> syn::Result<String> {
        let mut codes = String::new();
        for cpp in self.imported.cpps.iter() {
            codes.push_str(&cpp.export());
        }

        let mut classes = vec![];
        for c in self.imported.class.iter() {
            classes.push(Class::try_from(c)?);
        }

        for c in classes.iter() {
            codes.push_str(&c.export_methods_class()?);
        }

        for c in classes.iter() {
            codes.push_str(&c.export(&self.imported)?);
            codes.push_str(&RemoteProxy::new(c, &self.imported).export()?);
        }
        Ok(codes)
    }
}

pub struct ExportLib {
    imported: ImportLib,
    link: String,
    line: usize,
}

impl ExportLib {
    pub fn try_from(lib: ImportLib) -> syn::Result<Self> {
        let Ok(Some(link)) = Attr::get_value("link_name", &lib.attrs) else {
            return Err(syn::Error::new(lib.span, "not found #![link_name = ...]"));
        };
        let line = Attr::get_attr("link_name", &lib.attrs)
            .unwrap()
            .span()
            .start()
            .line;
        Ok(Self {
            imported: lib,
            link,
            line,
        })
    }

    pub fn export(&self) -> syn::Result<String> {
        let mut codes = String::new();
        for cpp in self.imported.cpps.iter() {
            codes.push_str(&cpp.export());
        }
        codes.push_str(&format!(
            "#line {}\nEXPORT_METHODS_BEG({}) {{\n",
            self.line, self.link
        ));

        for f in self.imported.funcs.iter() {
            let f = Function::try_from(f)?;
            codes.push_str(&f.export_test()?);
            codes.push_str(&f.export()?);
        }

        codes.push_str(&format!("#line {}\n}} EXPORT_METHODS_END();\n", self.line));
        Ok(codes)
    }
}
