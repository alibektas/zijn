use keycloak_admin_api::{
    KeyCloakAdmin,
    req::{AdminGetRealmsRequest, Requestable},
};
use reqwest::{Client, get};

use crate::get_address;

pub(crate) async fn get_realms() {
    let client = Client::builder().http1_only().build().unwrap();

    let address = get_address().join(AdminGetRealmsRequest::ENDPOINT).unwrap();
    // address = https://localhost:8080/admin/realms
    let response = client
        .get(address)
        .query(&KeyCloakAdmin {
            username: "admin".to_owned(),
            password: "admin_password".to_owned(),
        })
        .send()
        .await;

    match response {
        Ok(res) => {
            let status = res.status();
            let body = res.text().await.unwrap();

            match status.as_u16() {
                200 => {
                    let res = AdminGetRealmsRequest::parse_response(&body).unwrap();
                    dbg!(&res);
                }
                e => {
                    panic!("Not handled <400 code! {}", e);
                }
            }
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
}
