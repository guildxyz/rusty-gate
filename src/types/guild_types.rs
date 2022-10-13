use serde::Deserialize;

pub type Address = String;
pub type NumberId = u64;
pub type Amount = f64;

#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Logic {
    And,
    Or,
    Nand,
    Nor,
}

#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum RequirementType {
    // Erc20,
    // Erc721,
    // Erc1155,
    Coin,
    Allowlist,
    Free,
}

#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Chain {
    Ethereum,
    Polygon,
    Gnosis,
    Bsc,
    Fantom,
    Avalanche,
    Heco,
    Harmony,
    Goerly,
    Arbitrum,
    Celo,
    Optimism,
    Moonriver,
    Rinkeby,
    Metis,
    Cronos,
    Boba,
    Palm,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug, Clone)]
pub struct RequirementData {
    pub addresses: Option<Vec<Address>>,
    pub min_amount: Option<String>,
    pub max_amount: Option<String>,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug, Clone)]
pub struct Requirement {
    pub id: NumberId,
    #[serde(rename(deserialize = "type"))]
    pub typ: RequirementType,
    pub address: Option<Address>,
    pub data: Option<RequirementData>,
    pub chain: Option<Chain>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Role {
    pub id: Option<NumberId>,
    pub logic: Logic,
    pub requirements: Vec<Requirement>,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct CheckRolesOfMembersRequest {
    pub users: Vec<User>,
    pub roles: Vec<Role>,
    pub send_details: Option<bool>,
}

#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Deserialize, Debug, Clone, Copy)]
pub enum PlatformName {
    Discord = 1,
    Telegram = 2,
    Github = 3,
    Google = 4,
    Twitter = 5,
    Steam = 6,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug, Clone)]
pub struct PlatformUserData {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug, Clone)]
pub struct PlatformUser {
    pub platform_id: NumberId,
    pub platform_name: PlatformName,
    pub platform_user_id: String,
    pub platform_user_data: Option<PlatformUserData>,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug, Clone)]
pub struct User {
    pub id: NumberId,
    pub addresses: Vec<Address>,
    pub platform_users: Option<Vec<PlatformUser>>,
}

pub struct UserAddress {
    pub user_id: NumberId,
    pub address: String,
}
