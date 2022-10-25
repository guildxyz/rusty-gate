use self::errors::CheckableError;
use crate::{
    requirements::general::{
        allowlist::AllowListRequirement, coin::CoinRequirement, free::FreeRequirement,
    },
    types::{
        Access, CheckAccessResult, DetailedAccess, NumberId, ReqUserAccess, Requirement,
        RequirementError, RequirementType, User,
    },
};
use anyhow::Result;
use async_trait::async_trait;
use requiem::LogicTree;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::{Arc, Mutex},
};

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

pub async fn check_access(
    users: &[User],
    requirements: &[Requirement],
    logic: &str,
    send_details: bool,
) -> CheckAccessResult {
    let req_errors = Arc::new(Mutex::new(vec![]));
    let warning_for_user = Arc::new(Mutex::new(HashMap::<NumberId, Vec<RequirementError>>::new()));
    let error_for_user = Arc::new(Mutex::new(HashMap::<NumberId, Vec<RequirementError>>::new()));
    let acc_per_req = Arc::new(Mutex::new(Vec::<Vec<ReqUserAccess>>::new()));
    let user_ids = users.iter().map(|user| user.id);

    futures::future::join_all(requirements.iter().map(|req| async {
        let req_errors = Arc::clone(&req_errors);

        let accesses = match req.inner() {
            Ok(checkable) => checkable.check(users).await.unwrap(),
            Err(e) => {
                req_errors.lock().unwrap().push(RequirementError {
                    requirement_id: req.id,
                    msg: e.to_string(),
                });

                users
                    .iter()
                    .map(|u| ReqUserAccess {
                        requirement_id: req.id,
                        user_id: u.id,
                        access: None,
                        amount: None,
                        warning: None,
                        error: None,
                    })
                    .collect()
            }
        };

        let mut has_access_users_of_requirement = HashSet::<NumberId>::new();

        if send_details {
            // Calling unwrap is fine here, read the documentation of the
            // lock function for details.
            acc_per_req.lock().unwrap().push(accesses.clone());
        }

        for a in accesses.iter() {
            if a.access.unwrap_or_default() {
                has_access_users_of_requirement.insert(a.user_id);
            }

            if let Some(warning) = &a.warning {
                let warnings_mut = Arc::clone(&warning_for_user);
                let mut warnings = warnings_mut.lock().unwrap();

                let mut user_warnings = match warnings.get(&a.user_id) {
                    Some(v) => v.clone(),
                    None => vec![],
                };

                user_warnings.push(RequirementError {
                    requirement_id: req.id,
                    msg: warning.clone(),
                });

                warnings.remove(&a.user_id);
                warnings.insert(a.user_id, user_warnings.clone());
            }

            if let Some(error) = &a.error {
                let errors_mut = Arc::clone(&error_for_user);
                let mut errors = errors_mut.lock().unwrap();

                let mut user_errors = match errors.get(&a.user_id) {
                    Some(v) => v.clone(),
                    None => vec![],
                };

                user_errors.push(RequirementError {
                    requirement_id: req.id,
                    msg: error.clone(),
                });

                errors.insert(a.user_id, user_errors.clone());
            }
        }

        has_access_users_of_requirement
    }))
    .await;

    // Calling unwrap is fine here, read the documentation of the
    // lock function for details.
    let req_errors = req_errors.lock().unwrap();

    CheckAccessResult {
        accesses: user_ids
            .map(|id| {
                let acc_per_req = Arc::clone(&acc_per_req);
                let has_access = match LogicTree::from_str(logic) {
                    Ok(tree) => {
                        let mut terminals = HashMap::new();

                        for (idx, value) in acc_per_req
                            .lock()
                            .unwrap()
                            .iter()
                            .map(|req_accesses| {
                                req_accesses
                                    .iter()
                                    .find(|a| a.user_id == id)
                                    .unwrap()
                                    .access
                            })
                            .enumerate()
                        {
                            terminals.insert(idx as u32, value.unwrap_or(false));
                        }

                        tree.evaluate(&terminals).unwrap_or(false)
                    }
                    Err(_) => false,
                };

                let access = if req_errors.is_empty()
                    && !error_for_user.lock().unwrap().contains_key(&id)
                    || has_access
                {
                    Some(has_access)
                } else {
                    None
                };

                let warnings = warning_for_user.lock().unwrap().get(&id).cloned();
                let errors = error_for_user.lock().unwrap().get(&id).cloned();

                let detailed = if send_details {
                    // Calling unwrap is fine here, read the documentation of
                    // the lock function for details.
                    let inner = acc_per_req
                        .lock()
                        .unwrap()
                        .iter()
                        .map(|reqs| {
                            let mut filtered = reqs.iter().filter(|user| user.user_id == id);

                            let access = filtered
                                .clone()
                                .map(|f| f.access.unwrap_or_default())
                                .reduce(|a, b| a || b)
                                .unwrap_or_default();

                            let amount = filtered
                                .clone()
                                .map(|f| f.amount.unwrap_or_default())
                                .reduce(|a, b| a + b)
                                .unwrap_or_default();

                            let requirement_id =
                                filtered.next().expect("This should be fine").requirement_id;

                            DetailedAccess {
                                requirement_id,
                                access: Some(access),
                                amount: Some(amount),
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
                    warnings,
                    errors,
                    detailed,
                }
            })
            .collect(),
        errors: {
            if !req_errors.is_empty() {
                Some(req_errors.to_vec())
            } else {
                None
            }
        },
    }
}
