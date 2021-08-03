use serde::{Deserialize, Serialize};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let config = read_config();
    tide::log::start();
    let mut app = tide::new();
    app.at("/").get(root);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct IMAPConfig {
    username: String,
    password: String,
    host: String,
    port: u16,
    tls: bool
}

#[derive(Serialize, Deserialize)]
struct JMAPConfig {
    username: String,
    password: String
}

#[derive(Serialize, Deserialize)]
struct Config {
    imap: IMAPConfig,
    jmap: JMAPConfig
}

fn read_config() -> Config {
    let config_dir = dirs::config_dir().unwrap();
    let config_path = config_dir.join("jmap-proxy/config.json");
    let config_str = std::fs::read_to_string(config_path).unwrap();
    serde_json::from_str(&config_str).unwrap()
}

async fn root(_req: tide::Request<()>) -> tide::Result<String> {
    Ok(String::from("hello"))
}
