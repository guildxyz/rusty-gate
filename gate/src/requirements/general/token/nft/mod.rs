mod erc1155;
mod erc721;

use crate::types::{AmountLimits, U256};
pub use erc1155::Erc1155Requirement;
pub use erc721::Erc721Requirement;

struct NftData {
    id: Option<U256>,
    limits: Option<AmountLimits>,
}
