//! trait 级属性解析。
//!
//! 目前支持：`base_url = "https://api.example.com"`，用于生成 `XxxClient::new_default()`。

use syn::parse::Parser as _;
use syn::LitStr;

/// 从属性参数 TokenStream 中解析 `base_url = "..."`。
pub fn parse_base_url(args_ts: &proc_macro2::TokenStream) -> syn::Result<Option<String>> {
    let mut out: Option<String> = None;

    // 使用 syn v2 的 nested meta 解析器
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("base_url") {
            // 读取等号右侧的字面量字符串
            let lit: LitStr = meta.value()?.parse()?;
            out = Some(lit.value());
        } else {
            // 未识别的键，当前忽略（也可以返回 meta.error(...) 让编译期报错）
        }
        Ok(())
    });

    parser.parse2(args_ts.clone())?;
    Ok(out)
}