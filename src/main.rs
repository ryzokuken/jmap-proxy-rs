use std::collections::HashMap;

use serde::Deserialize;
use tide_http_auth::{BasicAuthRequest, Storage};

#[derive(Clone)]
struct State {
    config: Config,
}

impl State {
    fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Storage<(), BasicAuthRequest> for State {
    async fn get_user(&self, req: BasicAuthRequest) -> tide::Result<Option<()>> {
        if req.username == self.config.jmap.username && req.password == self.config.jmap.password {
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
    let mut app = tide::with_state(State::new(config));
    app.with(tide_http_auth::Authentication::new(
        tide_http_auth::BasicAuthScheme::default(),
    ));
    app.at("/").get(root);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

#[derive(Deserialize, Clone)]
struct IMAPConfig {
    username: String,
    password: String,
    email: String,
    host: String,
    port: u16,
    tls: bool,
}

#[derive(Deserialize, Clone)]
struct JMAPConfig {
    username: String,
    password: String,
}

#[derive(Deserialize, Clone)]
struct Config {
    imap: IMAPConfig,
    jmap: JMAPConfig,
}

fn read_config() -> Config {
    let config_dir = dirs::config_dir().unwrap();
    let config_path = config_dir.join("jmap-proxy/config.json");
    let config_str = std::fs::read_to_string(config_path).unwrap();
    serde_json::from_str(&config_str).unwrap()
}

async fn root(req: tide::Request<State>) -> tide::Result<String> {
    let email = req.state().config.imap.email.clone();
    let account = libjmap::rfc8620::Account {
        name: email,
        is_personal: true,
        is_read_only: true,
        account_capabilities: HashMap::default(),
        extra_properties: HashMap::default(),
    };
    let mut session = libjmap::rfc8620::JmapSession::default();
    session.accounts.insert(libjmap::rfc8620::Id::new(), account);
    Ok(serde_json::to_string(&session).unwrap())
}
