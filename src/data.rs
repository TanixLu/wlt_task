use serde::{Deserialize, Serialize};

const DATA_PATH: &str = "data.toml";
const DATA_COMMENT: &str = r#"
# ipv4：用于记录之前的IPv4地址，当IPv4地址变动时，会自动发送邮件通知
# ipv6：用于记录之前的IPv6地址，当IPv6地址变动时，会自动发送邮件通知
# rn：Cookie中的一个字段
# 连续超时次数: 连续超时次数
"#;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Data {
    pub ipv4: String,
    pub ipv6: String,
    pub rn: String,
    pub 连续超时次数: u32,
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
