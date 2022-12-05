pub mod balancy;
pub mod general;

pub use balancy::BalancyProvider;
pub use general::Provider;
use serde::{de::Error, Deserialize, Deserializer};
use web3::types::U256;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Chain {
    Ethereum,
    Polygon,
    Gnosis,
    Bsc,
    Fantom,
    Avalanche,
    Heco,
    Harmony,
    Goerli,
    Arbitrum,
    Celo,
    Optimism,
    Moonriver,
    Rinkeby,
    Metis,
    Cronos,
    Boba,
    Palm,
}

pub fn u256_from_str<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    U256::from_dec_str(s).map_err(D::Error::custom)
}

#[macro_export]
macro_rules! address {
    ($addr:expr) => {{
        use std::str::FromStr;
        web3::types::Address::from_str($addr).expect(&format!("Invalid address {}", $addr))
    }};
}
