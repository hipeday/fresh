use fresh::request;

#[allow(async_fn_in_trait)]
#[request(
    endpoint = "https://httpbin.org",
    headers(foo = "bar"),
    timeout = 10000,
    connect_timeout = 11000,
    read_timeout = 12000,
)]
pub trait FreshAttribute {

    #[get(
        path = "/get",
        headers(foo = "bar", token_auth = "abcd1234", foo = "override-bar"),
        timeout = 13000,
        connect_timeout = 14000,
        read_timeout = 15000,
    )]
    async fn search(&self, #[query] q: crate::SearchQuery) -> fresh::Result<crate::HttpBinGet>;
}