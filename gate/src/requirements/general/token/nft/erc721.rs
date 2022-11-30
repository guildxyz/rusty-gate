use crate::{
    providers::PROVIDERS,
    requirements::{
        errors::CheckableError,
        general::token::{nft::NftData, ERC721_ABI},
        utils::check_if_in_range,
        Checkable,
    },
    types::{
        Address, Amount, AmountLimits, Chain, NumberId, ReqUserAccess, Requirement, User,
        UserAddress, U256,
    },
};
use async_trait::async_trait;
use web3::contract::Options;

pub struct Erc721Requirement {
    id: NumberId,
    address: Address,
    data: NftData,
    chain: Chain,
}

#[async_trait]
impl Checkable for Erc721Requirement {
    async fn check(&self, users: &[User]) -> Vec<ReqUserAccess> {
        let Some(provider) = PROVIDERS.get(&(self.chain as u8)) else {
            panic!();
        };

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

        let contract: &'static _ = Box::leak(Box::new(
            web3::contract::Contract::from_json(provider.single.eth(), self.address, ERC721_ABI)
                .unwrap(),
        ));

        futures::future::join_all(user_addresses.iter().map(|ua| async move {
            let mut error = None;
            let mut amount = None;

            let response: Result<U256, web3::contract::Error> = match self.data.id {
                Some(id) => {
                    let owner_res: Result<Address, web3::contract::Error> = contract
                        .clone()
                        .query("ownerOf", (id,), None, Options::default(), None)
                        .await;

                    let res = match owner_res {
                        Ok(owner) => if owner == ua.address { 1 } else { 0 }.into(),
                        Err(_) => 0.into(),
                    };

                    Ok(res)
                }
                None => {
                    contract
                        .clone()
                        .query("balanceOf", (ua.address,), None, Options::default(), None)
                        .await
                }
            };

            match response {
                Ok(r) => amount = Some(r.as_u128() as Amount),
                Err(e) => error = Some(e.to_string()),
            }

            ReqUserAccess {
                requirement_id: self.id,
                user_id: ua.user_id,
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
        }))
        .await
    }
}

impl TryFrom<&Requirement> for Erc721Requirement {
    type Error = CheckableError;

    fn try_from(req: &Requirement) -> Result<Self, Self::Error> {
        match req.chain {
            Some(chain) => {
                if PROVIDERS.get(&(chain as u8)).is_none() {
                    return Err(CheckableError::NoSuchChain(
                        CheckableError::NoSuchChain(format!("{:?}", chain)).to_string(),
                    ));
                }

                let Some(data) = &req.data else {
                    return Err(CheckableError::NoSuchChain(
                        CheckableError::MissingField("data".into()).to_string(),
                    ));
                };

                match req.address {
                    Some(address) => {
                        let res = Erc721Requirement {
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
            general::token::nft::erc721::{Erc721Requirement, NftData},
            Checkable,
        },
        types::{Chain, User, U256},
    };

    #[tokio::test]
    async fn erc721_check() {
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

        let req = Erc721Requirement {
            id: 0,
            chain: Chain::Ethereum,
            address: address!("0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85"),
            data: NftData {
                id: Some(U256::from_dec_str(
                    "61313325075603536901663283754390960556726744542208800735045237225934362163454",
                )
                .unwrap()),
                limits: None
            },
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
