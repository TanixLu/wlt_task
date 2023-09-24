mod config;
mod email;
mod log;
mod utils;
mod wlt;

use config::Config;
use email::send_email;
use log::log;
use utils::AnyResult;
use wlt::{WltClient, WltPage};

fn main() -> AnyResult<()> {
    let main_try = || -> AnyResult<()> {
        let config = Config::load()?;
        let wlt_client = WltClient::new(
            &config.name,
            &config.password,
            config.type_,
            config.exp,
            &config.cookie,
        )?;

        let wlt_page = wlt_client.access_page()?;
        let old_ip = wlt_page.search_ip()?;

        let final_wlt_page = match wlt_page {
            WltPage::LoginPage(_) => {
                wlt_client.login(&old_ip)?;
                let new_wlt_page = wlt_client.set_wlt()?;
                match new_wlt_page {
                    WltPage::LoginPage(page) => {
                        return Err(format!(
                            "开通网络失败：{} {} {}",
                            page.url, page.status, page.text
                        )
                        .into())
                    }
                    WltPage::ControlPage(_) => new_wlt_page,
                }
            }
            WltPage::ControlPage(_) => wlt_page,
        };

        let new_ip = final_wlt_page.search_ip()?;
        if new_ip != old_ip {
            let body = config
                .email_body
                .replace("{old_ip}", &old_ip)
                .replace("{new_ip}", &new_ip);
            send_email(
                &config.email_server,
                &config.email_username,
                &config.email_password,
                &config.email_to_list,
                &config.email_subject,
                &body,
            )?;
        }

        let new_cookie = wlt_client.get_cookie_string()?;
        if new_cookie != config.cookie {
            let mut config = config;
            config.cookie = new_cookie;
            config.save()?;
        }

        Ok(())
    };

    if let Err(e) = main_try() {
        let e = e.to_string();
        log(&e)?;
        let config = Config::load()?;
        send_email(
            &config.email_server,
            &config.email_username,
            &config.email_password,
            &config.email_to_list,
            "WLT Task Error",
            &e,
        )?;
    }

    Ok(())
}
