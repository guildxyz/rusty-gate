use crate::{
    requirements::{errors::CheckableError, utils::check_if_in_range, Checkable},
    types::{Address, AmountLimits, EvmChain, NumberId, ReqUserAccess, Requirement, User},
};
use async_trait::async_trait;
use providers::{evm::general::PROVIDERS, BalanceQuerier};

pub struct Erc20Requirement {
    id: NumberId,
    address: Address,
    data: Option<AmountLimits>,
    chain: EvmChain,
}

#[async_trait]
impl Checkable for Erc20Requirement {
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
            .get_fungible_balance(self.address, &user_addresses)
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

impl TryFrom<&Requirement> for Erc20Requirement {
    type Error = CheckableError;

    fn try_from(req: &Requirement) -> Result<Self, Self::Error> {
        match req.chain {
            Some(chain) => {
                if PROVIDERS.get(&(chain as u8)).is_none() {
                    return Err(CheckableError::NoSuchChain(
                        CheckableError::NoSuchChain(format!("{chain:?}")).to_string(),
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
        types::{AmountLimits, EvmChain, User},
    };

    #[tokio::test]
    async fn erc20_check() {
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
            chain: EvmChain::Goerli,
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
