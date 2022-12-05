use crate::{
    requirements::{errors::CheckableError, Checkable},
    types::{Address, Amount, NumberId, ReqUserAccess, Requirement, User},
};
use anyhow::Result;
use async_trait::async_trait;

struct AllowlistData {
    addresses: Vec<Address>,
}

pub struct AllowListRequirement {
    id: NumberId,
    data: AllowlistData,
}

#[async_trait]
impl Checkable for AllowListRequirement {
    async fn check(&self, users: &[User]) -> Vec<ReqUserAccess> {
        users
            .iter()
            .flat_map(|u| u.addresses.iter().cloned().map(|address| (u.id, address)))
            .map(|(user_id, address)| {
                let access = self.data.addresses.contains(&address);

                ReqUserAccess {
                    requirement_id: self.id,
                    user_id,
                    access: Some(access),
                    amount: Some(access as i8 as Amount),
                    warning: None,
                    error: None,
                }
            })
            .collect()
    }
}

impl TryFrom<&Requirement> for AllowListRequirement {
    type Error = CheckableError;

    fn try_from(req: &Requirement) -> Result<Self, Self::Error> {
        match &req.data {
            Some(data) => match &data.addresses {
                Some(addresses) => Ok(AllowListRequirement {
                    id: req.id,
                    data: AllowlistData {
                        addresses: addresses.to_vec(),
                    },
                }),
                None => Err(CheckableError::MissingField("addresses".into())),
            },
            None => Err(CheckableError::MissingField("data".into())),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        address,
        requirements::{general::allowlist::AllowListRequirement, Checkable},
        types::User,
    };

    #[tokio::test]
    async fn allowlist_check() {
        let users_1 = vec![User {
            id: 0,
            addresses: vec![address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")],
            platform_users: None,
        }];

        let users_2 = vec![User {
            id: 0,
            addresses: vec![address!("0x14ddfe8ea7ffc338015627d160ccaf99e8f16dd3")],
            platform_users: None,
        }];

        let allowlist = AllowListRequirement {
            id: 0,
            data: super::AllowlistData {
                addresses: vec![
                    address!("0xe43878ce78934fe8007748ff481f03b8ee3b97de"),
                    address!("0x20cc54c7ebc5f43b74866d839b4bd5c01bb23503"),
                ],
            },
        };

        assert_eq!(
            allowlist
                .check(&users_1)
                .await
                .iter()
                .map(|a| a.access.unwrap_or_default())
                .collect::<Vec<bool>>(),
            vec![true]
        );
        assert_ne!(
            allowlist
                .check(&users_2)
                .await
                .iter()
                .map(|a| a.access.unwrap_or_default())
                .collect::<Vec<bool>>(),
            vec![true]
        );
    }
}
