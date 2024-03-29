use crate::types::{
    Access, CheckAccessResult, DetailedAccess, NumberId, ReqUserAccess, Requirement,
    RequirementError, User,
};
use async_trait::async_trait;
use requiem::LogicTree;
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};

pub mod errors;
pub mod general;
mod utils;

#[async_trait]
pub trait Checkable {
    async fn check(&self, users: &[User]) -> Vec<ReqUserAccess>;
}

pub async fn check_access(
    users: &[User],
    requirements: &[Requirement],
    logic: &str,
    send_details: bool,
) -> CheckAccessResult {
    let req_errors = Arc::new(RwLock::new(vec![]));
    let warning_for_user = Arc::new(RwLock::new(
        HashMap::<NumberId, Vec<RequirementError>>::new(),
    ));
    let error_for_user = Arc::new(RwLock::new(
        HashMap::<NumberId, Vec<RequirementError>>::new(),
    ));
    let acc_per_req = Arc::new(RwLock::new(Vec::<Vec<ReqUserAccess>>::new()));
    let user_ids = users.iter().map(|user| user.id);

    futures::future::join_all(requirements.iter().map(|req| async {
        let req_errors = Arc::clone(&req_errors);

        let accesses = match req.inner() {
            Ok(checkable) => checkable.check(users).await,
            Err(e) => {
                req_errors.write().unwrap().push(RequirementError {
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
                        error: Some(e.to_string()),
                    })
                    .collect()
            }
        };

        if send_details {
            // Calling unwrap is fine here, read the documentation of the
            // write function for details.
            acc_per_req.write().unwrap().push(accesses.clone());
        }

        for a in accesses.iter() {
            if let Some(warning) = &a.warning {
                let warnings = Arc::clone(&warning_for_user);

                let mut user_warnings = match warnings.read().unwrap().get(&a.user_id) {
                    Some(v) => v.clone(),
                    None => vec![],
                };

                user_warnings.push(RequirementError {
                    requirement_id: req.id,
                    msg: warning.clone(),
                });

                warnings
                    .write()
                    .unwrap()
                    .insert(a.user_id, user_warnings.clone());
            }

            if let Some(error) = &a.error {
                let errors = Arc::clone(&error_for_user);

                let mut user_errors = match errors.read().unwrap().get(&a.user_id) {
                    Some(v) => v.clone(),
                    None => vec![],
                };

                user_errors.push(RequirementError {
                    requirement_id: req.id,
                    msg: error.clone(),
                });

                errors
                    .write()
                    .unwrap()
                    .insert(a.user_id, user_errors.clone());
            }
        }
    }))
    .await;

    // Calling unwrap is fine here, read the documentation of the
    // lock function for details.
    let req_errors = req_errors.read().unwrap();

    CheckAccessResult {
        accesses: user_ids
            .map(|id| {
                let acc_per_req = Arc::clone(&acc_per_req);
                let has_access = match LogicTree::from_str(logic) {
                    Ok(tree) => {
                        let mut terminals = HashMap::new();
                        let mut error = false;

                        for (idx, value) in acc_per_req
                            .read()
                            .unwrap()
                            .iter()
                            .map(|req_accesses| {
                                req_accesses
                                    .iter()
                                    .find(|a| a.user_id == id)
                                    .expect("This should be fine")
                                    .access
                            })
                            .enumerate()
                        {
                            match value {
                                Some(v) => {
                                    terminals.insert(idx as u32, v);
                                }
                                None => error = true,
                            }
                        }

                        if error {
                            None
                        } else {
                            Some(tree.evaluate(&terminals).unwrap_or(false))
                        }
                    }
                    Err(_) => None,
                };

                let access = if req_errors.is_empty()
                    && !error_for_user.read().unwrap().contains_key(&id)
                    || has_access.is_some()
                {
                    has_access
                } else {
                    None
                };

                let warnings = warning_for_user.read().unwrap().get(&id).cloned();
                let errors = error_for_user.read().unwrap().get(&id).cloned();

                let detailed = if send_details {
                    // Calling unwrap is fine here, read the documentation of
                    // the lock function for details.
                    let inner = acc_per_req
                        .read()
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
