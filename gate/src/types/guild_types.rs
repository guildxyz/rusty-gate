use serde::Deserialize;

pub type Address = String;
pub type NumberId = u64;
pub type Amount = f64;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Logic {
    And,
    Or,
    Nand,
    Nor,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequirementType {
    // Erc20,
    // Erc721,
    // Erc1155,
    Coin,
    Allowlist,
    Free,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Chain {
    Ethereum,
    Polygon,
    Gnosis,
    Bsc,
    Fantom,
    Avalanche,
    Heco,
    Harmony,
    Goerli,
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequirementData {
    pub addresses: Option<Vec<Address>>,
    pub min_amount: Option<String>,
    pub max_amount: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
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
    pub address: String,
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
