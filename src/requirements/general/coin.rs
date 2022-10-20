use crate::{
    config::PROVIDERS,
    requirements::{errors::CheckableError, utils::check_if_in_range, Checkable},
    types::{Amount, AmountLimits, Chain, NumberId, ReqUserAccess, Requirement, User, UserAddress},
};
use anyhow::Result;
use async_trait::async_trait;

pub struct CoinRequirement {
    id: NumberId,
    data: Option<AmountLimits>,
    #[allow(dead_code)]
    chain: Chain,
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

            match &PROVIDERS.read().await.get(&(self.chain as u8)) {
                Some(provider) => {
                    match (ua.address[2..]).parse() {
                        Ok(a) => {
                            let response = provider.single.eth().balance(a, None).await;

                            match response {
                                Ok(r) => amount = Some(r.as_u128() as f64 / DIVISOR),
                                Err(e) => error = Some(e.to_string()),
                            }
                        }
                        Err(e) => error = Some(e.to_string()),
                    };
                }
                None => {
                    error =
                        Some(CheckableError::NoSuchChain(format!("{:?}", self.chain)).to_string())
                }
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
