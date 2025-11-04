use super::MacroCall;
use crate::{
    parser::{
        FreshAttributeParser,
        MethodMetaParser,
        Parser
    },
    expand::method::{MethodCtx, MethodExpander}
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemTrait, TraitItem};

pub struct FreshExpander;

impl super::Expander for FreshExpander {
    fn expand(&self, call: MacroCall) -> syn::Result<TokenStream> {
        match call.form {
            super::MacroForm::Attribute { attr, item } => {
                // 解析宏属性
                let attributes = FreshAttributeParser::parse(&attr)?;

                // 展开宏
                let mut trait_item: ItemTrait = syn::parse2(item.clone())?;

                let trait_ident = trait_item.ident.clone();
                let client_ident = format_ident!("{}Client", trait_ident);

                // 收集方法元信息（剥离前）
                let methods = MethodMetaParser::parse(&trait_item)?;

                // 剥离自定义宏 避免“未知属性”错误
                strip_custom_attrs_in_trait(&mut trait_item);

                // 展开每个方法
                let mut method_impls = Vec::new();
                for m in &methods {
                    method_impls.push(expand_method_impl(&m)?);
                }

                // 构造函数
                let mut ctor_extra = quote! {
                    pub fn with_endpoint(endpoint: &str) -> ::fresh::Result<Self> {
                        Ok(Self { core: ::fresh::HttpClient::with_endpoint(endpoint)? })
                    }
                };

                // 生成 .endpoint(...) 语句
                let endpoint_stmt = if let Some(endpoint) = &attributes.endpoint {
                    quote! { .endpoint(#endpoint) }
                } else {
                    quote! {}
                };

                // 生成 .headers(vec![...])
                let headers_pairs = attributes
                    .headers
                    .into_iter()
                    .map(|(k, v)| {
                        // "_" -> "-"
                        let k = k.replace('_', "-").to_ascii_lowercase();
                        let k = syn::LitStr::new(&k, proc_macro2::Span::call_site());
                        let v = syn::LitStr::new(&v, proc_macro2::Span::call_site());
                        quote! { (::std::string::String::from(#k), ::std::string::String::from(#v)) }
                    })
                    .collect::<Vec<_>>();

                let headers_stmt = if headers_pairs.is_empty() {
                    quote! {}
                } else {
                    quote! { .headers(vec![ #(#headers_pairs),* ]) }
                };

                // 生成 timeout 语句
                let timeout_stmt = if let Some(timeout_ms) = attributes.timeout {
                    let timeout_duration =
                        quote! { ::std::time::Duration::from_millis(#timeout_ms) };
                    quote! { .timeout(#timeout_duration) }
                } else {
                    quote! {}
                };

                // 生成 connect_timeout 语句
                let connect_timeout_stmt =
                    if let Some(connect_timeout_ms) = attributes.connect_timeout {
                        let connect_timeout_duration =
                            quote! { ::std::time::Duration::from_millis(#connect_timeout_ms) };
                        quote! { .connect_timeout(#connect_timeout_duration) }
                    } else {
                        quote! {}
                    };

                // 生成 read_timeout 语句
                let read_timeout_stmt = if let Some(read_timeout_ms) = attributes.read_timeout
                {
                    let read_timeout_duration =
                        quote! { ::std::time::Duration::from_millis(#read_timeout_ms) };
                    quote! { .read_timeout(#read_timeout_duration) }
                } else {
                    quote! {}
                };

                // 附加 new_default 构造函数
                ctor_extra = quote! {
                    #ctor_extra
                    pub fn new_default() -> ::fresh::Result<Self> {
                        let option = ::fresh::HttpClientOption::builder()
                            #endpoint_stmt
                            #headers_stmt
                            #timeout_stmt
                            #connect_timeout_stmt
                            #read_timeout_stmt
                            .build()
                            .map_err(|e| ::fresh::Error::InvalidArgument(format!("Build HttpClientOption failed: {}", e)))?;

                        Ok(Self { core: ::fresh::HttpClient::new(option)? })
                    }
                };

                let expanded = quote! {
                    #trait_item

                    pub struct #client_ident {
                        pub core: ::fresh::HttpClient,
                    }

                    impl #client_ident {
                        pub fn new(core: ::fresh::HttpClient) -> Self { Self { core } }
                        #ctor_extra
                    }

                    impl #trait_ident for #client_ident {
                        #(#method_impls)*
                    }
                };

                Ok(expanded)
            }
            _ => Err(syn::Error::new_spanned(
                TokenStream::new(),
                "Unsupported macro form for FreshExpander",
            )),
        }
    }
}

fn strip_custom_attrs_in_trait(trait_item: &mut ItemTrait) {
    for item in &mut trait_item.items {
        if let TraitItem::Fn(m) = item {
            // 方法级：去掉 get/post/put/delete/patch
            m.attrs.retain(|a| {
                let Some(id) = a.path().get_ident() else {
                    return true;
                };
                let n = id.to_string();
                !matches!(n.as_str(), "get" | "post" | "put" | "delete" | "patch")
            });
            // 参数级：去掉 path/query/json/header
            for input in &mut m.sig.inputs {
                if let FnArg::Typed(pt) = input {
                    pt.attrs.retain(|a| {
                        let Some(id) = a.path().get_ident() else {
                            return true;
                        };
                        let n = id.to_string();
                        !matches!(n.as_str(), "path" | "query" | "json" | "header")
                    });
                }
            }
        }
    }
}

fn expand_method_impl(meta: &crate::parser::MethodMeta) -> syn::Result<TokenStream> {
    // 将 MethodMeta 映射到 MethodCtx（补齐默认值/校验）
    let route = meta.route.clone();
    let method = route.method.ok_or_else(|| syn::Error::new(meta.sig_ident.span(), "缺少 HTTP 方法"))?;
    let path = route.path.clone().ok_or_else(|| syn::Error::new(meta.sig_ident.span(), "缺少 path"))?;

    let sig_ident = meta.sig_ident.clone(); // 方法签名

    let ctx = MethodCtx {
        sig_ident,
        ok_ty: meta.ok_ty.clone(),
        method,
        endpoint: None, // trait 级别的可传入
        path,
        route_headers: route.headers.clone(),
        timeout_ms: route.timeout,
        params: meta.params.clone(), // 统一参数模型
    };

    let body = MethodExpander::new(ctx)
        .validate()?
        .stage_init()
        .stage_replace_path_params()
        .stage_request_builder()
        .stage_apply_static_headers()
        .stage_apply_param_headers()
        .stage_apply_query()
        .stage_apply_json()
        .stage_apply_timeout()
        .stage_send_and_denser()
        .finish();

    // 用参数元信息组装 impl 方法的形参列表（跳过 self 等无类型参数）
    let mut impl_params = Vec::new();
    for p in &meta.params {
        if let Some(ty) = &p.ty {
            let id = &p.ident;
            impl_params.push(quote! { #id: #ty });
        }
    }

    let ident = &meta.sig_ident;
    let ok_ty = &meta.ok_ty;

    // 返回完整的方法定义
    Ok(quote! {
        async fn #ident(&self, #(#impl_params),*) -> ::fresh::Result<#ok_ty> {
            #body
        }
    })

}
