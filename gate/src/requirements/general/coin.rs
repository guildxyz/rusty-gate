use crate::{
    providers::PROVIDERS,
    requirements::{errors::CheckableError, utils::check_if_in_range, Checkable},
    types::{Amount, AmountLimits, Chain, NumberId, ReqUserAccess, Requirement, User, UserAddress},
};
use anyhow::Result;
use async_trait::async_trait;

pub struct CoinRequirement {
    id: NumberId,
    data: Option<AmountLimits>,
    chain: Chain,
}

const DECIMALS: u32 = 18;
const DIVISOR: Amount = 10_u128.pow(DECIMALS) as Amount;

#[async_trait]
impl Checkable for CoinRequirement {
    async fn check(&self, users: &[User]) -> Vec<ReqUserAccess> {
        let user_addresses: Vec<UserAddress> = users
            .iter()
            .flat_map(|u| {
                u.addresses.iter().cloned().map(|address| UserAddress {
                    user_id: u.id,
                    address,
                })
            })
            .collect();

        if user_addresses.is_empty() {
            return users
                .iter()
                .map(|u| ReqUserAccess {
                    requirement_id: self.id,
                    user_id: u.id,
                    access: None,
                    amount: None,
                    warning: None,
                    error: Some(CheckableError::MissingUserAddress(u.id.to_string()).to_string()),
                })
                .collect();
        }

        futures::future::join_all(user_addresses.iter().map(|ua| async move {
            let mut error = None;
            let mut amount = None;

            match &PROVIDERS.get(&(self.chain as u8)) {
                Some(provider) => {
                    let response = provider.single.eth().balance(ua.address, None).await;

                    match response {
                        Ok(r) => amount = Some(r.as_u128() as Amount / DIVISOR),
                        Err(e) => error = Some(e.to_string()),
                    }
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
                    Some(check_if_in_range(
                        amount.expect("This should be fine"),
                        &self.data,
                        false,
                    ))
                } else {
                    None
                },
                amount: if error.is_none() { amount } else { None },
                warning: None,
                error,
            }
        }))
        .await
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

#[cfg(test)]
mod test {
    use super::CoinRequirement;
    use crate::{
        address,
        requirements::Checkable,
        types::{AmountLimits, Chain, User},
    };

    #[tokio::test]
    async fn coin_check() {
        dotenv::dotenv().ok();

        let users_1 = vec![User {
            id: 0,
            addresses: vec![address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")],
            platform_users: None,
        }];

        let users_2 = vec![User {
            id: 0,
            addresses: vec![address!("0x20CC54c7ebc5f43b74866D839b4BD5c01BB23503")],
            platform_users: None,
        }];

        let req = CoinRequirement {
            id: 0,
            chain: Chain::Ethereum,
            data: Some(AmountLimits {
                min_amount: Some(0.0004),
                max_amount: None,
            }),
        };

        assert_eq!(
            req.check(&users_1)
                .await
                .iter()
                .map(|a| a.access.unwrap_or_default())
                .collect::<Vec<bool>>(),
            vec![true]
        );
        assert_ne!(
            req.check(&users_2)
                .await
                .iter()
                .map(|a| a.access.unwrap_or_default())
                .collect::<Vec<bool>>(),
            vec![true]
        );
    }
}
