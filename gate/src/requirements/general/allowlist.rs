use crate::{
    requirements::{errors::CheckableError, Checkable},
    types::{Address, Amount, NumberId, ReqUserAccess, Requirement, User, UserAddress},
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
            .flat_map(|u| {
                u.addresses.iter().map(|a| UserAddress {
                    user_id: u.id,
                    address: a.into(),
                })
            })
            .map(|ua| {
                let access = self.data.addresses.contains(&ua.address.to_lowercase());

                ReqUserAccess {
                    requirement_id: self.id,
                    user_id: ua.user_id,
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
