# fresh

基于 reqwest 的“Forest/Retrofit 风格”声明式 HTTP 客户端。用 trait + 注解描述接口，宏生成具体调用代码。

## 快速开始

```bash
cargo run -p demo
```

示例接口定义（位于 demo/src/main.rs）：

```rust
#[fresh(base_url = "https://httpbin.org")]
trait Api {
    #[get("/get")]
    async fn search(&self, #[query] q: SearchQuery) -> fresh::Result<HttpBinGet>;

    #[post("/post")]
    async fn create_user(&self, #[json] body: CreateUser) -> fresh::Result<serde_json::Value>;

    #[get("/anything/{id}")]
    async fn anything(
        &self,
        #[path] id: u64,
        #[header("X-Trace-Id")] trace: String,
    ) -> fresh::Result<serde_json::Value>;
}
```

生成的客户端名为 `ApiClient`，可用 `new_default()`（使用 trait 上的 base_url）或 `with_base_url("...")` 构造。

## 已支持

- 方法：`#[get]` `#[post]` `#[put]` `#[delete]` `#[patch]`（路径模板支持 `{id}`）
- 参数：
    - `#[path]` 路径替换
    - `#[query]` 查询参数（支持 `serde::Serialize`）
    - `#[json]` JSON 请求体
    - `#[header("Name")]` 自定义请求头
- 返回：`Result<T, fresh::Error>`，自动按 JSON 反序列化为 `T`
- 连接复用、HTTP/2、gzip/br/deflate、rustls TLS

## 约束与后续计划

- 返回体默认按 JSON 解析（后续会支持 `text()`、`bytes()`、流式下载等返回策略）
- 同步/blocking 版本（可基于 `reqwest::blocking` 扩展）
- 中间件/重试/超时/日志注解（可结合 `reqwest-middleware` 或 tower）
- multipart、cookie jar、代理、客户端证书等（在运行时构造 `HttpClient` 即可，或增加注解封装）