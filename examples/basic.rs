use aliyun_dypns::{
    CheckSmsRequest, CodeGenerationMode, SendSmsRequest, SmsClient, SmsSignature, SmsTemplate,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let access_key_id =
        env::var("ALIYUN_ACCESS_KEY_ID").expect("ALIYUN_ACCESS_KEY_ID not found in .env");
    let access_key_secret =
        env::var("ALIYUN_ACCESS_KEY_SECRET").expect("ALIYUN_ACCESS_KEY_SECRET not found in .env");
    let test_mobile = env::var("TEST_MOBILE").expect("TEST_MOBILE not found in .env");

    let scheme = "your-scheme-id";
    let client = SmsClient::new(access_key_id, access_key_secret);

    println!("发送验证码到: {}", test_mobile);
    let send_request = SendSmsRequest::new(&test_mobile, scheme)
        .signature(SmsSignature::YunZhuVerifyPlatform)
        .template(SmsTemplate::LoginRegister)
        .code_generation_mode(CodeGenerationMode::Dynamic(5))
        .code_length(6)
        .valid_time(300);

    match client.send_sms_verify_code(send_request).await {
        Ok(response) => {
            println!("✓ 验证码发送成功!");
            if let Some(request_id) = response.request_id() {
                println!("  RequestId: {}", request_id);
            }
            if let Some(biz_id) = response.biz_id() {
                println!("  BizId: {}", biz_id);
            }
        }
        Err(e) => {
            eprintln!("✗ 发送失败: {}", e);
            return Err(e.into());
        }
    }

    println!("\n请输入收到的验证码:");
    let mut code = String::new();
    std::io::stdin().read_line(&mut code)?;
    let code = code.trim().to_owned();

    println!("\n验证验证码...");
    let check_request = CheckSmsRequest::new(&test_mobile, &code, scheme);

    match client.check_sms_verify_code(check_request).await {
        Ok(response) => {
            println!("✓ 验证成功!");
            if let Some(result) = response.verify_result() {
                println!("  结果: {}", result);
            }
        }
        Err(e) => {
            eprintln!("✗ 验证失败: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
