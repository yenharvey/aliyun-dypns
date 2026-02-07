use thiserror::Error;

#[derive(Error, Debug)]
pub enum SmsError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("API error: {code} - {message}")]
    ApiError { code: String, message: String },

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Signature calculation failed: {0}")]
    SignatureError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Mobile number is illegal")]
    MobileNumberIllegal,

    #[error("The number has exceeded the limit for the day")]
    BusinessLimitControl,

    #[error("Check frequency failed - too many verification attempts")]
    FrequencyFail,

    #[error("Function not opened - please enable SMS authentication service")]
    FunctionNotOpened,

    #[error("Missing verify code")]
    MissingVerifyCode,

    #[error(
        "Custom verification code cannot be verified by Aliyun API - please implement your own verification logic"
    )]
    CustomCodeNotVerifiable,

    #[error("Scheme not found or invalid")]
    SchemeInvalid,

    #[error("Template parameter error: {0}")]
    TemplateParamError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl SmsError {
    /// 从阿里云 API 错误码创建具体的错误类型
    pub fn from_api_code(code: &str, message: String) -> Self {
        match code {
            "MOBILE_NUMBER_ILLEGAL" | "isv.MOBILE_NUMBER_ILLEGAL" => Self::MobileNumberIllegal,
            "BUSINESS_LIMIT_CONTROL" | "isv.BUSINESS_LIMIT_CONTROL" => Self::BusinessLimitControl,
            "FREQUENCY_FAIL" | "isv.FREQUENCY_FAIL" => Self::FrequencyFail,
            "FUNCTION_NOT_OPENED" | "isv.FUNCTION_NOT_OPENED" => Self::FunctionNotOpened,
            "MissingVerifyCode" | "isv.MissingVerifyCode" => Self::MissingVerifyCode,
            "INVALID_PARAMETERS" | "isv.INVALID_PARAMETERS" => Self::InvalidParameter(message),
            "ValidateFail" | "isv.ValidateFail" => Self::VerificationFailed(message),
            _ => Self::ApiError {
                code: code.to_string(),
                message,
            },
        }
    }
}

pub type Result<T> = std::result::Result<T, SmsError>;
