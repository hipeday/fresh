use serde::{Deserialize, Serialize};
use waygate::request;

#[derive(Debug, Serialize)]
struct SearchQuery {
    q: String,
    page: u32,
}

#[derive(Debug, Serialize)]
struct CreateUser {
    name: String,
}

#[derive(Debug, Deserialize)]
struct HttpBinGet {
    url: String,
    args: serde_json::Value,
    headers: serde_json::Value,
}

#[request(
    endpoint = "https://httpbin.org",
    headers(foo = "bar", user_agent = "waygate-test"),
    timeout = 10000,
    connect_timeout = 11000,
    read_timeout = 12000,
)]
trait Api {
    #[get(path = "/get")]
    async fn search(&self, #[query] q: SearchQuery) -> waygate::Result<HttpBinGet>;

    #[post(path = "/post")]
    async fn create_user(&self, #[json] body: CreateUser) -> waygate::Result<serde_json::Value>;

    #[get(path = "/anything/{id}")]
    async fn anything(
        &self,
        #[path] id: u64,
        #[header("X-Trace-Id")] trace: String,
    ) -> waygate::Result<serde_json::Value>;
}

#[tokio::main]
async fn main() -> waygate::Result<()> {
    let api = ApiClient::new_default()?;

    let out = api
        .search(SearchQuery { q: "rust".into(), page: 1 })
        .await?;
    println!("GET /get => url={}, args={}, header={}", out.url, out.args, out.headers);

    let created = api
        .create_user(CreateUser { name: "Ferris".into() })
        .await?;
    println!("POST /post => {}", created);

    let any = api.anything(42, "trace-123".into()).await?;
    println!("GET /anything/42 => {}", any);

    Ok(())
}