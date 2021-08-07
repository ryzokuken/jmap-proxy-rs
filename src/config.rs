use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct IMAPConfig {
    username: String,
    password: String,
    pub email: String,
    host: String,
    port: u16,
    tls: bool,
}

#[derive(Deserialize, Clone)]
pub struct JMAPConfig {
    pub username: String,
    pub password: String,
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub imap: IMAPConfig,
    pub jmap: JMAPConfig,
}

pub fn read_config() -> Config {
    let config_dir = dirs::config_dir().unwrap();
    let config_path = config_dir.join("jmap-proxy/config.json");
    let config_str = std::fs::read_to_string(config_path).unwrap();
    serde_json::from_str(&config_str).unwrap()
}
