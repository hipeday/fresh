// 解析器

mod request;

pub use request::{RequestParser, MethodMetaParser, MethodMeta, ParamKind, ParamMeta, Cardinality};

/// 解析器 trait
pub trait Parser<I> {
    type Output;

    fn parse(input: &I) -> syn::Result<Self::Output>;
}