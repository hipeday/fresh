//! 返回体解码策略

pub trait Decoder<T> {
    type Error;
    fn decode(self) -> Result<T, Self::Error>;
}

// 预留：json/text/bytes/stream 解码器实现
// pub struct JsonDecoder(reqwest::Response);
// pub struct TextDecoder(reqwest::Response);
// pub struct BytesDecoder(reqwest::Response);