use core::fmt;

use serde::{Serialize, de::DeserializeOwned};

use crate::res::AdminGetRealmsResponse;

pub trait Requestable
where
    Self: Serialize,
{
    type ResponseTy: DeserializeOwned + fmt::Debug;
    const ENDPOINT: &'static str;

    fn parse_response(de: &str) -> Result<Self::ResponseTy, serde_json::Error> {
        serde_json::from_str::<Self::ResponseTy>(de)
    }
}

/// Get accessible realms Returns a list of accessible realms. The list is filtered based on what realms the caller is allowed to view.
#[derive(Serialize, Default)]
pub struct AdminGetRealmsRequest {
    brief_representation: bool,
}

impl Requestable for AdminGetRealmsRequest {
    const ENDPOINT: &'static str = "/admin/realms";
    type ResponseTy = AdminGetRealmsResponse;
}
