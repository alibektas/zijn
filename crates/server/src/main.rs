mod chatdb;
mod middleware;

use actix_web::{
    App, HttpRequest, HttpServer, Responder, get,
    web::{self},
};
use keycloak::{self, KeycloakAdmin, KeycloakAdminToken};
use std::{env, fs::File, io::BufReader, path::PathBuf};

#[get("/")]
async fn index(data: web::Data<AppState>, _req: HttpRequest) -> impl Responder {
    "Hello TLS World!"
}

struct AppState {
    keycloak: KeycloakAdmin,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    // .env files being a resource for hiding sensitive data
    // we restrict its use exclusively to dev builds.
    #[cfg(debug_assertions)]
    dotenv::dotenv().expect("Error loading .env file");

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let workspace_root = PathBuf::from(
        env::var("CARGO_WORKSPACE_DIR")
            .expect("CARGO_WORKSPACE_DIR not set please refer to .cargo/config.toml"),
    );

    let (mut certs_file, mut key_file) = if cfg!(debug_assertions) {
        let cert_folder = PathBuf::from(
            env::var("DEV_SERVER_CERT_FOLDER").expect("DEV_SERVER_CERT_FOLDER not set"),
        );
        let cert_folder = workspace_root
            .join(cert_folder)
            .canonicalize()
            .expect("DEV_SERVER_CERT_FOLDER could not be canonicalized");
        let certs_file = BufReader::new(File::open(cert_folder.join("server/cert.pem")).unwrap());
        let key_file = BufReader::new(File::open(cert_folder.join("server/key.pem")).unwrap());
        (certs_file, key_file)
    } else {
        todo!()
    };

    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();

    // set up TLS config options
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap();

    // Fetch the admin token and initialize KeycloakAdmin synchronously
    let url = std::env::var("KEYCLOAK_ADDR").unwrap();
    let user = std::env::var("KEYCLOAK_ADMIN").unwrap();
    let password = std::env::var("KEYCLOAK_ADMIN_PASSWORD").unwrap();
    let client = reqwest::Client::new();
    let admin_token = KeycloakAdminToken::acquire(&url, &user, &password, &client)
        .await
        .expect("Admin token couldn't be acquired");

    let admin = KeycloakAdmin::new(&url, admin_token, client);
    let app_state = web::Data::new(AppState { keycloak: admin });

    HttpServer::new(move || App::new().app_data(app_state.clone()).service(index))
        .bind_rustls_0_23(("127.0.0.1", 8443), tls_config)
        .unwrap()
        .run()
        .await
}
