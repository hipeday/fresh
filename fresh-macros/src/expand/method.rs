use crate::parser::Cardinality;
use crate::{http::method::Method, parser::ParamKind, parser::ParamMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

#[derive(Clone)]
pub struct MethodCtx {
    pub sig_ident: syn::Ident,
    pub ok_ty: TokenStream,
    pub method: Method,
    pub endpoint: Option<String>,
    pub path: String,
    pub route_headers: Vec<(String, String)>, // 来自方法级 headers(...)
    pub timeout_ms: Option<u64>,              // 方法级整体超时
    pub params: Vec<ParamMeta>,               // 统一参数模型
}

pub struct MethodExpander {
    ctx: MethodCtx,
    body: TokenStream, // 方法体代码
}

impl MethodExpander {
    pub fn new(ctx: MethodCtx) -> Self {
        Self {
            ctx,
            body: TokenStream::new(),
        }
    }

    pub fn validate(self) -> syn::Result<Self> {
        if self.ctx.path.is_empty() {
            return Err(syn::Error::new(
                self.ctx.sig_ident.span(),
                "The 'path' attribute cannot be empty.",
            ));
        }
        Ok(self)
    }

    pub fn stage_init(mut self) -> Self {
        let MethodCtx { endpoint, path, .. } = &self.ctx;

        // 生成 __path 和 __url
        let path_lit = LitStr::new(path, self.ctx.sig_ident.span());
        let endpoint_stmt = if let Some(ep) = endpoint {
            let ep_lit = LitStr::new(ep, self.ctx.sig_ident.span());
            quote! {
                let __base = ::url::Url::parse(#ep_lit)
                    .expect("invalid endpoint in #[request(...)]");
                let __url = __base.join(&__path)
                    .expect("failed to join endpoint and path");
            }
        } else {
            // 假设 self.core.endpoint() -> Url
            quote! {
                let __url = self.core.endpoint().join(&__path)
                    .expect("failed to join endpoint and path");
            }
        };

        self.body.extend(quote! {
            // 初始 path 字符串
            let mut __path = #path_lit.to_string();
            #endpoint_stmt
        });
        self
    }

    pub fn stage_replace_path_params(mut self) -> Self {
        // 用 Path 参数替换 {name}
        for p in &self.ctx.params {
            if let ParamKind::Path = &p.kind {
                let ident = &p.ident;
                let key = p.ident.to_string();
                let key_lit = LitStr::new(&format!("{{{}}}", key), p.ident.span());
                self.body.extend(quote! {
                    __path = __path.replace(#key_lit, &::std::string::ToString::to_string(&#ident));
                });
            }
        }
        self
    }

    pub fn stage_request_builder(mut self) -> Self {
        let method_tokens = self.ctx.method.to_token();
        self.body.extend(quote! {
            let mut __req = self.core.client().request(#method_tokens, __url);
        });
        self
    }

    pub fn stage_apply_static_headers(mut self) -> Self {
        // 方法属性里的静态 headers(...)
        for (k, v) in &self.ctx.route_headers {
            let k_lit = LitStr::new(k, self.ctx.sig_ident.span());
            let v_lit = LitStr::new(v, self.ctx.sig_ident.span());
            self.body
                .extend(quote! { __req = __req.header(#k_lit, #v_lit); });
        }
        self
    }

    pub fn stage_apply_param_headers(mut self) -> Self {
        // 参数级 #[header] 支持
        for p in &self.ctx.params {
            if let ParamKind::Header { name } = &p.kind {
                let ident = &p.ident;
                // 未命名则用形参名
                let key = name
                    .clone()
                    .unwrap_or_else(|| LitStr::new(&p.ident.to_string(), p.ident.span()));
                self.body.extend(quote! {
                    __req = __req.header(#key, &#ident);
                });
            }
        }
        self
    }

    pub fn stage_apply_query(mut self) -> Self {
        use syn::Type;

        // 累积标量键值对，最后一次性调用 .query(&__query_vec)
        self.body.extend(quote! {
            let mut __query_vec: ::std::vec::Vec<(::std::borrow::Cow<'static, str>, ::std::string::String)> = ::std::vec::Vec::new();
        });

        for p in &self.ctx.params {
            if let ParamKind::Query { key } = &p.kind {
                let ident = &p.ident;
                // 未显式命名 => 使用形参名
                let key_lit: LitStr = key
                    .clone()
                    .unwrap_or_else(|| LitStr::new(&p.ident.to_string(), p.ident.span()));

                // 小工具：判断是否标量（String/str/数字/bool）
                fn is_scalar_ty(ty: &Type) -> bool {
                    match ty {
                        Type::Path(tp) => {
                            if let Some(seg) = tp.path.segments.last() {
                                match seg.ident.to_string().as_str() {
                                    "String" | "bool" |
                                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
                                    "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
                                    "f32" | "f64" => true,
                                    _ => false,
                                }
                            } else { false }
                        }
                        Type::Reference(r) => {
                            matches!(&*r.elem, Type::Path(tp) if tp.path.is_ident("str"))
                        }
                        _ => false,
                    }
                }

                // 取泛型内层类型（Option<T>/Vec<T>）
                fn first_generic_arg<'a>(ty: &'a Type) -> Option<&'a Type> {
                    if let Type::Path(tp) = ty {
                        if let Some(seg) = tp.path.segments.last() {
                            if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                                for arg in &args.args {
                                    if let syn::GenericArgument::Type(t) = arg {
                                        return Some(t);
                                    }
                                }
                            }
                        }
                    }
                    None
                }

                // 判断标量/复杂
                let (is_scalar_single, is_scalar_inner) = match (&p.ty, p.cardinality.clone()) {
                    (Some(ty), Cardinality::Single) => (is_scalar_ty(ty), true),
                    (Some(ty), Cardinality::Option) => (first_generic_arg(ty).map(is_scalar_ty).unwrap_or(false), false),
                    (Some(ty), Cardinality::Many)   => (first_generic_arg(ty).map(is_scalar_ty).unwrap_or(false), false),
                    _ => (true, true), // 无类型（如 self）不应出现到这里，按标量跳过
                };

                let has_explicit_key = key.is_some();

                match p.cardinality {
                    Cardinality::Single => {
                        if has_explicit_key {
                            // 显式命名：只允许标量
                            if is_scalar_single {
                                self.body.extend(quote! {
                                    __query_vec.push((::std::borrow::Cow::Borrowed(#key_lit), #ident.to_string()));
                                });
                            } else {
                                // 复杂类型 + 显式命名：给出友好错误
                                self.body.extend(quote! {
                                    compile_error!(concat!(
                                        "参数 `", stringify!(#ident),
                                        "` 使用了 #[query(\"…\")] 显式命名，但其类型为结构体/映射等复杂类型。\
                                        复杂类型请去掉名字，直接写 #[query] 以按字段展开为多个 query 参数。"
                                    ));
                                });
                            }
                        } else {
                            // 未命名：标量 -> (ident, val)；复杂类型 -> 直接 .query(&param) 展平字段
                            if is_scalar_single {
                                self.body.extend(quote! {
                                    __query_vec.push((::std::borrow::Cow::Borrowed(#key_lit), #ident.to_string()));
                                });
                            } else {
                                self.body.extend(quote! {
                                    __req = __req.query(&#ident);
                                });
                            }
                        }
                    }
                    Cardinality::Option => {
                        if has_explicit_key {
                            if is_scalar_inner {
                                self.body.extend(quote! {
                                    if let Some(ref __v) = #ident {
                                        __query_vec.push((::std::borrow::Cow::Borrowed(#key_lit), __v.to_string()));
                                    }
                                });
                            } else {
                                self.body.extend(quote! {
                                    compile_error!(concat!(
                                        "参数 `", stringify!(#ident),
                                        "` 是 Option<复杂类型> 且使用了 #[query(\"…\")]。\
                                        复杂类型请去掉名字，直接写 #[query]，我们会在 Some 时按字段展开；\
                                        例如：#[query] opt: Option<MyStruct>"
                                    ));
                                });
                            }
                        } else {
                            if is_scalar_inner {
                                self.body.extend(quote! {
                                    if let Some(ref __v) = #ident {
                                        __query_vec.push((::std::borrow::Cow::Borrowed(#key_lit), __v.to_string()));
                                    }
                                });
                            } else {
                                self.body.extend(quote! {
                                    if let Some(ref __v) = #ident {
                                        __req = __req.query(__v);
                                    }
                                });
                            }
                        }
                    }
                    Cardinality::Many => {
                        if has_explicit_key {
                            if is_scalar_inner {
                                self.body.extend(quote! {
                                    for __v in &#ident {
                                        __query_vec.push((::std::borrow::Cow::Borrowed(#key_lit), __v.to_string()));
                                    }
                                });
                            } else {
                                self.body.extend(quote! {
                                    compile_error!(concat!(
                                        "参数 `", stringify!(#ident),
                                        "` 是 Vec<复杂类型>/切片 且使用了 #[query(\"…\")]。\
                                        默认不支持将复杂元素用单一 key 重复提交（语义不明确）。\
                                        建议改为单个复杂对象 #[query]，或自行实现展平策略。"
                                    ));
                                });
                            }
                        } else {
                            if is_scalar_inner {
                                self.body.extend(quote! {
                                    for __v in &#ident {
                                        __query_vec.push((::std::borrow::Cow::Borrowed(#key_lit), __v.to_string()));
                                    }
                                });
                            } else {
                                self.body.extend(quote! {
                                    compile_error!(concat!(
                                        "参数 `", stringify!(#ident),
                                        "` 是 Vec<复杂类型>/切片 并使用了 #[query]。\
                                        默认不支持将多个复杂对象直接展开为 query 串（会产生 0[a]=… 这样的键）。\
                                        如确需支持，请改为单个复杂对象，或后续提供自定义展平选项。"
                                    ));
                                });
                            }
                        }
                    }
                }
            }
        }

        self.body.extend(quote! {
            if !__query_vec.is_empty() {
                __req = __req.query(&__query_vec);
            }
        });
        self
    }

    pub fn stage_apply_json(mut self) -> Self {
        // 允许最多一个 #[json] 参数（如果你允许多个，可以最后一次为准）
        for p in &self.ctx.params {
            if let ParamKind::Json = &p.kind {
                let ident = &p.ident;
                self.body.extend(quote! { __req = __req.json(&#ident); });
                break;
            }
        }
        self
    }

    pub fn stage_apply_timeout(mut self) -> Self {
        if let Some(ms) = self.ctx.timeout_ms {
            let ms_lit = ms;
            self.body.extend(quote! {
                __req = __req.timeout(::std::time::Duration::from_millis(#ms_lit as u64));
            });
        }
        self
    }

    pub fn stage_send_and_denser(mut self) -> Self {
        let ok_ty = &self.ctx.ok_ty;
        self.body.extend(quote! {
            let __resp = __req.send().await?;
            let __out = __resp.json::<#ok_ty>().await?;
            return ::core::result::Result::Ok(__out);
        });
        self
    }

    pub fn finish(self) -> TokenStream {
        let body = self.body;
        quote! {{ #body }}
    }
}
