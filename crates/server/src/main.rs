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
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut certs_file = BufReader::new(File::open("cert.pem").unwrap());
    let mut key_file = BufReader::new(File::open("key.pem").unwrap());

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
