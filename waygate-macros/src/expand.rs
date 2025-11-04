mod request;
mod method;

/// 宏输入类型枚举
pub enum MacroForm {
    /// #[proc_macro_attribute]
    ///
    /// #[request(endpoint = "...", ...)]
    Attribute {
        attr: proc_macro2::TokenStream,
        item: proc_macro2::TokenStream,
    },
    /// #[proc_macro_derive]
    /// #[derive(...)]
    _Derive {
        item: proc_macro2::TokenStream,
    },
    /// #[proc_macro]
    /// 函数式宏
    _Function {
        item: proc_macro2::TokenStream,
    },
}

/// 具体使用的宏枚举
pub enum MacroKind {
    Request,
}

/// 宏调用信息结构体
pub struct MacroCall {
    pub kind: MacroKind,
    pub form: MacroForm,
}

impl MacroCall {
    pub fn new(kind: MacroKind, form: MacroForm) -> Self {
        Self { kind, form }
    }
}

/// 展开器 trait
pub trait Expander {
    fn expand(&self, call: MacroCall) -> syn::Result<proc_macro2::TokenStream>;
}

/// dispatch 宏展开
pub fn dispatch(call: MacroCall) -> proc_macro2::TokenStream {
    let expander: Box<dyn Expander> = match call.kind {
        MacroKind::Request => Box::new(request::WaygateExpander {}),
    };
    expander.expand(call).unwrap_or_else(|e| e.to_compile_error())
}
