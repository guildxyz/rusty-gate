use serde::Serialize;

pub mod guild_types;
pub use guild_types::*;

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Debug)]
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

#[derive(Serialize, Debug)]
pub struct CheckAccessResult {
    pub accesses: Vec<Access>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<RequirementError>>,
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
