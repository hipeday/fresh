use reqwest::Client;
use url::Url;

pub struct HttpClient {
    inner: Client,
    base_url: Url,
}

impl HttpClient {

    /// 创建一个新的 HttpClient 实例
    pub fn new(base_url: impl AsRef<str>) -> crate::error::Result<Self> {
        let inner = Client::builder()
            .user_agent(format!("fresh-client/{}", env!("CARGO_PKG_VERSION")))
            .build()?;
        let base_url = Url::parse(base_url.as_ref())?;
        Ok(Self { inner, base_url })
    }

    pub fn from_reqwest(inner: Client, base_url: impl AsRef<str>) -> crate::error::Result<Self> {
        let base_url = Url::parse(base_url.as_ref())?;
        Ok(Self { inner, base_url })
    }

    pub fn client(&self) -> &Client {
        &self.inner
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }
}

