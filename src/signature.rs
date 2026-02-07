use crate::error::{Result, SmsError};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::Utc;
use hmac::{Hmac, Mac};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use sha1::Sha1;
use std::collections::BTreeMap;
use std::sync::Arc;
use uuid::Uuid;

const FRAGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'+')
    .add(b'%')
    .add(b'&')
    .add(b'=')
    .add(b'/')
    .add(b'?')
    .add(b':')
    .add(b'@')
    .add(b'[')
    .add(b']')
    .add(b'{')
    .add(b'}')
    .add(b',');

#[derive(Clone)]
pub struct SignatureBuilder {
    access_key_id: Arc<str>,
    access_key_secret: Arc<str>,
}

impl SignatureBuilder {
    pub fn new(access_key_id: impl Into<String>, access_key_secret: impl Into<String>) -> Self {
        Self {
            access_key_id: access_key_id.into().into(),
            access_key_secret: access_key_secret.into().into(),
        }
    }

    /// 对字符串进行 URL 编码
    fn percent_encode<S: AsRef<str>>(s: S) -> String {
        utf8_percent_encode(s.as_ref(), FRAGMENT)
            .to_string()
            .replace("+", "%20")
            .replace("*", "%2A")
            .replace("%7E", "~")
    }

    /// 计算签名
    fn calculate_signature<M, Q>(
        &self,
        http_method: M,
        canonicalized_query_string: Q,
    ) -> Result<String>
    where
        M: AsRef<str>,
        Q: AsRef<str>,
    {
        let string_to_sign = format!(
            "{}&{}&{}",
            http_method.as_ref(),
            Self::percent_encode("/"),
            Self::percent_encode(canonicalized_query_string.as_ref())
        );

        type HmacSha1 = Hmac<Sha1>;
        let key = format!("{}&", self.access_key_secret);
        let mut mac = HmacSha1::new_from_slice(key.as_bytes())
            .map_err(|e| SmsError::SignatureError(e.to_string()))?;

        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize();
        Ok(STANDARD.encode(result.into_bytes()))
    }

    /// 构建请求参数（包含签名）
    pub fn build_params<A, V>(
        &self,
        action: A,
        version: V,
        business_params: BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>>
    where
        A: AsRef<str>,
        V: AsRef<str>,
    {
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let nonce = Uuid::new_v4().to_string();

        let mut params = business_params;

        params.insert("AccessKeyId".to_string(), self.access_key_id.to_string());
        params.insert("Action".to_string(), action.as_ref().to_string());
        params.insert("Format".to_string(), "JSON".to_string());
        params.insert("SignatureMethod".to_string(), "HMAC-SHA1".to_string());
        params.insert("SignatureNonce".to_string(), nonce);
        params.insert("SignatureVersion".to_string(), "1.0".to_string());
        params.insert("Timestamp".to_string(), timestamp);
        params.insert("Version".to_string(), version.as_ref().to_string());

        // 构建规范化查询字符串（用于签名计算）
        // 按照阿里云文档：对每个键和值分别编码，然后用 = 连接，最后用 & 连接
        let canonicalized_query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", Self::percent_encode(k), Self::percent_encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let signature = self.calculate_signature("GET", &canonicalized_query_string)?;

        params.insert("Signature".to_string(), signature);

        Ok(params)
    }
}
