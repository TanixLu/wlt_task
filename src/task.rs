use std::process::{Command, Output};

use crate::utils::AnyResult;

const TASK_NAME: &str = "wlt_task";

fn output_string(mut output: Output) -> AnyResult<String> {
    let mut buf = output.stdout;
    buf.append(&mut output.stderr);
    Ok(String::from_utf8(buf)?)
}

pub fn query_task() -> AnyResult<String> {
    let output = Command::new("powershell")
        .arg("Get-ScheduledTask")
        .arg("-TaskName")
        .arg(TASK_NAME)
        .arg("-TaskPath")
        .arg("\\")
        .output()?;
    output_string(output)
}

pub fn set_task() -> AnyResult<String> {
    let action = format!(
        "New-ScheduledTaskAction -Execute {} -WorkingDirectory {}",
        std::env::current_exe()?.to_string_lossy(),
        std::env::current_dir()?.to_string_lossy()
    );
    const DESCRIPTION: &str = "Configure WLT and send notification emails when the IP changes";
    const SETTINGS: &str = "New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -StartWhenAvailable -DontStopIfGoingOnBatteries -RunOnlyIfNetworkAvailable";
    const TRIGGER: &str = "New-ScheduledTaskTrigger -Once -At 2000-01-01 00:00:00 -RepetitionInterval (New-TimeSpan -Minutes 5)";
    let output = Command::new("powershell")
        .arg("Register-ScheduledTask")
        .arg("-Force")
        .arg("-TaskName")
        .arg(TASK_NAME)
        .arg("-Action")
        .arg(action)
        .arg("-Description")
        .arg(DESCRIPTION)
        .arg("-Settings")
        .arg(SETTINGS)
        .arg("-Trigger")
        .arg(TRIGGER)
        .output()?;
    output_string(output)
}

pub fn unset_task() -> AnyResult<String> {
    let output = Command::new("powershell")
        .arg("Unregister-ScheduledTask")
        .arg("-TaskName")
        .arg(TASK_NAME)
        .arg("-TaskPath")
        .arg("\\")
        .output()?;
    output_string(output)
}
