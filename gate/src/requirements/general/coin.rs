use crate::{
    requirements::{errors::CheckableError, utils::check_if_in_range, Checkable},
    types::{Address, AmountLimits, Chain, NumberId, ReqUserAccess, Requirement, User},
};
use anyhow::Result;
use async_trait::async_trait;
use providers::{evm::general::PROVIDERS, BalanceQuerier};

pub struct CoinRequirement {
    id: NumberId,
    data: Option<AmountLimits>,
    chain: Chain,
}

#[async_trait]
impl Checkable for CoinRequirement {
    async fn check(&self, users: &[User]) -> Vec<ReqUserAccess> {
        let user_addresses: Vec<Address> = users
            .iter()
            .flat_map(|u| u.addresses.iter().cloned())
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

        let provider = PROVIDERS
            .get(&(self.chain as u8))
            .expect("This should be fine");

        provider
            .get_native_balance(&user_addresses)
            .await
            .iter()
            .enumerate()
            .map(|(idx, b)| {
                let mut error = None;
                let mut amount = None;

                match b {
                    Ok(v) => amount = Some(*v),
                    Err(e) => error = Some(e.to_string()),
                };

                let user_id = users
                    .iter()
                    .find(|u| u.addresses.contains(&user_addresses[idx]))
                    .unwrap()
                    .id;

                ReqUserAccess {
                    requirement_id: self.id,
                    user_id,
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
            })
            .collect()
    }
}

impl TryFrom<&Requirement> for CoinRequirement {
    type Error = CheckableError;

    fn try_from(req: &Requirement) -> Result<Self, Self::Error> {
        match req.chain {
            Some(chain) => {
                if PROVIDERS.get(&(chain as u8)).is_none() {
                    return Err(CheckableError::NoSuchChain(format!("{:?}", chain)));
                }

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
    use crate::{
        address,
        requirements::{general::coin::CoinRequirement, Checkable},
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
