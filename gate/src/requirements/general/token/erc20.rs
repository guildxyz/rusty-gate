use std::sync::Arc;

use crate::{
    providers::PROVIDERS,
    requirements::{
        errors::CheckableError, general::token::ERC20_ABI, utils::check_if_in_range, Checkable,
    },
    types::{
        Address, Amount, AmountLimits, Chain, NumberId, ReqUserAccess, Requirement, User,
        UserAddress, U256,
    },
};
use async_trait::async_trait;
use web3::contract::Options;

pub struct Erc20Requirement {
    id: NumberId,
    address: Address,
    data: Option<AmountLimits>,
    chain: Chain,
}

#[async_trait]
impl Checkable for Erc20Requirement {
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

        let provider = PROVIDERS
            .get(&(self.chain as u8))
            .expect("This should be fine");

        let contract = Arc::new(
            web3::contract::Contract::from_json(provider.single.eth(), self.address, ERC20_ABI)
                .unwrap(),
        );

        let decimals: u8 = contract
            .query("decimals", (), None, Options::default(), None)
            .await
            .unwrap();

        futures::future::join_all(user_addresses.iter().map(|ua| async {
            let mut error = None;
            let mut amount = None;

            let contract = Arc::clone(&contract);

            let response: Result<U256, web3::contract::Error> = contract
                .query("balanceOf", (ua.address,), None, Options::default(), None)
                .await;

            match response {
                Ok(r) => {
                    amount = Some(r.as_u128() as Amount / 10_u128.pow(decimals as u32) as Amount)
                }
                Err(e) => error = Some(e.to_string()),
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

impl TryFrom<&Requirement> for Erc20Requirement {
    type Error = CheckableError;

    fn try_from(req: &Requirement) -> Result<Self, Self::Error> {
        match req.chain {
            Some(chain) => {
                if PROVIDERS.get(&(chain as u8)).is_none() {
                    return Err(CheckableError::NoSuchChain(
                        CheckableError::NoSuchChain(format!("{:?}", chain)).to_string(),
                    ));
                }

                match req.address {
                    Some(address) => {
                        let res = Erc20Requirement {
                            id: req.id,
                            address,
                            data: AmountLimits::from_req(req),
                            chain,
                        };

                        Ok(res)
                    }
                    None => Err(CheckableError::MissingTokenAddress(req.id.to_string())),
                }
            }
            None => Err(CheckableError::MissingField("chain".into())),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        address,
        requirements::{general::token::erc20::Erc20Requirement, Checkable},
        types::{AmountLimits, Chain, User},
    };

    #[tokio::test]
    async fn erc20_check() {
        dotenv::dotenv().ok();

        let users_1 = vec![User {
            id: 0,
            addresses: vec![address!("0x14DDFE8EA7FFc338015627D160ccAf99e8F16Dd3")],
            platform_users: None,
        }];

        let users_2 = vec![User {
            id: 0,
            addresses: vec![address!("0x20CC54c7ebc5f43b74866D839b4BD5c01BB23503")],
            platform_users: None,
        }];

        let req = Erc20Requirement {
            id: 0,
            chain: Chain::Goerli,
            address: address!("0x3C65D35A8190294d39013287B246117eBf6615Bd"),
            data: Some(AmountLimits {
                min_amount: Some(420.69),
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
