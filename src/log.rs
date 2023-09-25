use std::io::{Read, Seek, Write};

use chrono::Local;

const LOG_PATH: &str = "log.txt";

pub fn log_append(msg: impl AsRef<str>) {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)
        .unwrap();
    file.write_all(msg.as_ref().as_bytes()).unwrap();
}

fn need_new_line() -> bool {
    if !std::path::Path::new(LOG_PATH).exists() {
        return false;
    }
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(LOG_PATH)
        .unwrap();
    if file.metadata().unwrap().len() == 0 {
        return false;
    }
    let mut buf = [0; 1];
    file.seek(std::io::SeekFrom::End(-1)).unwrap();
    file.read_exact(&mut buf).unwrap();
    buf[0] != b'\n'
}

pub fn log(msg: impl AsRef<str>) {
    let mut log_text = String::new();
    if need_new_line() {
        log_text.push('\n');
    }

    log_text.push_str(&format!(
        "{}: {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        msg.as_ref()
    ));
    log_append(log_text);
}
