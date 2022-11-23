use crate::types::{Amount, AmountLimits};

pub fn check_if_in_range(amount: Amount, limits: &Option<AmountLimits>, equal_max: bool) -> bool {
    match limits {
        Some(limits) => {
            let min_amount = limits.min_amount.unwrap_or_default();
            let min_ok = if min_amount > 0.0 {
                amount >= min_amount
            } else {
                amount > 0.0
            };

            match limits.max_amount {
                Some(max_amount) => {
                    let max_ok = if equal_max {
                        amount <= max_amount
                    } else {
                        amount < max_amount
                    };

                    min_ok && max_ok
                }
                None => min_ok,
            }
        }

        None => amount > 0.0,
    }
}
