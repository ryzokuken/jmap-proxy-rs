use serde::{Deserialize, Serialize};
use tide_http_auth::{BasicAuthRequest, Storage};

#[derive(Clone)]
struct ServerAuthState {
    username: String,
    password: String
}

impl ServerAuthState {
    fn new(username: String, password: String) -> Self {
        ServerAuthState { username, password }
    }
}

#[async_trait::async_trait]
impl Storage<(), BasicAuthRequest> for ServerAuthState {
    async fn get_user(&self, req: BasicAuthRequest) -> tide::Result<Option<()>> {
        if req.username == self.username && req.password == self.password {
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let config = read_config();
    let mut app = tide::with_state(ServerAuthState::new(config.jmap.username, config.jmap.password));
    app.with(tide_http_auth::Authentication::new(tide_http_auth::BasicAuthScheme::default()));
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

async fn root<State>(_req: tide::Request<State>) -> tide::Result<String> {
    Ok(String::from("hello"))
}
