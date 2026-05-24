use crate::path_2_string;
use proc_macro2::{Delimiter, Group, LineColumn, TokenStream, TokenTree};
use syn::parse;
use syn::spanned::Spanned;

/// 对应`cpp!`, 只能是字符串形式存在, 和汇编不同，并不在`rust`上下文中调用，不支持任何参数.
#[derive(Clone, Debug)]
pub struct Cpp {
    cpp: TokenStream, //syn::LitStr,
}

impl parse::Parse for Cpp {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        //let cpp = input.parse::<syn::LitStr>()?;
        let cpp = input.parse::<TokenStream>()?;
        Ok(Self { cpp })
    }
}

impl Cpp {
    pub fn from_item(item: &syn::Item) -> Result<Option<Self>, syn::Error> {
        let syn::Item::Macro(macro_item) = item else {
            return Ok(None);
        };
        let path = path_2_string(&macro_item.mac.path);
        if matches!(path.as_str(), "cpp" | "hicc :: cpp" | ":: hicc :: cpp") {
            let cpp = syn::parse2::<TokenStream>(macro_item.mac.tokens.clone())?;
            return Ok(Some(Self { cpp }));
        }
        Ok(None)
    }

    /// 生成CPP代码. 只能在`build.rs`中调用.
    pub fn export(&self) -> String {
        let mut codes = format!("#line {}\n", self.cpp.span().start().line);
        token_2_string(self.cpp.clone(), &mut codes);
        if !matches!(codes.as_bytes().last(), Some(b'\n')) {
            codes.push('\n');
        }
        codes
    }
}

fn token_2_string(tokens: TokenStream, codes: &mut String) {
    if tokens.is_empty() {
        return;
    }
    let mut pos = tokens.span().start();
    for token in tokens {
        pos_2_string(pos, token.span().start(), codes);
        pos = token.span().end();
        match token {
            TokenTree::Ident(ident) => codes.push_str(&format!("{ident}")),
            TokenTree::Literal(lit) => codes.push_str(&format!("{lit}")),
            TokenTree::Punct(punct) => codes.push_str(&format!("{punct}")),
            TokenTree::Group(group) => group_2_string(group, codes),
        }
    }
}

fn pos_2_string(from: LineColumn, to: LineColumn, codes: &mut String) {
    for _ in from.line..to.line {
        codes.push('\n');
    }
    let col = if from.line == to.line { from.column } else { 0 };
    for _ in col..to.column {
        codes.push(' ');
    }
}

fn group_2_string(group: Group, codes: &mut String) {
    let (open, close) = match group.delimiter() {
        Delimiter::Parenthesis => ('(', ')'),
        Delimiter::Brace => ('{', '}'),
        Delimiter::Bracket => ('[', ']'),
        _ => return,
    };

    codes.push(open);
    let tokens = group.stream();
    if !tokens.is_empty() {
        let tokens_beg = tokens.span().start();
        let tokens_end = tokens.span().end();
        pos_2_string(group.span_open().end(), tokens_beg, codes);
        token_2_string(tokens, codes);
        pos_2_string(tokens_end, group.span_close().start(), codes);
    }
    codes.push(close);
}
