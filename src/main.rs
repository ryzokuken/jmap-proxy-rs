use std::sync::Arc;

use libjmap::rfc8620::{Account, CapabilitiesObject, Id, JmapSession};
use tide_http_auth::{BasicAuthRequest, Storage};

mod config;

use config::Config;

#[derive(Clone)]
struct State {
    config: Config,
    account: Account,
    account_id: Id<Account>,
    address: String,
}

trait GenerateID {
    fn generate_id<T>() -> Id<T>;
}

impl GenerateID for Account {
    fn generate_id<T>() -> Id<T> {
        let id = uuid::Uuid::new_v4().to_string();
        Id::from('A'.to_string() + &id)
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
            address,
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
    let config = config::read_config();
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

struct URLs {
    api: String,
    download: String,
    upload: String,
    event_source: String,
}

impl URLs {
    fn from_address(addr: &str) -> Self {
        Self {
            api: format!("https://{}/api", addr),
            download: format!(
                "https://{}/download/{{accountId}}/{{blobId}}/{{name}}?accept={{type}}",
                addr
            ),
            upload: format!("https://{}/upload/{{accountId}}/", addr),
            event_source: format!(
                "https://{}/eventsource?types={{types}}&closeafter={{closeafter}}&ping={{ping}}",
                addr
            ),
        }
    }
}

fn generate_session(state: &State) -> JmapSession {
    let mut session = JmapSession::default();
    let core_capabilities = CapabilitiesObject {
        max_size_upload: 50_000_000,
        max_concurrent_upload: 4,
        max_size_request: 10_000_000,
        max_concurrent_requests: 4,
        max_calls_in_request: 16,
        max_objects_in_get: 500,
        max_objects_in_set: 500,
        collation_algorithms: Vec::default(), // TODO: properly set this list
    };
    session
        .capabilities
        .insert("urn:ietf:params:jmap:core".to_string(), core_capabilities);
    session
        .accounts
        .insert(state.account_id.clone(), state.account.clone());
    session.username = state.config.imap.email.clone();

    let urls = URLs::from_address(&state.address);
    session.api_url = Arc::new(urls.api);
    session.download_url = Arc::new(urls.download);
    session.upload_url = Arc::new(urls.upload);
    session.event_source_url = Arc::new(urls.event_source);

    // TODO: set other fields properly
    session
}

async fn root(req: tide::Request<State>) -> tide::Result<String> {
    let session = generate_session(req.state());
    Ok(serde_json::to_string(&session).unwrap())
}
