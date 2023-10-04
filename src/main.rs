mod config;
mod data;
mod email;
mod log;
mod task;
mod utils;
mod wlt;

use config::Config;
use data::Data;
use email::send_email;
use log::{log, log_append};
use task::{query_task, set_task, unset_task};
use utils::{get_str_between, replace_password};
use wlt::{WltClient, WltPageType};

fn check_wlt() -> anyhow::Result<()> {
    let config = Config::load()?;
    let mut data = Data::load()?;

    let mut wlt_client = WltClient::new(
        &config.name,
        &config.password,
        config.type_,
        config.exp,
        &data.rn,
    )?;

    let wlt_page = wlt_client.access_page()?;
    let mut new_ip = wlt_page.search_ip()?;

    let need_set_wlt = match wlt_page.page_type()? {
        WltPageType::ControlPage => {
            let type_text = get_str_between(&wlt_page.text, "出口: ", "网出口")?;
            let type_ = type_text.as_bytes()[0] - b'1';
            if type_ == config.type_ {
                false
            } else {
                log(format!("old_type: {} new_type: {}", type_, config.type_));
                true
            }
        }
        WltPageType::LoginPage => {
            wlt_client.login(&new_ip)?;
            if wlt_client.get_rn() != data.rn {
                log(format!(
                    "old_rn: {} new_rn: {}",
                    data.rn,
                    wlt_client.get_rn()
                ));
                data.rn = wlt_client.get_rn().to_owned();
                data.save()?;
            }
            true
        }
    };

    if need_set_wlt {
        let set_wlt_page = wlt_client.set_wlt()?;
        new_ip = set_wlt_page.search_ip()?
    }

    let old_ip = data.ip.clone();
    if new_ip != old_ip {
        log(format!("old_ip: {} new_ip: {}", old_ip, new_ip));
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
        );
        data.ip = new_ip;
        data.save()?;
    }

    log_append(".");
    Ok(())
}

const USAGE: &str = "usage:
    wlt_task             Log in to WLT and send an email if the IP changes.
    wlt_task set         Set wlt_task as a scheduled task to run every 5 minutes.
    wlt_task unset       Unset the scheduled task.
    wlt_task query       Query the status of the scheduled task.";

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        if let Err(e) = check_wlt() {
            let mut e = e.to_string();
            if let Ok(config) = Config::load() {
                e = replace_password(e, config.password, "***");
                log(&e);
                send_email(
                    &config.email_server,
                    &config.email_username,
                    &config.email_password,
                    &config.email_to_list,
                    "WLT Task Error",
                    &e,
                );
            } else {
                log(&e);
            }
        }
    } else if args.len() == 2 && (args[1] == "set" || args[1] == "unset" || args[1] == "query") {
        let output = match args[1].as_str() {
            "set" => set_task(),
            "unset" => unset_task(),
            "query" => query_task(),
            _ => unreachable!(),
        }?;
        println!("{}", output);
    } else {
        println!("{}", USAGE);
    }

    Ok(())
}
