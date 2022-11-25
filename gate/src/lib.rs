#![deny(clippy::dbg_macro)]

pub mod providers;
pub mod requirements;
pub mod types;
mod utils;

#[macro_export]
macro_rules! address {
    ($addr:expr) => {{
        use std::str::FromStr;
        $crate::types::Address::from_str($addr).expect(&format!("Invalid address {}", $addr))
    }};
}
