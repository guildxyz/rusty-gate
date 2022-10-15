use crate::{
    requirements::{errors::CheckableError, utils::check_if_in_range, Checkable},
    types::{Amount, AmountLimits, Chain, NumberId, ReqUserAccess, Requirement, User, UserAddress},
};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_aux::prelude::*;

pub struct CoinRequirement {
    id: NumberId,
    data: Option<CoinReqData>,
    #[allow(dead_code)]
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

const DECIMALS: u32 = 18;
const DIVISOR: Amount = 10_u128.pow(DECIMALS) as Amount;

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
            let mut error = None;
            let mut amount = None;

            let response = reqwest::get(format!(
                "{ETHERSCAN}{}{TAG_AND_KEY}{ETHERSCAN_API_KEY}",
                ua.address
            ))
            .await;

            match response {
                Ok(result) => match result.json::<EtherscanResponse>().await {
                    Ok(body) => amount = Some(body.result as f64 / DIVISOR),
                    Err(e) => error = Some(e.to_string()),
                },
                Err(e) => error = Some(e.to_string()),
            }

            ReqUserAccess {
                requirement_id: self.id,
                user_id: ua.user_id,
                access: if error.is_none() {
                    Some(check_if_in_range(amount.unwrap(), &self.data, false))
                } else {
                    None
                },
                amount: if error.is_none() { amount } else { None },
                warning: None,
                error,
            }
        }))
        .await;

        Ok(res)
    }
}

impl TryFrom<&Requirement> for CoinRequirement {
    type Error = CheckableError;

    fn try_from(req: &Requirement) -> Result<Self, Self::Error> {
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
