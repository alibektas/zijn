mod auth;
mod schema;
mod session;
#[cfg(test)]
mod tests;
mod user;

use std::env;

use actix_web::HttpResponse;
use actix_web::post;
use auth::AuthResponse;
use auth::LoginRequest;
use auth::LogoutRequest;
use auth::SignupRequest;
use chrono::Duration;
use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;

use actix_web::{
    App, HttpServer, Responder,
    web::{self},
};
use session::Session;
use std::{fs::File, io::BufReader, path::PathBuf};
use tracing::debug;
use user::User;
use uuid::Uuid;

/// The auth endpoint that validates the user and generates a key.
#[post("/login")]
async fn login(state: web::Data<AppState>, auth_info: web::Json<LoginRequest>) -> impl Responder {
    // Get a database connection from the pool
    let mut conn = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    use crate::schema::active_sessions::dsl::*;
    use crate::schema::users::dsl::*;

    // Query the database to find a matching user.
    let Ok(user) = users
        .filter(email.eq(&auth_info.email))
        .filter(password.eq(&auth_info.password))
        .first::<User>(&mut conn)
    else {
        return HttpResponse::Unauthorized().finish();
    };

    tracing::info!(%user.email , "logged in");
    let session_key = Uuid::new_v4();
    let now_plus_1h = Utc::now()
        .checked_add_signed(Duration::seconds(3600))
        .unwrap()
        .naive_utc();

    // Add the new session into active_sessions
    let insert_result = diesel::insert_into(active_sessions)
        .values(&Session {
            id: user.id,
            session_id: session_key,
            expires_at: now_plus_1h,
        })
        .execute(&mut conn);

    match insert_result {
        Ok(_) => HttpResponse::Ok().json(AuthResponse { session_key }),
        Err(e) => {
            tracing::error!("Failed to insert session: {}", e);
            HttpResponse::InternalServerError().body("Failed to create session")
        }
    }
}

/// The logout endpoint that deletes the user's active session.
#[post("/logout")]
async fn logout(
    state: web::Data<AppState>,
    session_key: web::Json<LogoutRequest>,
) -> impl Responder {
    use crate::schema::active_sessions::dsl::*;
    use diesel::prelude::*;

    let mut conn = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let parsed_key = match Uuid::parse_str(&session_key.session_key.to_string()) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid session key"),
    };

    let deleted =
        diesel::delete(active_sessions.filter(session_id.eq(parsed_key))).execute(&mut conn);

    match deleted {
        Ok(0) => HttpResponse::NotFound().body("Session not found"),
        Ok(_) => HttpResponse::Ok().body("Logged out"),
        Err(e) => {
            tracing::error!("Failed to delete session: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// The signup endpoint that registers a user and returns a session key.
#[post("/signup")]
async fn signup(state: web::Data<AppState>, auth_info: web::Json<SignupRequest>) -> impl Responder {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = match state.db_pool.get() {
        Ok(conn) => conn,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Try to insert the new user
    let new_user = User {
        id: Uuid::new_v4(),
        email: auth_info.email.clone(),
        password: auth_info.password.clone(),
    };

    let insert_result = diesel::insert_into(users)
        .values(&new_user)
        .get_result::<User>(&mut conn);

    let user = match insert_result {
        Ok(user) => user,
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        )) => {
            return HttpResponse::Conflict().body("Email already exists");
        }
        Err(e) => {
            tracing::error!("Failed to insert user: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    tracing::info!(%user.email, "signed up");
    HttpResponse::Ok().into()
}

#[derive(Clone)]
struct AppState {
    db_pool: Pool<ConnectionManager<PgConnection>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let workspace_root = PathBuf::from(
        env::var("CARGO_WORKSPACE_DIR")
            .expect("CARGO_WORKSPACE_DIR not set please refer to .cargo/config.toml"),
    );

    let (mut certs_file, mut key_file) = if cfg!(debug_assertions) {
        let cert_folder = PathBuf::from("cert/auth");
        let cert_folder = workspace_root
            .join(cert_folder)
            .canonicalize()
            .expect("Cert folder could not be canonicalized");
        debug!(?cert_folder, "cert folder path");
        let certs_file = BufReader::new(File::open(cert_folder.join("cert.pem")).unwrap());
        let key_file = BufReader::new(File::open(cert_folder.join("key.pem")).unwrap());
        (certs_file, key_file)
    } else {
        panic!("Prod is TODO");
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

    let database_url = env::var("AUTH_DATABASE_URL").expect("AUTH_DATABASE_URL was not set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to create pool.");

    let app_state = web::Data::new(AppState { db_pool: pool });

    HttpServer::new(move || App::new().app_data(app_state.clone()).service(login))
        .bind_rustls_0_23(("127.0.0.1", 8444), tls_config)
        .unwrap()
        .run()
        .await
}
