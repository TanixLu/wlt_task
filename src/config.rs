use serde::{Deserialize, Serialize};

use crate::log::log;
use crate::utils::AnyResult;

const CONFIG_PATH: &str = "config.toml";
const CONFIG_COMMENT: &str = "
# name：网络通用户名
# password：网络通密码
# type：出口选择
#   0 教育网出口(国际,仅用教育网访问,适合看文献)
#   1 电信网出口(国际,到教育网走教育网)
#   2 联通网出口(国际,到教育网走教育网)
#   3 电信网出口2(国际,到教育网免费地址走教育网)
#   4 联通网出口2(国际,到教育网免费地址走教育网)
#   5 电信网出口3(国际,默认电信,其他分流)
#   6 联通网出口3(国际,默认联通,其他分流)
#   7 教育网出口2(国际,默认教育网,其他分流)
#   8 移动网出口(国际,无P2P或带宽限制)
# exp：使用时限
#   0     永久
#   3600  1小时
#   14400 4小时
#   39600 11小时
#   50400 14小时
# ip：用于记录之前的ip地址，当ip地址变动时，会自动发送邮件通知，这一项不要手动更改
# rn：Cookie中的一个字段，这一项不要手动更改
# email_server：邮件服务器地址
# email_username：邮箱
# email_password：邮箱密码（SMTP授权码）
# email_to_list：若为空，则发给自己
# email_subject：邮件主题（标题）
# email_body：邮件内容，其中的{old_ip}会被替换为旧的ip，{new_ip}会被替换为新的ip
";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub name: String,
    pub password: String,
    #[serde(rename = "type")]
    pub type_: u8,
    pub exp: u32,
    pub ip: String,
    pub rn: String,
    pub email_server: String,
    pub email_username: String,
    pub email_password: String,
    pub email_to_list: Vec<String>,
    pub email_subject: String,
    pub email_body: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: String::new(),
            password: String::new(),
            type_: 8,
            exp: 0,
            ip: String::new(),
            rn: String::new(),
            email_server: "smtp.qq.com".to_string(),
            email_username: "10000@qq.com".to_string(),
            email_password: "f0123456789abcdef".to_string(),
            email_to_list: Vec::new(),
            email_subject: "WLT IP Address Change Notification".to_string(),
            email_body: "Old IP Address: {old_ip}\nNew IP Address: {new_ip}\n".to_string(),
        }
    }
}

impl Config {
    pub fn save(&self) -> AnyResult<()> {
        let config_string = toml::to_string_pretty(self)?;
        let content = format!("{}\n{}", config_string, CONFIG_COMMENT);
        std::fs::write(CONFIG_PATH, content)?;
        Ok(())
    }

    pub fn load() -> AnyResult<Self> {
        let config = if let Ok(config) = std::fs::read_to_string(CONFIG_PATH) {
            toml::from_str::<Config>(&config)?
        } else {
            log("配置文件不存在，创建默认配置文件");
            let config = Config::default();
            config.save()?;
            std::process::exit(0);
        };
        Ok(config)
    }
}
