mod erc20;
mod nft;

pub use erc20::Erc20Requirement;
pub use nft::*;

pub const ERC_ABI: &[u8] = include_bytes!("../../../../../abi/TOKEN.json");
