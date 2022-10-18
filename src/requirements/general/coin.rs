use crate::{
    config::ETHEREUM_RPC,
    requirements::{errors::CheckableError, utils::check_if_in_range, Checkable},
    types::{Amount, AmountLimits, Chain, NumberId, ReqUserAccess, Requirement, User, UserAddress},
};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_aux::prelude::*;
use web3_rpc::{model::Tag, web3::Web3};

pub struct CoinRequirement {
    id: NumberId,
    data: Option<AmountLimits>,
    #[allow(dead_code)]
    chain: Chain,
}

#[derive(Deserialize, Debug)]
pub struct EtherscanResponse {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub result: u128,
}

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

            let rpc = Web3::new(ETHEREUM_RPC.to_string());

            let response = rpc.eth_get_balance(&ua.address, Some(Tag::Latest)).await;

            match response {
                Ok(r) => match r.result {
                    Some(v) => match u128::from_str_radix(&v[2..], 16) {
                        Ok(balance) => amount = Some(balance as f64 / DIVISOR),
                        Err(e) => error = Some(e.to_string()),
                    },
                    None => error = Some("Something went wrong".to_string()),
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
