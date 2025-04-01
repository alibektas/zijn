use clap::Parser;
use realms::get_realms;
use reqwest::Url;
use std::env;
use std::sync::OnceLock;

mod realms;

pub(crate) fn get_address() -> &'static Url {
    static HASHMAP: OnceLock<Url> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let adr = env::var("KC_SERVER_ADDRESS").expect("KC_SERVER_ADDRESS env is not set");
        let port = env::var("KC_SERVER_PORT").expect("KC_SERVER_PORT env is not set");

        let formatted = format!("http://{adr}:{port}/");

        Url::parse(&format!("http://{adr}:{port}/"))
            .unwrap_or_else(|_| panic!("{} could not be parsed into a URL", formatted))
    })
}

#[derive(Debug, Parser)]
enum Cmds {
    /// Get all realms
    GetRealms,
}

#[tokio::main]
async fn main() {
    if !cfg!(debug_assertions) {
        panic!("Tasks are DEV only");
    }

    dotenv::dotenv().unwrap();
    let s = Cmds::parse();

    match s {
        Cmds::GetRealms => get_realms().await,
    }
}
