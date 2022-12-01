mod erc20;
mod nft;

pub use erc20::Erc20Requirement;
pub use nft::*;

pub const ERC20_ABI: &[u8] = include_bytes!("../../../../../abi/ERC20.json");
pub const ERC721_ABI: &[u8] = include_bytes!("../../../../../abi/ERC721.json");
pub const ERC1155_ABI: &[u8] = include_bytes!("../../../../../abi/ERC1155.json");
