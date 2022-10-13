use crate::{
    requirements::Checkable,
    types::{Amount, Chain, NumberId, ReqUserAccess, Requirement, User, UserAddress},
};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_aux::prelude::*;

struct CoinReqData {
    pub min_amount: Option<Amount>,
    pub max_amount: Option<Amount>,
}

pub struct CoinRequirement {
    id: NumberId,
    data: Option<CoinReqData>,
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
            .clone()
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

            let access = {
                if self.data.is_none() {
                    amount > 0.0
                } else {
                    let data = self.data.as_ref().expect("This should be fine");
                    let min_amount = data.min_amount.unwrap_or_default();
                    let min_ok = if min_amount > 0.0 {
                        amount >= min_amount
                    } else {
                        amount > 0.0
                    };

                    match data.max_amount {
                        Some(max_amount) => min_ok && amount < max_amount,
                        None => min_ok,
                    }
                }
            };

            ReqUserAccess {
                requirement_id: self.id,
                user_id: ua.user_id,
                access,
                amount,
            }
        }))
        .await;

        Ok(res)
    }
}

impl From<&Requirement> for CoinRequirement {
    fn from(req: &Requirement) -> Self {
        CoinRequirement {
            id: req.id,
            data: if req.data.is_some() {
                let req_data = req.data.as_ref().expect("This should be fine");

                Some(CoinReqData {
                    min_amount: if let Some(value) = &req_data.min_amount {
                        match value.parse::<Amount>() {
                            Ok(v) => Some(v),
                            Err(_) => None,
                        }
                    } else {
                        None
                    },
                    max_amount: if let Some(value) = &req_data.max_amount {
                        match value.parse::<Amount>() {
                            Ok(v) => Some(v),
                            Err(_) => None,
                        }
                    } else {
                        None
                    },
                })
            } else {
                None
            },
            chain: req.chain.unwrap(),
        }
    }
}
