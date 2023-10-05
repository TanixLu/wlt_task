use serde::{Deserialize, Serialize};

const DATA_PATH: &str = "data.toml";
const DATA_COMMENT: &str = r#"
# ip：用于记录之前的ip地址，当ip地址变动时，会自动发送邮件通知
# rn：Cookie中的一个字段
# time_out_count: timeout次数
"#;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Data {
    pub ip: String,
    pub rn: String,
    pub timeout_count: u32,
}

impl Data {
    pub fn save(&self) -> anyhow::Result<()> {
        let data_string = toml::to_string_pretty(self)?;
        let content = format!("{}\n{}", data_string, DATA_COMMENT);
        std::fs::write(DATA_PATH, content)?;
        Ok(())
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = std::path::Path::new(DATA_PATH);
        if !path.exists() || path.metadata().unwrap().len() == 0 {
            let data = Data::default();
            data.save()?;
            Ok(data)
        } else {
            let data = std::fs::read_to_string(DATA_PATH)?;
            let data = toml::from_str::<Data>(&data)?;
            Ok(data)
        }
    }
}
