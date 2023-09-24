use std::io::Write;

use chrono::Local;
use encoding_rs::GBK;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use reqwest::StatusCode;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

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
# cookie：保存Cookie，这一项不要手动更改
# email_server：邮件服务器地址
# email_username：邮箱
# email_password：邮箱密码
# email_to_list：若为空，则发给自己
# email_subject：邮件主题（标题）
# email_body：邮件内容，其中的{old_ip}会被替换为旧的ip，{new_ip}会被替换为新的ip
";
const LOG_PATH: &str = "log.txt";

const WLT_URL: &str = "http://202.38.64.59/cgi-bin/ip";

type AnyResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    name: String,
    password: String,
    #[serde(rename = "type")]
    type_: u8,
    exp: u32,
    ip: String,
    cookie: String,
    email_server: String,
    email_username: String,
    email_password: String,
    email_to_list: Vec<String>,
    email_subject: String,
    email_body: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: String::new(),
            password: String::new(),
            type_: 8,
            exp: 0,
            ip: String::new(),
            cookie: String::new(),
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
    fn save(&self) -> AnyResult<()> {
        let config_string = toml::to_string_pretty(self)?;
        let content = format!("{}\n{}", config_string, CONFIG_COMMENT);
        std::fs::write(CONFIG_PATH, content)?;
        Ok(())
    }

    fn load() -> AnyResult<Self> {
        let config = if let Ok(config) = std::fs::read_to_string(CONFIG_PATH) {
            toml::from_str::<Config>(&config)?
        } else {
            log("配置文件不存在，创建默认配置文件")?;
            let config = Config::default();
            config.save()?;
            config
        };
        Ok(config)
    }

    fn send_email(&self, old_ip: &str, new_ip: &str) -> AnyResult<()> {
        let creds = Credentials::new(
            self.email_username.to_owned(),
            self.email_password.to_owned(),
        );

        let mailer = SmtpTransport::relay(&self.email_server)?
            .credentials(creds)
            .build();

        let body = self
            .email_body
            .replace("{old_id}", old_ip)
            .replace("{new_ip}", new_ip);

        let mut email = Message::builder().from(self.email_username.parse()?);
        if self.email_to_list.is_empty() {
            email = email.to(self.email_username.parse()?);
        } else {
            for mailbox_string in self.email_to_list.iter() {
                email = email.to(mailbox_string.parse()?);
            }
        }
        let email = email
            .subject(self.email_subject.to_owned())
            .header(ContentType::TEXT_PLAIN)
            .body(body)?;

        mailer.send(&email)?;

        Ok(())
    }

    /// 返回(是否登录, ip地址)
    fn check_wlt(&self) -> AnyResult<(bool, String)> {
        let cookie_store = reqwest_cookie_store::CookieStore::load_json(self.cookie.as_bytes())?;
        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);
        let client = reqwest::blocking::Client::builder()
            .cookie_provider(cookie_store)
            .build()?;
        let resp = client.get(WLT_URL).send()?;
        let text = resp.text_with_charset("GBK")?;
        let ip = if text.contains("网络通账号登录") {
            // 先找一个ip
            let html = Html::parse_document(&text);
            let ip_selector = Selector::parse("body > htm > form > table > tbody > tr:nth-child(2) > td > p > table > tbody > tr:nth-child(1) > td:nth-child(2)").unwrap();
            let ip = match html.select(&ip_selector).next() {
                Some(ip) => ip.text().next().unwrap().trim(),
                None => panic!("在登录页面没有找到ip"),
            };

            // 登录界面，需要使用client进行登录请求
            let go = &urlencoding::encode_binary(&GBK.encode("登录账户").0);
            let login_form = [
                ("cmd", "login"),
                ("url", "URL"),
                ("ip", ip),
                ("name", &self.name),
                ("password", &self.password),
                ("savepass", "on"),
                ("go", go),
            ];
            let resp = client.post(WLT_URL).form(&login_form).send()?;
            if resp.status() != StatusCode::OK {
                let err = format!(
                    "登录失败 {} {}",
                    resp.status(),
                    resp.text_with_charset("GBK")?
                );
                panic!("{}", err);
            };

            // 然后再进行开通网络请求
            let go = &urlencoding::encode_binary(&GBK.encode("开通网络").0);
            let set_url = format!(
                "{}?cmd=set&url=URL&type={}&exp={}&go=+{}+",
                WLT_URL, self.type_, self.exp, go,
            );

            // 从开通请求的里面返回ip
        } else {
        };

        // 将client的cookie存进去
    }
}

fn log(msg: impl AsRef<str>) -> AnyResult<()> {
    let msg = format!(
        "{}: {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        msg.as_ref()
    );
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)?;
    file.write_all(msg.as_bytes())?;
    Ok(())
}

fn main() -> AnyResult<()> {
    let url = format!("http://202.38.64.59/cgi-bin/ip");
    let resp = reqwest::blocking::get(url)?;

    let main_steps = || -> AnyResult<()> {
        let config = Config::load()?;

        Ok(())
    };

    if let Err(e) = main_steps() {
        log(e.to_string())?;
    }

    Ok(())
}
