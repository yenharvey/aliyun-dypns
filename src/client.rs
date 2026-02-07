use crate::error::{Result, SmsError};
use crate::signature::SignatureBuilder;
use crate::types::*;
use reqwest::Client as HttpClient;
use std::collections::BTreeMap;

const API_ENDPOINT: &str = "https://dypnsapi.aliyuncs.com/";
const API_VERSION: &str = "2017-05-25";

#[derive(Clone)]
pub struct SmsClient {
    http_client: HttpClient,
    signature_builder: SignatureBuilder,
}

impl SmsClient {
    /// 创建新的短信客户端
    pub fn new(access_key_id: impl Into<String>, access_key_secret: impl Into<String>) -> Self {
        Self {
            http_client: HttpClient::new(),
            signature_builder: SignatureBuilder::new(
                access_key_id.into(),
                access_key_secret.into(),
            ),
        }
    }

    fn build_query_string(params: &BTreeMap<String, String>) -> String {
        params
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    percent_encoding::utf8_percent_encode(k, percent_encoding::NON_ALPHANUMERIC),
                    percent_encoding::utf8_percent_encode(v, percent_encoding::NON_ALPHANUMERIC)
                )
            })
            .collect::<Vec<_>>()
            .join("&")
    }

    /// 发送短信验证码
    pub async fn send_sms_verify_code(&self, request: SendSmsRequest) -> Result<SendSmsResponse> {
        if !request.mobile.chars().all(|c| c.is_ascii_digit()) {
            return Err(SmsError::MobileNumberIllegal);
        }

        if request.mobile.len() != 11 {
            return Err(SmsError::InvalidParameter("手机号必须是 11 位".to_string()));
        }

        let mut params = BTreeMap::new();
        params.insert("PhoneNumber".to_string(), request.mobile);
        params.insert("Scheme".to_string(), request.scheme);

        if let Some(signature) = request.signature {
            params.insert("SignName".to_string(), signature.as_str().to_string());
        }

        if let Some(template) = request.template {
            params.insert("TemplateCode".to_string(), template.code().to_string());
        }

        let template_params = request.code_generation_mode.into_template_params();
        params.insert(
            "TemplateParam".to_string(),
            template_params
                .to_json()
                .map_err(|e| SmsError::InvalidParameter(format!("模板参数序列化失败: {}", e)))?,
        );

        if let Some(code_length) = request.code_length {
            params.insert("CodeLength".to_string(), code_length.to_string());
        }

        if let Some(valid_time) = request.valid_time {
            params.insert("ValidTime".to_string(), valid_time.to_string());
        }

        let params =
            self.signature_builder
                .build_params("SendSmsVerifyCode", API_VERSION, params)?;

        let query_string = Self::build_query_string(&params);

        let url = format!("{}?{}", API_ENDPOINT, query_string);

        let response = self.http_client.get(&url).send().await?;
        let response_text = response.text().await?;

        if let Ok(error_resp) = serde_json::from_str::<ErrorResponse>(&response_text)
            && (!error_resp.success || error_resp.code != "OK")
        {
            return Err(SmsError::from_api_code(
                &error_resp.code,
                error_resp.message,
            ));
        }

        let response_data: SendSmsResponse = serde_json::from_str(&response_text)
            .map_err(|e| SmsError::InvalidResponse(format!("{}: {}", e, response_text)))?;

        if response_data.code != "OK" || !response_data.success {
            return Err(SmsError::from_api_code(
                &response_data.code,
                response_data.message,
            ));
        }

        Ok(response_data)
    }

    /// 核验短信验证码
    pub async fn check_sms_verify_code(
        &self,
        request: CheckSmsRequest,
    ) -> Result<CheckSmsResponse> {
        if !request.mobile.chars().all(|c| c.is_ascii_digit()) {
            return Err(SmsError::MobileNumberIllegal);
        }

        if request.mobile.len() != 11 {
            return Err(SmsError::InvalidParameter("手机号必须是 11 位".to_string()));
        }

        if request.code.is_empty() {
            return Err(SmsError::InvalidParameter("验证码不能为空".to_string()));
        }

        let mut params = BTreeMap::new();
        params.insert("PhoneNumber".to_string(), request.mobile);
        params.insert("VerifyCode".to_string(), request.code);
        params.insert("Scheme".to_string(), request.scheme);

        let params =
            self.signature_builder
                .build_params("CheckSmsVerifyCode", API_VERSION, params)?;

        let query_string = Self::build_query_string(&params);

        let url = format!("{}?{}", API_ENDPOINT, query_string);

        let response = self.http_client.get(&url).send().await?;
        let response_text = response.text().await?;

        if let Ok(error_resp) = serde_json::from_str::<ErrorResponse>(&response_text)
            && (!error_resp.success || error_resp.code != "OK")
        {
            return Err(SmsError::from_api_code(
                &error_resp.code,
                error_resp.message,
            ));
        }

        let response_data: CheckSmsResponse = serde_json::from_str(&response_text)
            .map_err(|e| SmsError::InvalidResponse(format!("{}: {}", e, response_text)))?;

        if response_data.code != "OK" || !response_data.success {
            return Err(SmsError::from_api_code(
                &response_data.code,
                response_data.message,
            ));
        }

        if !response_data.is_pass() {
            return Err(SmsError::VerificationFailed(format!(
                "验证失败: {}",
                response_data.verify_result().unwrap_or("UNKNOWN")
            )));
        }

        Ok(response_data)
    }
}
