use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

mod platform;
mod requirement;
mod user;
pub use platform::*;
pub use requirement::*;
pub use user::*;

pub use providers::{Address, EvmChain, U256};
pub type NumberId = u64;
pub type Amount = f64;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DetailedAccess {
    pub requirement_id: NumberId,
    pub access: Option<bool>,
    pub amount: Option<Amount>,
}

#[derive(Serialize, Debug)]
pub struct Access {
    pub id: NumberId,
    pub access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<RequirementError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<RequirementError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed: Option<Vec<DetailedAccess>>,
}

#[skip_serializing_none]
#[derive(Serialize, Debug)]
pub struct CheckAccessResult {
    pub accesses: Vec<Access>,
    pub errors: Option<Vec<RequirementError>>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequirementError {
    pub requirement_id: NumberId,
    pub msg: String,
}

#[derive(Serialize, Debug)]
#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
pub struct CheckRolesOfMembersResult {
    pub role_id: NumberId,
    pub users: Vec<Access>,
    pub errors: Option<Vec<RequirementError>>,
}

pub struct AmountLimits {
    pub min_amount: Option<Amount>,
    pub max_amount: Option<Amount>,
}

impl AmountLimits {
    pub fn from_req(req: &Requirement) -> Option<Self> {
        let get_inner = |field: &Option<String>| match field {
            Some(value) => match value.parse::<Amount>() {
                Ok(v) => Some(v),
                Err(_) => None,
            },
            None => None,
        };

        req.data.as_ref().map(|data| Self {
            min_amount: get_inner(&data.min_amount),
            max_amount: get_inner(&data.max_amount),
        })
    }
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
