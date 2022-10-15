use crate::{
    requirements::{errors::CheckableError, Checkable},
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
                access: Some(true),
                amount: Some(1.0),
                warning: None,
                error: None,
            })
            .collect();

        Ok(res)
    }
}

impl TryFrom<&Requirement> for FreeRequirement {
    type Error = CheckableError;

    fn try_from(req: &Requirement) -> Result<Self, Self::Error> {
        Ok(FreeRequirement { id: req.id })
    }
}
