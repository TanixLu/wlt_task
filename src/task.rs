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
        .join("wscript.exe")
        .to_string_lossy()
        .to_string();
    let current_dir = std::env::current_dir()?.to_string_lossy().to_string();
    let vbs_path = std::env::current_dir()?
        .join(VBS_NAME)
        .to_string_lossy()
        .to_string();
    let command = format!(
        r#"$action = New-ScheduledTaskAction -Execute {wscript_path} -WorkingDirectory {current_dir} -Argument {vbs_path}
$description = "Configure WLT and send notification emails when the IP changes"
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -StartWhenAvailable -DontStopIfGoingOnBatteries -RunOnlyIfNetworkAvailable

$triggers = @()
$triggers += New-ScheduledTaskTrigger -Once -At "2000-01-01 00:00:00" -RepetitionInterval (New-TimeSpan -Minutes 5)

$CIMTriggerClass = Get-CimClass -ClassName MSFT_TaskEventTrigger -Namespace Root/Microsoft/Windows/TaskScheduler:MSFT_TaskEventTrigger
$trigger = New-CimInstance -CimClass $CIMTriggerClass -ClientOnly
$trigger.Subscription = '<QueryList><Query Id="0" Path="Microsoft-Windows-NetworkProfile/Operational"><Select Path="Microsoft-Windows-NetworkProfile/Operational">*[System[(EventID=10000)]]</Select></Query></QueryList>'
$trigger.Delay = "PT5S"
$trigger.Enabled = $True
$triggers += $trigger

Register-ScheduledTask -Force -TaskName {TASK_NAME} -Action $action -Description $description -Settings $settings -Trigger $triggers"#
    );

    let output = Command::new("powershell").arg(command).output()?;
    output_string(output)
}

pub fn unset_task() -> AnyResult<String> {
    let output = Command::new("powershell")
        .arg(format!(
            "Unregister-ScheduledTask -TaskName {TASK_NAME} -TaskPath \\ -Confirm:$false"
        ))
        .output()?;
    output_string(output)
}

pub fn query_task() -> AnyResult<String> {
    let output = Command::new("powershell")
        .arg(format!(
            "Get-ScheduledTaskInfo -TaskName {TASK_NAME} -TaskPath \\ -Verbose"
        ))
        .output()?;
    output_string(output)
}
