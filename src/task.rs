use std::{
    path::PathBuf,
    process::{Command, Output},
};

use crate::utils::AnyResult;

const TASK_NAME: &str = "wlt_task";
const VBS_NAME: &str = "wlt_task.vbs";

fn output_string(mut output: Output) -> AnyResult<String> {
    let mut buf = output.stdout;
    buf.append(&mut output.stderr);
    Ok(String::from_utf8(buf)?)
}

fn make_task_vbs_file() -> AnyResult<()> {
    let current_exe = std::env::current_exe()?;
    let contents = format!(
        r#"Set wShell = CreateObject("WScript.Shell")
wShell.Run "cmd /c {}", 0
"#,
        current_exe.to_string_lossy()
    );
    std::fs::write(VBS_NAME, contents)?;

    Ok(())
}

pub fn set_task() -> AnyResult<String> {
    make_task_vbs_file()?;
    let wscript_path = PathBuf::new()
        .join(std::env::var("WINDIR")?)
        .join("System32")
        .join("wscript.exe");
    let current_dir = std::env::current_dir()?;
    let vbs_path = current_dir.join(VBS_NAME);
    let action = format!(
        "(New-ScheduledTaskAction -Execute {} -WorkingDirectory {} -Argument {})",
        wscript_path.to_string_lossy(),
        current_dir.to_string_lossy(),
        vbs_path.to_string_lossy()
    );
    const DESCRIPTION: &str = r#""Configure WLT and send notification emails when the IP changes""#;
    const SETTINGS: &str = "(New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -StartWhenAvailable -DontStopIfGoingOnBatteries -RunOnlyIfNetworkAvailable)";
    const TRIGGER: &str = r#"(New-ScheduledTaskTrigger -Once -At "2000-01-01 00:00:00" -RepetitionInterval (New-TimeSpan -Minutes 5))"#;
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
        .arg("-Confirm:$false")
        .output()?;
    output_string(output)
}

pub fn query_task() -> AnyResult<String> {
    let output = Command::new("powershell")
        .arg("Get-ScheduledTaskInfo")
        .arg("-TaskName")
        .arg(TASK_NAME)
        .arg("-TaskPath")
        .arg("\\")
        .arg("-Verbose")
        .output()?;
    output_string(output)
}

#[test]
fn t() {
    dbg!(std::env::var("WINDIR").unwrap());
}
