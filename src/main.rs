#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let mut app = tide::new();
    app.at("/").get(root);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

fn read_config() -> String {
    let config_dir = dirs::config_dir().unwrap();
    let config_path = config_dir.join("jmap-proxy/config.json");
    std::fs::read_to_string(config_path).unwrap()
}

async fn root(_req: tide::Request<()>) -> tide::Result<String> {
    Ok(read_config())
}
