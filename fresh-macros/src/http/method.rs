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
            Method::GET => quote! { ::fresh::reqwest::Method::GET },
            Method::POST => quote! { ::fresh::reqwest::Method::POST },
            Method::PUT => quote! { ::fresh::reqwest::Method::PUT },
            Method::HEAD => quote! { ::fresh::reqwest::Method::HEAD },
            Method::OPTIONS => quote! { ::fresh::reqwest::Method::OPTIONS },
            Method::DELETE => quote! { ::fresh::reqwest::Method::DELETE },
            Method::PATCH => quote! { ::fresh::reqwest::Method::PATCH },
            Method::TRACE => quote! { ::fresh::reqwest::Method::TRACE },
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