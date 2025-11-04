use waygate::request;

#[allow(async_fn_in_trait)]
#[request(
    endpoint = "https://httpbin.org",
    headers(foo = "bar"),
    timeout = 10000,
    connect_timeout = 11000,
    read_timeout = 12000,
)]
pub trait Api {

    #[get(
        path = "/get",
        headers(foo = "bar", token_auth = "abcd1234", foo = "override-bar"),
        timeout = 13000,
    )]
    async fn get(&self, #[query] q: crate::SearchQuery) -> waygate::Result<crate::HttpBinGet>;

    #[get(
        path = "/anything/{id}",
        headers(foo = "bar", token_auth = "abcd1234", foo = "override-bar"),
        timeout = 13000,
    )]
    async fn search(
        &self,
        #[query] q: crate::SearchQuery,
        #[query] nickname: String,
        #[query("age")] age: u32,
        #[path] id: u32,
        #[header("X-Trace-Id")] trace: String,
    ) -> waygate::Result<crate::HttpBinGet>;
}