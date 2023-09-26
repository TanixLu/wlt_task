mod config;
mod email;
mod log;
mod task;
mod utils;
mod wlt;

use config::Config;
use email::send_email;
use log::{log, log_append};
use task::{query_task, set_task, unset_task};
use utils::{replace_password, AnyResult};
use wlt::{WltClient, WltPageType};

fn check_wlt() -> AnyResult<()> {
    let mut config = Config::load()?;
    let mut wlt_client = WltClient::new(
        &config.name,
        &config.password,
        &config.rn,
        config.type_,
        config.exp,
    )?;
    let wlt_page = wlt_client.access_page()?;
    let new_ip = wlt_page.search_ip()?;
    let new_ip = match wlt_page.page_type()? {
        WltPageType::LoginPage => {
            wlt_client.login(&new_ip)?;
            if wlt_client.get_rn() != &config.rn {
                log(&format!(
                    "old_rn: {}, new_rn: {}",
                    config.rn,
                    wlt_client.get_rn()
                ));
                config.rn = wlt_client.get_rn().to_owned();
                config.save()?;
            }
            let set_wlt_page = wlt_client.set_wlt()?;
            set_wlt_page.search_ip()?
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
    }
    log_append(".");
    Ok(())
}

const USAGE: &str = "usage:
wlt_task: login wlt and if ip changes, send an email
wlt_task set: set wlt_task as a scheduled task, which runs every 5 minutes
wlt_task unset: unset the scheduled task
wlt_task query: query status of the scheduled task";

fn main() -> AnyResult<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        match args[1].as_str() {
            subcommand @ ("set" | "unset" | "query") => {
                let output = match subcommand {
                    "set" => set_task(),
                    "unset" => unset_task(),
                    "query" => query_task(),
                    _ => unreachable!(),
                }?;
                println!("{}", output);
            }
            _ => println!("{}", USAGE),
        }
    } else {
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
    }

    Ok(())
}
