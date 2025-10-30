//! 方法注解解析。
//!
//! 支持的注解：`#[get("/path")]`、`#[post("/path")]`、`#[put]`、`#[delete]`、`#[patch]`
//! 产出：HTTP 方法名与路径字面量。

use syn::{Attribute, Lit};

/// 解析方法注解，返回 (http_method, path_literal)。
pub fn parse_method_attr(attrs: &[Attribute]) -> Option<(String, proc_macro2::TokenStream)> {
    for a in attrs {
        if let Some(ident) = a.path().get_ident() {
            let name = ident.to_string().to_lowercase();
            if matches!(name.as_str(), "get" | "post" | "put" | "delete" | "patch") {
                let meta = a.parse_args::<Lit>().ok()?;
                if let Lit::Str(s) = meta {
                    let path = s.value();
                    return Some((name, quote::quote! { #path }));
                }
            }
        }
    }
    None
}