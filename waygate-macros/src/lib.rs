use proc_macro::TokenStream;

mod http;
mod util;
mod expand;
mod parser;

/// trait 宏入口：`#[request(...)]`
/// 只在入口使用 `proc_macro::TokenStream`，内部统一用 `proc_macro2::TokenStream`
#[proc_macro_attribute]
pub fn request(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = expand::MacroForm::Attribute { attr: attr.into(), item: item.into() };
    let call = expand::MacroCall::new(expand::MacroKind::Request, input);
    expand::dispatch(call).into()
}