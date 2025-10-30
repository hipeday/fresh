// 解析器

mod fresh_attribute;

/// 解析器 trait
pub trait Parser<I> {
    type Output;

    fn parse(input: I) -> syn::Result<Self::Output>;
}