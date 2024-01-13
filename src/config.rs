use serde::{Deserialize, Serialize};

use crate::utils::{str_decrypt, substr_encrypt};

const CONFIG_PATH: &str = "config.toml";
const CONFIG_COMMENT: &str = r#"
# 网络通用户名：网络通用户名
# 网络通密码：网络通密码，运行程序后，这个密码会被加密
# 网络通出口：
#   0 教育网出口(国际,仅用教育网访问,适合看文献)
#   1 电信网出口(国际,到教育网走教育网)
#   2 联通网出口(国际,到教育网走教育网)
#   3 电信网出口2(国际,到教育网免费地址走教育网)
#   4 联通网出口2(国际,到教育网免费地址走教育网)
#   5 电信网出口3(国际,默认电信,其他分流)
#   6 联通网出口3(国际,默认联通,其他分流)
#   7 教育网出口2(国际,默认教育网,其他分流)
#   8 移动网出口(国际,无P2P或带宽限制)
# 网络通使用时限：
#   0     永久
#   3600  1小时
#   14400 4小时
#   39600 11小时
#   50400 14小时
# 邮箱服务器：如smtp.qq.com
# 邮箱用户名：如10000@qq.com
# 邮箱密码：一般是SMTP授权码，如f0123456789abcdef，运行程序后，这个密码会被加密
# 邮件发送列表：可以填自己的邮箱，如["10000@qq.com", "10000@mail.ustc.edu.cn"]，留空则禁用邮件功能
# 邮件主题：也即邮件标题
# 邮件内容：其中的{旧IP}会被替换为旧的ip，{新IP}会被替换为新的ip，{新IPV6}会被替换为新的ipv6
"#;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub 网络通用户名: String,
    pub 网络通密码: String,
    pub 网络通出口: u8,
    pub 网络通使用时限: u32,
    pub 邮箱服务器: String,
    pub 邮箱用户名: String,
    pub 邮箱密码: String,
    pub 邮件发送列表: Vec<String>,
    pub 邮件主题: String,
    pub 邮件内容: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            网络通用户名: String::new(),
            网络通密码: String::new(),
            网络通出口: 8,
            网络通使用时限: 0,
            邮箱服务器: "smtp.qq.com".to_string(),
            邮箱用户名: "10000@qq.com".to_string(),
            邮箱密码: "f0123456789abcdef".to_string(),
            邮件发送列表: Vec::new(),
            邮件主题: "网络通IP变化通知".to_string(),
            邮件内容: "旧IP: {旧IP}\n新IP: {新IP}\n新IPV6: {新IPV6}\n".to_string(),
        }
    }
}

impl Config {
    fn save(&self) -> anyhow::Result<()> {
        let config_string = toml::to_string_pretty(self)?;
        let config_string = substr_encrypt(config_string, &self.网络通密码)?;
        let config_string = substr_encrypt(config_string, &self.邮箱密码)?;
        let content = format!("{}\n{}", config_string, CONFIG_COMMENT);
        std::fs::write(CONFIG_PATH, content)?;
        Ok(())
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = std::path::Path::new(CONFIG_PATH);
        if !path.exists() || path.metadata().unwrap().len() == 0 {
            let config = Config::default();
            config.save()?;
            Ok(config)
        } else {
            let content = std::fs::read_to_string(CONFIG_PATH)?;
            let mut config = toml::from_str::<Config>(&content)?;
            let mut need_save_to_encrypt = false;
            if let Ok(plain_password) = str_decrypt(&config.网络通密码) {
                config.网络通密码 = plain_password;
            } else if !config.网络通密码.is_empty() {
                need_save_to_encrypt = true;
            }
            if let Ok(plain_email_password) = str_decrypt(&config.邮箱密码) {
                config.邮箱密码 = plain_email_password;
            } else if !config.邮箱密码.is_empty() {
                need_save_to_encrypt = true;
            }
            if need_save_to_encrypt {
                config.save()?;
            }

            Ok(config)
        }
    }
}
