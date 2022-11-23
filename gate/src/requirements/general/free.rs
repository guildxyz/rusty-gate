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
    async fn check(&self, users: &[User]) -> Vec<ReqUserAccess> {
        users
            .iter()
            .flat_map(|u| {
                u.addresses.iter().cloned().map(|address| UserAddress {
                    user_id: u.id,
                    address,
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
            .collect()
    }
}

impl TryFrom<&Requirement> for FreeRequirement {
    type Error = CheckableError;

    fn try_from(req: &Requirement) -> Result<Self, Self::Error> {
        Ok(FreeRequirement { id: req.id })
    }
}

#[cfg(test)]
mod test {
    use super::FreeRequirement;
    use crate::{address, requirements::Checkable, types::User};

    #[tokio::test]
    async fn check() {
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

        let req = FreeRequirement { id: 0 };

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
            vec![false]
        );
    }
}
