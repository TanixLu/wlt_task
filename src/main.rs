mod config;
mod email;
mod log;
mod utils;
mod wlt;

use config::Config;
use email::send_email;
use log::{log, log_append};
use utils::AnyResult;
use wlt::{WltClient, WltPageType};

fn main() -> AnyResult<()> {
    let try_main = || -> AnyResult<()> {
        let mut config = Config::load()?;
        let mut wlt_client = WltClient::new(
            &config.name,
            &config.password,
            &config.rd,
            config.type_,
            config.exp,
        )?;

        let wlt_page = wlt_client.access_page()?;
        let new_ip = wlt_page.search_ip()?;
        let new_ip = match wlt_page.page_type()? {
            WltPageType::LoginPage => {
                wlt_client.login(&new_ip)?;
                let set_wlt_page = wlt_client.set_wlt()?;
                match set_wlt_page.page_type()? {
                    WltPageType::LoginPage => {
                        return Err(format!(
                            "开通网络失败\nurl: {}\ncookie: {}\ntext: {}",
                            set_wlt_page.url,
                            wlt_client.get_cookie(),
                            set_wlt_page.text
                        )
                        .into())
                    }
                    WltPageType::ControlPage => set_wlt_page.search_ip()?,
                }
            }
            WltPageType::ControlPage => new_ip,
        };

        let old_ip = config.ip.clone();
        if new_ip != old_ip {
            config.ip = new_ip.clone();
            config.save()?;
            let body = config
                .email_body
                .replace("{old_ip}", &old_ip)
                .replace("{new_ip}", &new_ip);
            log(&format!("old_ip: {}, new_ip: {}", old_ip, new_ip));
            send_email(
                &config.email_server,
                &config.email_username,
                &config.email_password,
                &config.email_to_list,
                &config.email_subject,
                &body,
            );
        } else {
            log_append(".");
        }

        if wlt_client.get_rd() != &config.rd {
            log(&format!(
                "rd变更\nold_rd: {}\nnew_rd: {}",
                config.rd,
                wlt_client.get_rd()
            ));
            config.rd = wlt_client.get_rd().to_owned();
            config.save()?;
        }

        Ok(())
    };

    if let Err(e) = try_main() {
        let e = e.to_string();
        log(&e);
        let config = Config::load()?;
        send_email(
            &config.email_server,
            &config.email_username,
            &config.email_password,
            &config.email_to_list,
            "WLT Task Error",
            &e,
        );
    }

    Ok(())
}
