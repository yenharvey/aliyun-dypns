# aliyun-dypns

云通信号码认证服务 (Phone Number Verification Service) Rust SDK

[![Crates.io](https://img.shields.io/crates/v/aliyun-dypns.svg)](https://crates.io/crates/aliyun-dypns)
[![Documentation](https://docs.rs/aliyun-dypns/badge.svg)](https://docs.rs/aliyun-dypns)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

基于阿里云云通信号码认证服务 API 的 Rust SDK，提供短信验证码发送和校验功能。

## 安装

```toml
[dependencies]
aliyun-dypns = "0.1"
```

## 快速开始

```rust
use aliyun_dypns::{
    SmsClient, SendSmsRequest, CheckSmsRequest,
    CodeGenerationMode, SmsSignature, SmsTemplate,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SmsClient::new("your-access-key-id", "your-access-key-secret");

    // 发送验证码（阿里云动态生成）
    let send_request = SendSmsRequest::new("138****8000", "your-scheme-id")
        .signature(SmsSignature::YunZhuVerifyPlatform)
        .template(SmsTemplate::LoginRegister)
        .code_generation_mode(CodeGenerationMode::Dynamic(5))
        .code_length(6)
        .valid_time(300);

    let response = client.send_sms_verify_code(send_request).await?;
    println!("发送成功: {:?}", response.biz_id());

    // 校验验证码
    let check_request = CheckSmsRequest::new("138****8000", "123456", "your-scheme-id");
    let check_response = client.check_sms_verify_code(check_request).await?;
    if check_response.is_pass() {
        println!("验证成功！");
    }

    Ok(())
}
```

### Builder API

`SendSmsRequest` 使用链式 builder 模式，所有可选字段都有默认值：

```rust
// 最简单的方式
let request = SendSmsRequest::new("138****8000", "your-scheme-id");

// 自定义选项
let request = SendSmsRequest::new("138****8000", "your-scheme-id")
    .code_length(6)
    .valid_time(300);
```

## 验证码生成模式

### 动态生成（推荐）

阿里云自动生成验证码，可通过 API 校验：

```rust
CodeGenerationMode::Dynamic(5) // 5分钟有效期
```

### 自定义验证码

使用自定义验证码（需自行实现校验逻辑）：

```rust
use aliyun_dypns::{SendSmsRequest, CodeGenerationMode, TemplateParams};

let send_request = SendSmsRequest::new("138****8000", "your-scheme-id")
    .code_generation_mode(CodeGenerationMode::Custom(
        TemplateParams::with_custom_code("123456", 5)
    ));
```

## 错误处理

```rust
use aliyun_dypns::SmsError;

match client.send_sms_verify_code(request).await {
    Ok(response) => println!("成功"),
    Err(SmsError::MobileNumberIllegal) => println!("手机号格式错误"),
    Err(SmsError::BusinessLimitControl) => println!("超过每日限额"),
    Err(SmsError::FrequencyFail) => println!("请求频率过高"),
    Err(e) => println!("其他错误: {}", e),
}
```

## 主要错误类型

| 错误类型               | 说明             |
| ---------------------- | ---------------- |
| `MobileNumberIllegal`  | 手机号格式错误   |
| `BusinessLimitControl` | 超过每日发送限额 |
| `FrequencyFail`        | 验证频率过高     |
| `FunctionNotOpened`    | 未开通服务       |
| `InvalidParameter`     | 参数错误         |
| `VerificationFailed`   | 验证失败         |

## 注意事项

- 自定义验证码无法通过阿里云 API 校验，需自行实现验证逻辑
- 签名和模板需要在阿里云控制台预先创建并审核通过
- 注意阿里云的发送频率和每日限额

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 相关链接

- [云通信号码认证服务文档](https://next.api.aliyun.com/document/Dypnsapi/2017-05-25/SendSmsVerifyCode)
- [GitHub 仓库](https://github.com/yenharvey/aliyun-dypns)