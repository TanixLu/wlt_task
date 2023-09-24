use std::io::Write;

use chrono::Local;

use crate::utils::AnyResult;

const LOG_PATH: &str = "log.txt";

pub fn log(msg: impl AsRef<str>) -> AnyResult<()> {
    let msg = format!(
        "{}: {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        msg.as_ref()
    );
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)?;
    file.write_all(msg.as_bytes())?;
    Ok(())
}
