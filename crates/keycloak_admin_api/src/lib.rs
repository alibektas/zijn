use serde::Serialize;

pub mod req;
pub mod res;

#[derive(Serialize)]
pub struct KeyCloakAdmin {
    pub username: String,
    pub password: String,
}
