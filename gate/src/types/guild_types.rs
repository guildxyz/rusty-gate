use super::{Address, Amount, NumberId};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Role {
    pub id: Option<NumberId>,
    pub logic: String,
    pub requirements: Vec<Requirement>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CheckRolesOfMembersRequest {
    pub users: Vec<User>,
    pub roles: Vec<Role>,
    pub send_details: Option<bool>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlatformName {
    Discord = 1,
    Telegram = 2,
    Github = 3,
    Google = 4,
    Twitter = 5,
    Steam = 6,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlatformUserData {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlatformUser {
    pub platform_id: NumberId,
    pub platform_name: PlatformName,
    pub platform_user_id: String,
    pub platform_user_data: Option<PlatformUserData>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: NumberId,
    pub addresses: Vec<Address>,
    pub platform_users: Option<Vec<PlatformUser>>,
}

pub struct UserAddress {
    pub user_id: NumberId,
    pub address: Address,
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
