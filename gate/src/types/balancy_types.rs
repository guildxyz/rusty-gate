use super::{Address, U256};
use serde::Deserialize;
use serde_aux::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BalancyError {
    #[error("Chain `{0}` is not supported by Balancy")]
    ChainNotSupported(String),
    #[error("User doesn't have token associated with address `{0}`")]
    NoSuchTokenInWallet(Address),
    #[error("{0}")]
    RequestFailed(#[from] reqwest::Error),
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
    pub token_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Erc1155 {
    pub addr: Address,
    pub token_id: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub amount: U256,
}

#[derive(Deserialize, Debug)]
pub struct AddressTokenResponse {
    pub erc20: Vec<Erc20>,
    pub erc721: Vec<Erc721>,
    pub erc1155: Vec<Erc1155>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub token_address: Address,
    pub amount: U256,
}

#[derive(Deserialize, Debug)]
pub struct Erc20Response {
    pub result: Vec<TokenAmount>,
}
