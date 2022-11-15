use super::{Address, U256};
use serde::Deserialize;
use serde_aux::prelude::*;

pub enum BalancyChain {
    Ethereum = 1,
    Bsc = 56,
    Gnosis = 100,
    Polygon = 137,
}

pub enum TokenType {
    Native,
    Erc20 { address: Address },
    Erc721 { address: Address, id: U256 },
    Erc1155 { address: Address, id: U256 },
}

#[derive(Deserialize, Debug)]
pub struct Erc20 {
    pub address: Address,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub amount: U256,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Erc721 {
    pub address: Address,
    pub token_id: U256,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Erc1155 {
    pub addr: Address,
    pub token_id: U256,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub amount: U256,
}

#[derive(Deserialize, Debug)]
pub struct BalancyResponse {
    pub erc20: Vec<Erc20>,
    pub erc721: Vec<Erc721>,
    pub erc1155: Vec<Erc1155>,
}
