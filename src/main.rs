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
use utils::{get_range_u32, get_str_between, input_key_to_continue, print_list, replace_password};
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
    wlt_task             打开交互界面
    wlt_task task        登录网络通，如果IP变化，发送邮件通知
    wlt_task set         设置一个计划任务，每5分钟（或者网络连接的时候）执行一次wlt_task task
    wlt_task unset       取消这个计划任务
    wlt_task query       查看计划任务状态";

fn main() -> anyhow::Result<()> {
    let mut args: Vec<String> = std::env::args().collect();
    let mut need_pause = false;
    if args.len() == 1 {
        need_pause = true;
        println!("请输入选择的数字并回车：");
        print_list(
            [
                "登录网络通，如果IP变化，发送邮件通知（wlt_task task）",
                "设置一个计划任务，每5分钟（或者网络连接的时候）执行一次wlt_task task（wlt_task set）",
                "取消这个计划任务（wlt_task unset）",
                "查看计划任务状态（wlt_task query）",
            ],
            1,
        );
        let select = get_range_u32(1, 4);
        match select {
            1 => args.push("task".to_owned()),
            2 => args.push("set".to_owned()),
            3 => args.push("unset".to_owned()),
            4 => args.push("query".to_owned()),
            _ => unreachable!(),
        }
    }

    if args.len() == 2 && args[1] == "task" {
        if let Err(e) = check_wlt() {
            let mut e = e.to_string();
            if let Ok(mut data) = Data::load() {
                if e.contains("operation timed out") {
                    data.timeout_count += 1;
                    data.save()?;
                    if data.timeout_count < 3 {
                        log_append("?");
                        return Ok(()); // timeout次数大于等于3才通知
                    }
                } else {
                    data.timeout_count = 0;
                    data.save()?;
                }
            }

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

    if need_pause {
        input_key_to_continue("", "按回车键退出...");
    }
    Ok(())
}
