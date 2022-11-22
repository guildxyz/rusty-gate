use crate::{
    requirements::{
        errors::CheckableError,
        general::{allowlist::AllowListRequirement, coin::CoinRequirement, free::FreeRequirement},
        Checkable,
    },
    types::{Chain, NumberId},
};
use serde::{Deserialize, Serialize};
pub use web3::types::Address;

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

impl Requirement {
    pub fn inner(&self) -> Result<Box<dyn Checkable>, CheckableError> {
        use RequirementType::*;

        Ok(match self.typ {
            Free => Box::new(FreeRequirement::try_from(self)?),
            Allowlist => Box::new(AllowListRequirement::try_from(self)?),
            Coin => Box::new(CoinRequirement::try_from(self)?),
        })
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequirementError {
    pub requirement_id: NumberId,
    pub msg: String,
}
