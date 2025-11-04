use std::str::FromStr;
use quote::quote;

/// HTTP 请求方法枚举
#[derive(Debug, Clone, Copy)]
pub enum Method {
    GET,
    POST,
    PUT,
    HEAD,
    OPTIONS,
    DELETE,
    PATCH,
    TRACE,
}

impl Method {
    pub fn to_token(&self) -> proc_macro2::TokenStream {
        match self {
            Method::GET => quote! { ::waygate::reqwest::Method::GET },
            Method::POST => quote! { ::waygate::reqwest::Method::POST },
            Method::PUT => quote! { ::waygate::reqwest::Method::PUT },
            Method::HEAD => quote! { ::waygate::reqwest::Method::HEAD },
            Method::OPTIONS => quote! { ::waygate::reqwest::Method::OPTIONS },
            Method::DELETE => quote! { ::waygate::reqwest::Method::DELETE },
            Method::PATCH => quote! { ::waygate::reqwest::Method::PATCH },
            Method::TRACE => quote! { ::waygate::reqwest::Method::TRACE },
        }
    }
}

impl FromStr for Method {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "HEAD" => Ok(Method::HEAD),
            "OPTIONS" => Ok(Method::OPTIONS),
            "DELETE" => Ok(Method::DELETE),
            "PATCH" => Ok(Method::PATCH),
            "TRACE" => Ok(Method::TRACE),
            _ => Err(format!("Unsupported HTTP method: {}", s)),
        }
    }
}

impl From<String> for Method {
    fn from(value: String) -> Self {
        Method::from_str(&value).unwrap_or(Method::GET)
    }
}