use crate::{
    requirements::Checkable,
    types::{NumberId, ReqUserAccess, Requirement, User, UserAddress},
};
use anyhow::Result;
use async_trait::async_trait;

pub struct FreeRequirement {
    id: NumberId,
}

#[async_trait]
impl Checkable for FreeRequirement {
    async fn check(&self, users: &[User]) -> Result<Vec<ReqUserAccess>> {
        let res = users
            .iter()
            .flat_map(|u| {
                u.addresses.iter().map(|a| UserAddress {
                    user_id: u.id,
                    address: a.into(),
                })
            })
            .map(|ua| ReqUserAccess {
                requirement_id: self.id,
                user_id: ua.user_id,
                access: true,
                amount: 1.0,
            })
            .collect();

        Ok(res)
    }
}

impl From<&Requirement> for FreeRequirement {
    fn from(req: &Requirement) -> Self {
        FreeRequirement { id: req.id }
    }
}
