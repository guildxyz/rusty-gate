use crate::{
    requirements::Checkable,
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
    async fn check(&self, users: &[User]) -> Result<Vec<ReqUserAccess>> {
        let res = users
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
                    access,
                    amount: access as i8 as Amount,
                }
            })
            .collect();

        Ok(res)
    }
}

impl From<&Requirement> for AllowListRequirement {
    fn from(req: &Requirement) -> Self {
        AllowListRequirement {
            id: req.id,
            data: AllowlistData {
                addresses: req.data.as_ref().unwrap().addresses.clone().unwrap(),
            },
        }
    }
}
