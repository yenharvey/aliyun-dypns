//! 云通信号码认证服务 SDK
//!
//! 提供发送和验证短信验证码的功能
//!
//! # 示例
//!
//! ```no_run
//! use aliyun_dypns::{SmsClient, SendSmsRequest, CheckSmsRequest, CodeGenerationMode};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = SmsClient::new("your-access-key-id", "your-access-key-secret");
//!
//!     let send_request = SendSmsRequest::new("138****8000", "your-scheme-id")
//!         .code_generation_mode(CodeGenerationMode::Dynamic(5))
//!         .code_length(6)
//!         .valid_time(300);
//!
//!     let send_response = client.send_sms_verify_code(send_request).await?;
//!     println!("验证码已发送: {:?}", send_response);
//!
//!     let check_request = CheckSmsRequest::new("138****8000", "123456", "your-scheme-id");
//!     let check_response = client.check_sms_verify_code(check_request).await?;
//!     println!("验证结果: {:?}", check_response);
//!
//!     Ok(())
//! }
//! ```

mod client;
mod error;
mod signature;
mod types;

pub use client::SmsClient;
pub use error::{Result, SmsError};
pub use types::{
    CheckResultModel, CheckSmsRequest, CheckSmsResponse, CodeGenerationMode, SendSmsRequest,
    SendSmsResponse, SmsSignature, SmsTemplate, TemplateParams,
};
