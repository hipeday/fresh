//! 参数注解解析。
//!
//! 支持的参数标注：
//! - `#[path]`   路径占位符替换（如 `/users/{id}`）
//! - `#[query]`  序列化为查询参数（要求实现 `serde::Serialize`）
//! - `#[json]`   序列化为 JSON 请求体
//! - `#[header("Name")]` 自定义请求头

use syn::{FnArg, Pat};

/// 参数标注类型
#[derive(Clone)]
pub enum ParamKind {
    Path,
    Query,
    Json,
    Header(String),
    Other,
}

/// 参数标注元信息
#[derive(Clone)]
pub struct ParamMeta {
    // 参数名
    pub ident: syn::Ident,
    // 参数类型（可能为空，例如 `self` 参数）
    pub ty: Option<syn::Type>,
    // 标注类型
    pub kind: ParamKind,
}

/// 解析参数标注属性
pub fn parse_param_attrs(inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) -> Vec<ParamMeta> {
    let mut params = Vec::new();

    for input in inputs {
        if let FnArg::Typed(pt) = input {
            let ident = match &*pt.pat {
                Pat::Ident(pi) => pi.ident.clone(),
                _ => panic!("Unsupported parameter pattern"),
            };
            let mut kind = ParamKind::Other;
            let mut header_name: Option<String> = None;
            for a in &pt.attrs {
                let name = a.path().get_ident().map(|i| i.to_string()).unwrap_or_default();
                match name.as_str() {
                    "path" => kind = ParamKind::Path,
                    "query" => kind = ParamKind::Query,
                    "json" => kind = ParamKind::Json,
                    "header" => {
                        if let Some(syn::Lit::Str(s)) = a.parse_args().ok() {
                            header_name = Some(s.value());
                        }
                    }
                    _ => {}
                }
            }
            if let Some(h) = header_name {
                kind = ParamKind::Header(h);
            }
            params.push(ParamMeta { ident, ty: Some((*pt.ty).clone()), kind });
        }
    }

    params
}