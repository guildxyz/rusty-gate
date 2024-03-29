use crate::{evm::u256_from_str, Address, U256};
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BalancyError {
    #[error("Chain `{0}` is not supported by Balancy")]
    ChainNotSupported(String),
    #[error("User doesn't have token associated with address `{0}`")]
    NoSuchTokenInWallet(Address),
    #[error("Invalid Balancy request")]
    InvalidBalancyRequest,
    #[error("Too many requests to Balancy")]
    TooManyRequests,
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("Got response with status code `{0}`")]
    Unknown(u16),
}

#[derive(Deserialize, Debug)]
pub struct Erc20 {
    pub address: Address,
    #[serde(deserialize_with = "u256_from_str")]
    pub amount: U256,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Erc721 {
    pub address: Address,
    #[serde(deserialize_with = "u256_from_str")]
    pub token_id: U256,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Erc1155 {
    pub addr: Address,
    #[serde(deserialize_with = "u256_from_str")]
    pub token_id: U256,
    #[serde(deserialize_with = "u256_from_str")]
    pub amount: U256,
}

#[derive(Deserialize, Debug)]
pub struct AddressTokenResponse {
    pub erc20: Vec<Erc20>,
    pub erc721: Vec<Erc721>,
    pub erc1155: Vec<Erc1155>,
}
