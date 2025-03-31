#![allow(unused_imports)]
use actix_web::{
    App, HttpRequest, HttpServer, Responder, get,
    web::{self, service},
};
use std::{env, fs::File, io::BufReader, net::SocketAddr, path::PathBuf};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[get("/")]
async fn index(_req: HttpRequest) -> impl Responder {
    "Hello TLS World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

    HttpServer::new(|| App::new().service(index))
        .bind_rustls_0_23(("127.0.0.1", 8443), tls_config)?
        .run()
        .await
}
