use self::errors::CheckableError;
use crate::{
    requirements::general::{
        allowlist::AllowListRequirement, coin::CoinRequirement, free::FreeRequirement,
    },
    types::{
        Access, CheckAccessResult, DetailedAccess, Logic, NumberId, ReqUserAccess, Requirement,
        RequirementError, RequirementType, User,
    },
};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashSet;

mod errors;
mod general;
mod utils;

#[async_trait]
pub trait Checkable {
    async fn check(&self, users: &[User]) -> Result<Vec<ReqUserAccess>>;
}

impl Requirement {
    pub fn inner(&self) -> Result<Box<dyn Checkable>, CheckableError> {
        use RequirementType::*;

        Ok(match self.typ {
            Free => Box::new(FreeRequirement::try_from(self)?),
            Allowlist => Box::new(AllowListRequirement::try_from(self)?),
            Coin => Box::new(CoinRequirement::try_from(self)?),
        })
    }
}

fn logic_gate(
    has_access_users_per_requirement: Vec<HashSet<NumberId>>,
    logic: Logic,
) -> HashSet<NumberId> {
    let mut has_access_users = has_access_users_per_requirement[0].clone();

    for set in has_access_users_per_requirement.iter().skip(1) {
        match logic {
            Logic::Or | Logic::Nor => has_access_users.extend(set),
            Logic::And | Logic::Nand => {
                has_access_users = has_access_users
                    .iter()
                    .copied()
                    .filter(|x| set.contains(x))
                    .collect()
            }
        }
    }

    has_access_users
}

pub async fn check_access(
    users: &[User],
    requirements: &[Requirement],
    logic: Logic,
    send_details: bool,
) -> CheckAccessResult {
    use std::sync::Mutex;
    type StatMut<T> = &'static Mutex<T>;

    let requirement_errors: StatMut<Vec<RequirementError>> =
        Box::leak(Box::new(Mutex::new(vec![])));
    let accesses_per_requirement: StatMut<Vec<Vec<ReqUserAccess>>> =
        Box::leak(Box::new(Mutex::new(vec![])));
    let user_ids = users.iter().map(|user| user.id);

    let has_access_users_per_requirement =
        futures::future::join_all(requirements.iter().map(move |req| async move {
            let accesses = match req.inner() {
                Ok(checkable) => checkable.check(users).await.unwrap(),
                Err(e) => {
                    requirement_errors.lock().unwrap().push(RequirementError {
                        requirement_id: req.id,
                        msg: e.to_string(),
                    });

                    users
                        .iter()
                        .map(|u| ReqUserAccess {
                            requirement_id: req.id,
                            user_id: u.id,
                            access: false,
                            amount: 0.0,
                        })
                        .collect()
                }
            };

            let mut has_access_users_of_requirement = HashSet::<NumberId>::new();

            if send_details {
                // Calling unwrap is fine here, read the documentation of the
                // lock function for details.
                accesses_per_requirement.lock().unwrap().push(
                    accesses
                        .iter()
                        .map(|a| ReqUserAccess {
                            requirement_id: req.id,
                            ..*a
                        })
                        .collect(),
                );
            }

            for a in accesses.iter() {
                if a.access {
                    has_access_users_of_requirement.insert(a.user_id);
                }
            }

            has_access_users_of_requirement
        }))
        .await;

    let has_access_users = logic_gate(has_access_users_per_requirement, logic);

    let ngate = logic == Logic::Nand || logic == Logic::Nor;

    CheckAccessResult {
        accesses: user_ids
            .map(|id| {
                let access = if ngate {
                    !has_access_users.contains(&id)
                } else {
                    has_access_users.contains(&id)
                };

                let detailed = if send_details {
                    // Calling unwrap is fine here, read the documentation of
                    // the lock function for details.
                    let inner = accesses_per_requirement.lock().unwrap()
                        .iter()
                        .map(|reqs| {
                            let mut filtered =
                                reqs.iter().filter(|user| user.user_id == id);

                            let access = filtered
                                .clone()
                                .map(|f| f.access)
                                .reduce(|a, b| a || b)
                                .unwrap_or_default();

                            let amount = filtered
                                .clone()
                                .map(|f| f.amount)
                                .reduce(|a, b| a + b)
                                .unwrap_or_default();

                            let requirement_id = filtered
                                .next()
                                .expect("Unwrapping the first element of a non-empty vector should be fine")
                                .requirement_id;

                            DetailedAccess {
                                requirement_id,
                                access,
                                amount,
                            }
                        })
                        .collect();

                    Some(inner)
                } else {
                    None
                };

                Access {
                    id,
                    access,
                    detailed,
                }
            })
            .collect(),
        errors: {
            // Calling unwrap is fine here, read the documentation of the
            // lock function for details.
            let req_errors = requirement_errors.lock().unwrap();

            if !req_errors.is_empty() {
                Some(req_errors.to_vec())
            } else {
                None
            }
        },
    }
}
