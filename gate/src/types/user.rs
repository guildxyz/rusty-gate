use super::{Address, Amount, NumberId, PlatformUser};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: NumberId,
    pub addresses: Vec<Address>,
    pub platform_users: Option<Vec<PlatformUser>>,
}

#[derive(Clone)]
pub struct ReqUserAccess {
    pub requirement_id: NumberId,
    pub user_id: NumberId,
    pub access: Option<bool>,
    pub amount: Option<Amount>,
    pub warning: Option<String>,
    pub error: Option<String>,
}
