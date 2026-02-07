use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;

/// 验证码生成模式
#[derive(Debug, Clone, PartialEq)]
pub enum CodeGenerationMode {
    /// 阿里云动态生成（推荐）- 可以使用 CheckSmsVerifyCode API 校验
    /// 参数：有效期（分钟）
    Dynamic(u32),

    /// 自定义验证码 - 需要自行实现验证逻辑，阿里云无法校验
    Custom(TemplateParams),
}

impl CodeGenerationMode {
    /// 转换为模板参数（消费所有权）
    pub fn into_template_params(self) -> TemplateParams {
        match self {
            Self::Dynamic(valid_minutes) => TemplateParams::with_dynamic_code(valid_minutes),
            Self::Custom(params) => params,
        }
    }
}

impl Default for CodeGenerationMode {
    fn default() -> Self {
        Self::Dynamic(5)
    }
}

/// 阿里云短信签名
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SmsSignature {
    /// 云渚科技验证平台
    #[default]
    YunZhuVerifyPlatform,

    /// 云渚科技验证服务
    YunZhuVerifyService,

    /// 速通互联验证码
    SuTongVerifyCode,

    /// 速通互联验证平台
    SuTongVerifyPlatform,

    /// 速通互联验证服务
    SuTongVerifyService,
}

impl SmsSignature {
    /// 获取签名名称
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::YunZhuVerifyPlatform => "云渚科技验证平台",
            Self::YunZhuVerifyService => "云渚科技验证服务",
            Self::SuTongVerifyCode => "速通互联验证码",
            Self::SuTongVerifyPlatform => "速通互联验证平台",
            Self::SuTongVerifyService => "速通互联验证服务",
        }
    }
}

impl fmt::Display for SmsSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 短信模板参数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateParams {
    /// 验证码
    #[serde(
        serialize_with = "serialize_cow_str",
        deserialize_with = "deserialize_cow_str"
    )]
    pub code: Cow<'static, str>,

    /// 有效期（分钟）
    pub min: u32,
}

fn serialize_cow_str<S>(cow: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(cow)
}

fn deserialize_cow_str<'de, D>(deserializer: D) -> Result<Cow<'static, str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Cow::Owned(s))
}

impl TemplateParams {
    /// 创建新的模板参数
    pub fn new(code: impl Into<String>, valid_minutes: u32) -> Self {
        Self {
            code: Cow::Owned(code.into()),
            min: valid_minutes,
        }
    }

    /// 创建使用动态生成验证码的模板参数
    /// 阿里云会自动生成验证码并可以进行校验
    pub fn with_dynamic_code(valid_minutes: u32) -> Self {
        Self {
            code: Cow::Borrowed("##code##"),
            min: valid_minutes,
        }
    }

    /// 创建使用自定义验证码的模板参数
    /// 注意：自定义验证码无法通过阿里云 CheckSmsVerifyCode API 进行校验
    pub fn with_custom_code(code: impl Into<String>, valid_minutes: u32) -> Self {
        Self {
            code: Cow::Owned(code.into()),
            min: valid_minutes,
        }
    }

    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// 短信模板
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SmsTemplate {
    /// 登录/注册模板
    #[default]
    LoginRegister,

    /// 修改绑定手机号模板
    ModifyPhone,

    /// 重置密码模板
    ResetPassword,

    /// 绑定新手机号模板
    BindNewPhone,

    /// 验证绑定手机号模板
    VerifyBindPhone,
}

impl SmsTemplate {
    /// 获取模板 CODE
    pub fn code(&self) -> &'static str {
        match self {
            Self::LoginRegister => "100001",
            Self::ModifyPhone => "100002",
            Self::ResetPassword => "100003",
            Self::BindNewPhone => "100004",
            Self::VerifyBindPhone => "100005",
        }
    }
}

impl fmt::Display for SmsTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendSmsRequest {
    /// 手机号码
    pub mobile: String,

    /// 验证码方案 ID
    pub scheme: String,

    /// 短信签名（可选，默认使用云渚科技验证平台）
    #[serde(skip)]
    pub signature: Option<SmsSignature>,

    /// 短信模板（可选，默认使用登录/注册模板）
    #[serde(skip)]
    pub template: Option<SmsTemplate>,

    /// 验证码生成模式（包含模板参数）
    #[serde(skip)]
    pub code_generation_mode: CodeGenerationMode,

    /// 验证码长度（可选，默认 4-6 位）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_length: Option<u8>,

    /// 有效期（秒，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_time: Option<u32>,
}

impl SendSmsRequest {
    pub fn new(mobile: impl Into<String>, scheme: impl Into<String>) -> Self {
        Self {
            mobile: mobile.into(),
            scheme: scheme.into(),
            signature: Some(SmsSignature::default()),
            template: Some(SmsTemplate::default()),
            code_generation_mode: CodeGenerationMode::default(),
            code_length: None,
            valid_time: None,
        }
    }

    pub fn signature(mut self, signature: SmsSignature) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn template(mut self, template: SmsTemplate) -> Self {
        self.template = Some(template);
        self
    }

    pub fn code_generation_mode(mut self, mode: CodeGenerationMode) -> Self {
        self.code_generation_mode = mode;
        self
    }

    pub fn code_length(mut self, length: u8) -> Self {
        self.code_length = Some(length);
        self
    }

    pub fn valid_time(mut self, seconds: u32) -> Self {
        self.valid_time = Some(seconds);
        self
    }
}

/// 发送短信响应数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendSmsModel {
    pub request_id: String,

    pub biz_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendSmsResponse {
    pub code: String,

    pub message: String,

    #[serde(default)]
    pub success: bool,

    #[serde(default)]
    pub model: Option<SendSmsModel>,
}

impl SendSmsResponse {
    /// 获取 RequestId（从 Model 中提取）
    pub fn request_id(&self) -> Option<&str> {
        self.model.as_ref().map(|m| m.request_id.as_str())
    }

    /// 获取 BizId（从 Model 中提取）
    pub fn biz_id(&self) -> Option<&str> {
        self.model.as_ref().map(|m| m.biz_id.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckSmsRequest {
    /// 手机号码
    pub mobile: String,

    /// 验证码
    pub code: String,

    /// 验证码方案 ID
    pub scheme: String,
}

impl CheckSmsRequest {
    pub fn new(
        mobile: impl Into<String>,
        code: impl Into<String>,
        scheme: impl Into<String>,
    ) -> Self {
        Self {
            mobile: mobile.into(),
            code: code.into(),
            scheme: scheme.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CheckResultModel {
    /// 验证结果：PASS-通过，UNKNOWN-未知，NOT_PASS-不通过
    pub verify_result: String,
}

/// 核验短信响应模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CheckSmsModel {
    pub verify_result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CheckSmsResponse {
    pub code: String,

    pub message: String,

    #[serde(default)]
    pub success: bool,

    #[serde(default)]
    pub model: Option<CheckSmsModel>,
}

impl CheckSmsResponse {
    /// 获取验证结果
    pub fn verify_result(&self) -> Option<&str> {
        self.model.as_ref().map(|m| m.verify_result.as_str())
    }

    /// 判断是否验证通过
    pub fn is_pass(&self) -> bool {
        self.verify_result() == Some("PASS")
    }
}

/// 阿里云 API 错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorResponse {
    pub code: String,

    pub message: String,

    #[serde(default)]
    pub success: bool,
}
