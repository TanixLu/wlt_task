use std::time::Duration;

use encoding_rs::GBK;
use reqwest::{
    blocking::{Client, Response},
    header::{COOKIE, SET_COOKIE},
    StatusCode,
};

use crate::utils::{get_str_between, AnyResult};

const WLT_URL: &str = "http://202.38.64.59/cgi-bin/ip";

pub struct WltPage {
    pub url: String,
    pub status: StatusCode,
    pub text: String,
}

pub enum WltPageType {
    LoginPage,
    ControlPage,
}

impl WltPage {
    fn new(url: impl Into<String>, resp: Response) -> AnyResult<Self> {
        let url = url.into();
        let status = resp.status();
        let text = resp.text_with_charset("GBK")?;
        Ok(Self { url, status, text })
    }

    pub fn check_ok(&self) -> bool {
        self.status == StatusCode::OK
    }

    pub fn search_ip(&self) -> AnyResult<String> {
        let ip = match self.page_type()? {
            WltPageType::LoginPage => get_str_between(&self.text, "name=ip value=", ">"),
            WltPageType::ControlPage => get_str_between(&self.text, "当前IP地址", "状态"),
        }?
        .to_owned();
        Ok(ip)
    }

    pub fn page_type(&self) -> AnyResult<WltPageType> {
        if self.text.contains("网络通账号登录") {
            Ok(WltPageType::LoginPage)
        } else if self.text.contains("访问文献资源建议使用1出口") {
            Ok(WltPageType::ControlPage)
        } else {
            Err(format!("未知类型页面\nurl: {}\ntext: {}", self.url, self.text).into())
        }
    }
}

pub struct WltClient {
    client: Client,
    name: String,
    password: String,
    type_: u8,
    exp: u32,
    rn: String,
}

impl WltClient {
    pub fn new(name: &str, password: &str, type_: u8, exp: u32, rn: &str) -> AnyResult<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .no_proxy()
            .build()?;
        Ok(Self {
            client,
            name: name.to_owned(),
            password: password.to_owned(),
            type_,
            exp,
            rn: rn.to_owned(),
        })
    }

    pub fn get_cookie(&self) -> String {
        let password = urlencoding::encode(&self.password);
        let mut cookie = format!("name={}; password={}", self.name, password);
        if !self.rn.is_empty() {
            cookie.push_str("; rn=");
            cookie.push_str(&self.rn);
        }
        cookie
    }

    pub fn get_rn(&self) -> &str {
        &self.rn
    }

    fn update_rn(&mut self, resp: &Response) -> AnyResult<()> {
        if let Some(set_cookie) = resp.headers().get(SET_COOKIE) {
            let set_cookie = set_cookie.to_str()?;
            if let Some(rn) = set_cookie.strip_prefix("rn=") {
                self.rn = rn.to_owned();
            }
        }
        Ok(())
    }

    pub fn access_page(&mut self) -> AnyResult<WltPage> {
        let resp = self
            .client
            .get(WLT_URL)
            .header(COOKIE, self.get_cookie())
            .send()?;
        let wlt_page = WltPage::new(WLT_URL, resp)?;
        if wlt_page.check_ok() {
            Ok(wlt_page)
        } else {
            Err(format!(
                "访问网络通页面失败\nurl: {}\nstatus: {}\ntext: {}",
                WLT_URL, wlt_page.status, wlt_page.text
            )
            .into())
        }
    }

    pub fn login(&mut self, ip: &str) -> AnyResult<WltPage> {
        if self.name.is_empty() {
            return Err("输入的用户名为空".into());
        } else if self.password.is_empty() {
            return Err("输入的密码为空".into());
        }

        let name = self.name.to_owned();
        let password = self.password.to_owned();
        let go = GBK.encode("登录账户").0;
        let go = &urlencoding::encode_binary(&go);
        let login_form = [
            ("cmd", "login"),
            ("url", "URL"),
            ("ip", ip),
            ("name", &name),
            ("password", &password),
            ("savepass", "on"),
            ("go", go),
        ];
        let resp = self
            .client
            .post(WLT_URL)
            .form(&login_form)
            .header(COOKIE, self.get_cookie())
            .send()?;
        self.update_rn(&resp)?;
        let wlt_page = WltPage::new(WLT_URL, resp)?;
        for err_str in ["用户名不存在", "用户名或密码错误"] {
            if wlt_page.text.contains(err_str) {
                return Err(err_str.into());
            }
        }
        if wlt_page.status != StatusCode::OK {
            Err(format!(
                "登录账户失败\nurl: {}\nform: {:?}\nstatus: {}\ntext: {}",
                WLT_URL, login_form, wlt_page.status, wlt_page.text
            )
            .into())
        } else {
            Ok(wlt_page)
        }
    }

    pub fn set_wlt(&mut self) -> AnyResult<WltPage> {
        let go = GBK.encode("开通网络").0;
        let go = &urlencoding::encode_binary(&go);
        let url = format!(
            "{}?cmd=set&url=URL&type={}&exp={}&go=+{}+",
            WLT_URL, self.type_, self.exp, go,
        );
        let resp = self
            .client
            .get(&url)
            .header(COOKIE, self.get_cookie())
            .send()?;
        let wlt_page = WltPage::new(&url, resp)?;
        if wlt_page.text.contains("信息：网络设置成功") {
            Ok(wlt_page)
        } else {
            Err(format!(
                "开通网络失败\nurl: {}\ncookies: {}\nstatus: {}\ntext: {}",
                url,
                self.get_cookie(),
                wlt_page.status,
                wlt_page.text
            )
            .into())
        }
    }
}
