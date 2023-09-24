use encoding_rs::GBK;
use reqwest::{
    blocking::{Client, Response},
    StatusCode,
};
use scraper::{Html, Selector};

use crate::utils::{find_str_between, AnyResult};

const WLT_URL: &str = "http://202.38.64.59/cgi-bin/ip";

struct Page {
    url: String,
    status: StatusCode,
    text: String,
}

enum WltPage {
    LoginPage(Page),
    ControlPage(Page),
}

impl Page {
    fn check_ok(&self) -> AnyResult<()> {
        if self.status == StatusCode::OK {
            Ok(())
        } else {
            Err(format!("访问页面出错：{} {} {}", self.url, self.status, self.text).into())
        }
    }
}

impl WltPage {
    fn new(url: impl Into<String>, resp: Response) -> AnyResult<Self> {
        let status = resp.status();
        let text = resp.text_with_charset("GBK")?;
        let page = Page {
            url: url.into(),
            status,
            text,
        };
        page.check_ok()?;
        if page.text.contains("网络通账号登录") {
            Ok(WltPage::LoginPage(page))
        } else if page.text.contains("访问文献资源建议使用1出口") {
            Ok(WltPage::ControlPage(page))
        } else {
            Err(format!("未知页面：{} {} {}", WLT_URL, status, page.text).into())
        }
    }

    fn search_ip(&self) -> AnyResult<String> {
        match self {
            WltPage::LoginPage(page) => {
                let html = Html::parse_document(&page.text);
                let ip_selector = Selector::parse("body > htm > form > table > tbody > tr:nth-child(2) > td > p > table > tbody > tr:nth-child(1) > td:nth-child(2)").map_err(|e| e.to_string())?;
                match html.select(&ip_selector).next() {
                    Some(element) => Ok(element
                        .text()
                        .next()
                        .ok_or("在登录页面没有找到ip对应元素")?
                        .trim()
                        .to_owned()),
                    None => Err("在登录页面没有找到ip".into()),
                }
            }
            WltPage::ControlPage(page) => {
                let html = Html::parse_document(&page.text);
                let ip_selector = Selector::parse(
                    "body > htm > p:nth-child(11) > table > tbody > tr > td:nth-child(1)",
                )
                .map_err(|e| e.to_string())?;
                match html.select(&ip_selector).next() {
                    Some(element) => {
                        let text = element
                            .text()
                            .next()
                            .ok_or("在控制页面没有找到ip对应元素")?
                            .trim()
                            .to_owned();
                        Ok(find_str_between(&text, "当前IP地址", "状态")?.to_owned())
                    }
                    None => Err("在控制页面没有找到ip".into()),
                }
            }
        }
    }
}

struct WltClient {
    client: Client,
    cookie_store: std::sync::Arc<reqwest_cookie_store::CookieStoreMutex>,
    name: String,
    password: String,
    type_: u8,
    exp: u32,
}

impl WltClient {
    fn new(name: &str, password: &str, type_: u8, exp: u32, cookie: &str) -> AnyResult<Self> {
        let cookie_store = reqwest_cookie_store::CookieStore::load_json(cookie.as_bytes())?;
        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);
        let client = reqwest::blocking::Client::builder()
            .cookie_provider(cookie_store.clone())
            .build()?;
        Ok(Self {
            client,
            cookie_store,
            name: name.to_owned(),
            password: password.to_owned(),
            type_,
            exp,
        })
    }

    fn cookie_string(&self) -> AnyResult<String> {
        let mut buf = Vec::new();
        self.cookie_store
            .lock()
            .map_err(|e| e.to_string())?
            .save_json(&mut buf)?;
        Ok(String::from_utf8(buf)?)
    }

    fn access_page(&self) -> AnyResult<WltPage> {
        let resp = self.client.get(WLT_URL).send()?;
        WltPage::new(WLT_URL, resp)
    }

    fn login(&self, ip: &str) -> AnyResult<WltPage> {
        let go = GBK.encode("登录账户").0;
        let go = &urlencoding::encode_binary(&go);
        let login_form = [
            ("cmd", "login"),
            ("url", "URL"),
            ("ip", ip),
            ("name", &self.name),
            ("password", &self.password),
            ("savepass", "on"),
            ("go", go),
        ];
        let resp = self.client.post(WLT_URL).form(&login_form).send()?;
        if resp.status() == StatusCode::OK {
            WltPage::new(WLT_URL, resp)
        } else {
            Err(format!(
                "登录账户失败：{} {} {}",
                WLT_URL,
                resp.status(),
                resp.text_with_charset("GBK")?
            )
            .into())
        }
    }

    fn set_network(&self) -> AnyResult<WltPage> {
        let go = GBK.encode("开通网络").0;
        let go = &urlencoding::encode_binary(&go);
        let url = format!(
            "{}?cmd=set&url=URL&type={}&exp={}&go=+{}+",
            WLT_URL, self.type_, self.exp, go,
        );
        let resp = self.client.get(&url).send()?;
        if resp.status() == StatusCode::OK {
            WltPage::new(url, resp)
        } else {
            Err(format!(
                "开通网络失败：{} {} {}",
                url,
                resp.status(),
                resp.text_with_charset("GBK")?
            )
            .into())
        }
    }
}
