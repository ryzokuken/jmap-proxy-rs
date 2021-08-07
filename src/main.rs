use libjmap::rfc8620::{Account, Id};
use serde::Deserialize;
use tide_http_auth::{BasicAuthRequest, Storage};

#[derive(Clone)]
struct State {
    config: Config,
    account: Account,
    account_id: Id<Account>,
    address: String
}

trait GenerateID {
    fn generate_id<T>() -> Id<T>;
}

impl GenerateID for Account {
    fn generate_id<T>() -> Id<T> {
        let id = uuid::Uuid::new_v4().to_string();
        Id::from(String::from('A') + &id)
    }
}

impl State {
    fn new(config: Config, address: String) -> Self {
        let account = Account::new(config.imap.email.clone(), true, true, None);
        let account_id = Account::generate_id();
        Self {
            config,
            account,
            account_id,
            address
        }
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
    let jmap_config = config.jmap.clone();
    let host = jmap_config.host.unwrap_or_else(|| "127.0.0.1".to_string());
    let port = jmap_config.port.unwrap_or_else(|| 8080);
    let addr = format!("{}:{}", host, port);
    let mut app = tide::with_state(State::new(config, addr.clone()));
    app.with(tide_http_auth::Authentication::new(
        tide_http_auth::BasicAuthScheme::default(),
    ));
    app.at("/").get(root);
    app.listen(&addr).await?;
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
    host: Option<String>,
    port: Option<u16>,
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
    let state = req.state();
    let mut session = libjmap::rfc8620::JmapSession::default();
    session
        .accounts
        .insert(state.account_id.clone(), state.account.clone());
    Ok(serde_json::to_string(&session).unwrap())
}
