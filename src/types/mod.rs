use serde::Serialize;

pub mod guild_types;
pub use guild_types::*;

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Debug)]
pub struct DetailedAccess {
    pub requirement_id: NumberId,
    pub access: bool,
    pub amount: Amount,
}

#[derive(Serialize, Debug)]
pub struct Access {
    pub id: NumberId,
    pub access: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed: Option<Vec<DetailedAccess>>,
}

#[derive(Serialize, Debug)]
pub struct CheckAccessResult {
    pub accesses: Vec<Access>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<RequirementError>>,
}

#[derive(Copy, Clone)]
pub struct ReqUserAccess {
    pub requirement_id: NumberId,
    pub user_id: NumberId,
    pub access: bool,
    pub amount: Amount,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Debug, Clone)]
pub struct RequirementError {
    pub requirement_id: NumberId,
    pub msg: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Debug)]
pub struct CheckRolesOfMembersResult {
    pub role_id: NumberId,
    pub users: Vec<Access>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<RequirementError>>,
}
