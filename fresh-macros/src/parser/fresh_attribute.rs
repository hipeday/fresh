use proc_macro2::TokenStream;

// 接口级宏上的属性解析器
pub struct FreshAttributeParser;

impl FreshAttributeParser {
    pub fn new() -> Self {
        FreshAttributeParser {}
    }
}

/// 接口级解析属性
pub struct FreshAttributes {
    pub base_url: Option<String>,
}

impl crate::parser::Parser<TokenStream> for FreshAttributeParser {
    type Output = FreshAttributes;

    fn parse(input: TokenStream) -> syn::Result<Self::Output> {
        let base_url = crate::attr::parse_base_url(&input)?;

        Ok(FreshAttributes { base_url })
    }
}