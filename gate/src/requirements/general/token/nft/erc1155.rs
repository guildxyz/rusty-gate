use crate::{
    requirements::{
        errors::CheckableError, general::token::nft::NftData, utils::check_if_in_range, Checkable,
    },
    types::{Address, AmountLimits, EvmChain, NumberId, ReqUserAccess, Requirement, User},
};
use async_trait::async_trait;
use providers::{evm::general::PROVIDERS, BalanceQuerier};

pub struct Erc1155Requirement {
    id: NumberId,
    address: Address,
    data: NftData,
    chain: EvmChain,
}

#[async_trait]
impl Checkable for Erc1155Requirement {
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
            .get_special_balance(self.address, self.data.id, &user_addresses)
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
                            &self.data.limits,
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

impl TryFrom<&Requirement> for Erc1155Requirement {
    type Error = CheckableError;

    fn try_from(req: &Requirement) -> Result<Self, Self::Error> {
        match req.chain {
            Some(chain) => {
                if PROVIDERS.get(&(chain as u8)).is_none() {
                    return Err(CheckableError::NoSuchChain(
                        CheckableError::NoSuchChain(format!("{chain:?}")).to_string(),
                    ));
                }

                let Some(data) = &req.data else {
                    return Err(CheckableError::NoSuchChain(
                        CheckableError::MissingField("data".into()).to_string(),
                    ));
                };

                if data.id.is_none() {
                    return Err(CheckableError::NoSuchChain(
                        CheckableError::MissingField("id".into()).to_string(),
                    ));
                };

                match req.address {
                    Some(address) => {
                        let res = Erc1155Requirement {
                            id: req.id,
                            address,
                            data: NftData {
                                id: data.id,
                                limits: AmountLimits::from_req(req),
                            },
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
        requirements::{
            general::token::nft::erc1155::{Erc1155Requirement, NftData},
            Checkable,
        },
        types::{AmountLimits, EvmChain, User, U256},
    };

    #[tokio::test]
    async fn erc1155_check() {
        let users_1 = vec![User {
            id: 0,
            addresses: vec![address!("0x283d678711daa088640c86a1ad3f12c00ec1252e")],
            platform_users: None,
        }];

        let users_2 = vec![User {
            id: 0,
            addresses: vec![address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")],
            platform_users: None,
        }];

        let req = Erc1155Requirement {
            id: 0,
            chain: EvmChain::Ethereum,
            address: address!("0x76be3b62873462d2142405439777e971754e8e77"),
            data: NftData {
                id: Some(U256::from_dec_str("10527").unwrap()),
                limits: Some(AmountLimits {
                    min_amount: Some(3.0),
                    max_amount: None,
                }),
            },
        };

        assert_eq!(
            req.check(&users_1)
                .await
                .iter()
                .map(|a| {
                    a.amount;
                    a.access.unwrap_or_default()
                })
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
