use crate::{
    requirements::{
        errors::CheckableError,
        general::{
            allowlist::AllowListRequirement,
            coin::CoinRequirement,
            free::FreeRequirement,
            token::{Erc1155Requirement, Erc20Requirement, Erc721Requirement},
        },
        Checkable,
    },
    types::{Address, EvmChain, NumberId, U256},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequirementType {
    Erc20,
    Erc721,
    Erc1155,
    Coin,
    Allowlist,
    Free,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequirementData {
    pub id: Option<U256>,
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
    pub chain: Option<EvmChain>,
}

impl Requirement {
    pub fn inner(&self) -> Result<Box<dyn Checkable>, CheckableError> {
        use RequirementType::*;

        Ok(match self.typ {
            Free => Box::new(FreeRequirement::try_from(self)?),
            Allowlist => Box::new(AllowListRequirement::try_from(self)?),
            Coin => Box::new(CoinRequirement::try_from(self)?),
            Erc20 => Box::new(Erc20Requirement::try_from(self)?),
            Erc721 => Box::new(Erc721Requirement::try_from(self)?),
            Erc1155 => Box::new(Erc1155Requirement::try_from(self)?),
        })
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequirementError {
    pub requirement_id: NumberId,
    pub msg: String,
}
