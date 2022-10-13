use crate::{
    requirements::{errors::CheckableError, utils::check_if_in_range, Checkable},
    types::{AmountLimits, Chain, NumberId, ReqUserAccess, Requirement, User, UserAddress},
};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_aux::prelude::*;

pub struct CoinRequirement {
    id: NumberId,
    data: Option<AmountLimits>,
    chain: Chain,
}

#[derive(Deserialize, Debug)]
pub struct EtherscanResponse {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub result: u128,
}

// Etherscan
const ETHERSCAN: &str = "https://api.etherscan.io/api?module=account&action=balance&address=";
const TAG_AND_KEY: &str = "&tag=latest&apikey=";
const ETHERSCAN_API_KEY: &str = std::include_str!("../../../.secrets/etherscan-api-key");

#[async_trait]
impl Checkable for CoinRequirement {
    async fn check(&self, users: &[User]) -> Result<Vec<ReqUserAccess>> {
        let user_addresses: Vec<UserAddress> = users
            .iter()
            .flat_map(|u| {
                u.addresses.iter().map(|a| UserAddress {
                    user_id: u.id,
                    address: a.into(),
                })
            })
            .collect();

        let res = futures::future::join_all(user_addresses.iter().map(|ua| async move {
            let body: EtherscanResponse = reqwest::get(format!(
                "{ETHERSCAN}{}{TAG_AND_KEY}{ETHERSCAN_API_KEY}",
                ua.address
            ))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

            let amount = (body.result as f64) / (10_u128.pow(18) as f64);

            ReqUserAccess {
                requirement_id: self.id,
                user_id: ua.user_id,
                access: check_if_in_range(amount, &self.data, false),
                amount,
            }
        }))
        .await;

        Ok(res)
    }
}

impl TryFrom<&Requirement> for CoinRequirement {
    type Error = CheckableError;

    fn try_from(req: &Requirement) -> Result<Self, Self::Error> {
        if req.chain.is_none() {
            return Err(CheckableError::MissingField("chain".into()));
        }
        match req.chain {
            Some(chain) => {
                let res = CoinRequirement {
                    id: req.id,
                    data: AmountLimits::from_req(req),
                    chain,
                };

                Ok(res)
            }
            None => Err(CheckableError::MissingField("chain".into())),
        }
    }
}
