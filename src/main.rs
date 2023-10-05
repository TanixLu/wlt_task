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
        &config.网络通用户名,
        &config.网络通密码,
        config.网络通出口,
        config.网络通使用时限,
        &data.rn,
    )?;

    let wlt_page = wlt_client.access_page()?;
    let mut new_ip = wlt_page.search_ip()?;

    let need_set_wlt = match wlt_page.page_type()? {
        WltPageType::ControlPage => {
            let type_text = get_str_between(&wlt_page.text, "出口: ", "网出口")?;
            let type_ = type_text.as_bytes()[0] - b'1';
            if type_ == config.网络通出口 {
                false
            } else {
                log(format!("旧出口: {} 新出口: {}", type_, config.网络通出口));
                true
            }
        }
        WltPageType::LoginPage => {
            wlt_client.login(&new_ip)?;
            if wlt_client.get_rn() != data.rn {
                log(format!(
                    "旧rn: {} 新rn: {}",
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
        log(format!("旧IP: {} 新IP: {}", old_ip, new_ip));
        let body = config
            .邮件内容
            .replace("{旧IP}", &old_ip)
            .replace("{新IP}", &new_ip);
        send_email(
            &config.邮箱服务器,
            &config.邮箱用户名,
            &config.邮箱密码,
            &config.邮件发送列表,
            &config.邮件主题,
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
    wlt_task run         登录网络通，如果IP变化，发送邮件通知
    wlt_task set         设置一个计划任务，每5分钟（或者网络连接的时候）执行一次wlt_task run
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
                "登录网络通，如果IP变化，发送邮件通知（wlt_task run）",
                "设置一个计划任务，每5分钟（或者网络连接的时候）执行一次wlt_task run（wlt_task set）",
                "取消这个计划任务（wlt_task unset）",
                "查看计划任务状态（wlt_task query）",
            ],
            1,
        );
        let select = get_range_u32(1, 4);
        match select {
            1 => args.push("run".to_owned()),
            2 => args.push("set".to_owned()),
            3 => args.push("unset".to_owned()),
            4 => args.push("query".to_owned()),
            _ => unreachable!(),
        }
    }

    if args.len() == 2 && args[1] == "run" {
        if let Err(e) = check_wlt() {
            let mut e = e.to_string();
            if let Ok(mut data) = Data::load() {
                if e.contains("operation timed out") {
                    data.连续超时次数 += 1;
                    data.save()?;
                    if data.连续超时次数 < 3 {
                        log_append("?");
                        return Ok(()); // timeout次数大于等于3才通知
                    }
                } else {
                    data.连续超时次数 = 0;
                    data.save()?;
                }
            }

            if let Ok(config) = Config::load() {
                e = replace_password(e, config.网络通密码, "***");
                log(&e);
                send_email(
                    &config.邮箱服务器,
                    &config.邮箱用户名,
                    &config.邮箱密码,
                    &config.邮件发送列表,
                    "网络通任务出错",
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
