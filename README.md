# Waygate - 声明式 HTTP 客户端

[![Crates.io](https://img.shields.io/crates/v/waygate.svg)](https://crates.io/crates/waygate)
[![Docs.rs](https://docs.rs/waygate/badge.svg)](https://docs.rs/waygate)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

基于 reqwest 的`Retrofit 风格`声明式 HTTP 客户端。用 trait + 注解描述接口，过程宏生成具体调用代码。

- 简单：用 `#[request(...)]` 标注 trait，一键生成 `XxxClient`
- 直观：方法上用 `#[get]` / `#[post]` 指定 HTTP 动作与路径
- 安全：编译期展开，零运行时反射
- 轻量：基于 reqwest，无侵入

## 安装与特性

工作区内已默认将 `waygate-macros` 作为可选依赖并通过特性启用。对外 crate 使用方式：

```toml
[dependencies]
waygate = { version = "0.1.0", features = ["macros"] } # macros 缺省已开启；可按需关闭/开启
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

若要禁用并按需开启：

```toml
[dependencies]
waygate = { version = "0.1.0", default-features = false, features = ["macros"] }
```

在本仓库中，`waygate` 已对宏进行根导出，可使用 `waygate::request`。

## 快速开始

- 定义请求与响应数据结构

```rust
use waygate::request;

#[allow(async_fn_in_trait)]
#[request(
  endpoint = "https://httpbin.org",
  headers(foo = "bar"),
  timeout = 10000,
  connect_timeout = 11000,
  read_timeout = 12000,
)]
pub trait waygateAttribute {

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
```

- 生成客户端并调用

```rust
use waygate_test::{
    SearchQuery,
    macros::{waygateAttribute, waygateAttributeClient},
};
use std::time::Duration;

#[tokio::test]
async fn test_search() {
    let client = waygateAttributeClient::new_default().unwrap();
    let response = client
        .search(
            SearchQuery {
                q: "test".into(),
                page: 1,
            },
            String::from("zhuzhuxia"),
            30,
            123,
            String::from("trace-xyz"),
        )
        .await
        .unwrap();
    println!("{}", serde_json::to_string(&response).unwrap());
    // {"url":"https://httpbin.org/anything/123?q=test&page=1&nickname=zhuzhuxia&age=30","args":{"age":"30","nickname":"zhuzhuxia","page":"1","q":"test"},"headers":{"Accept":"*/*","Accept-Encoding":"gzip, br, deflate","Foo":"bar,override-bar","Host":"httpbin.org","Token-Auth":"abcd1234","User-Agent":"waygate-client/0.1.0","X-Amzn-Trace-Id":"Root=1-69099f0f-1ee585000f72fc6337d86bc6","X-Trace-Id":"trace-xyz"}}
}
```

宏将生成 `ApiClient`，并注入构造方法：

- `ApiClient::with_endpoint("&str")`
- `ApiClient::new_default()` 使用 trait 上的 `endpoint` 与 `headers` 构造

## 运行示例与测试

运行示例：

```bash
cargo run --example hello_world
```

运行测试（含宏测试与实际访问 httpbin 的用例）：

```bash
cargo test -p waygate-test
```

## 运行时 API（摘）

`HttpClientOption` 提供 Builder 构造：

```rust
let opt = waygate::HttpClientOption::builder()
    .endpoint("https://httpbin.org")
    .header("user-agent", "waygate-client/0.1")
    .headers(vec![("x-token", "demo-token")])
    .build()?; // endpoint 不能为空
let client = waygate::HttpClient::new(opt)?;
```

注意：
- `build()` 要求 `endpoint` 必填；若希望提供默认端点，可在你自己的调用侧封装。
- 默认会附加 `User-Agent: waygate-client/{version}`。

## 关于请求头的大小写与字符集

- 键名：宏会将 `headers(user_agent = "...")` 等键名规范为短横线小写（`user-agent`）。
- 值：HTTP 协议规范推荐 ASCII。库在内部优先用 `HeaderValue::from_str`，若失败会回退用原始字节构造以兼容中文，但对端可能按 ISO-8859-1/ASCII 展示导致“乱码”。建议仅在必要时于头部放非 ASCII，或考虑将信息放入 body/query。

## 设计约束与建议

- 公开 trait 中使用 `async fn` 会触发编译器建议（`async_fn_in_trait`）。你可以：
  - 在 trait 上加 `#[allow(async_fn_in_trait)]`（仓库中测试已如此处理）
  - 或改为返回 `impl Future<Output = ...> + Send` 的签名（更稳健）
- `waygate` 根已导出：`HttpClient`、`HttpClientOption`、`HttpClientOptionBuilder`、`waygate::waygate`（受特性 `macros` 控制）。

## 许可证

本项目使用 [MIT License](LICENSE)。