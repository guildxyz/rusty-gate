pub mod types;

use crate::{
    evm::{
        balancy::types::{AddressTokenResponse, BalancyError},
        EvmChain,
    },
    Address, U256,
};
use reqwest::StatusCode;
use std::collections::HashMap;
use tokio::sync::RwLock;

// Balancy
const BASE_URL: &str = "https://balancy.guild.xyz/api";
const ADDRESS_TOKENS: &str = "addressTokens?address=";
const BALANCY_CHAIN: &str = "&chain=";

lazy_static::lazy_static! {
    static ref CLIENT: RwLock<reqwest::Client> =
        RwLock::new(reqwest::Client::new());
    static ref CHAIN_IDS: HashMap<u32, u32> = {
        let mut h = HashMap::new();

        h.insert(EvmChain::Ethereum as u32, 1);
        h.insert(EvmChain::Bsc as u32, 56);
        h.insert(EvmChain::Gnosis as u32, 100);
        h.insert(EvmChain::Polygon as u32, 137);

        h
    };
}

pub async fn get_address_tokens(
    chain: EvmChain,
    address: Address,
) -> Result<AddressTokenResponse, BalancyError> {
    match CHAIN_IDS.get(&(chain as u32)) {
        None => Err(BalancyError::ChainNotSupported(format!("{:?}", chain))),
        Some(id) => {
            let res = CLIENT
                .read()
                .await
                .get(format!(
                    "{BASE_URL}/{ADDRESS_TOKENS}{:#x}{BALANCY_CHAIN}{id}",
                    address
                ))
                .send()
                .await?;

            let status = res.status();

            match status {
                StatusCode::OK => Ok(res.json::<AddressTokenResponse>().await?),
                StatusCode::BAD_REQUEST => Err(BalancyError::InvalidBalancyRequest),
                StatusCode::TOO_MANY_REQUESTS => Err(BalancyError::TooManyRequests),
                _ => Err(BalancyError::Unknown(status.as_u16())),
            }
        }
    }
}

pub struct BalancyProvider;

impl BalancyProvider {
    pub async fn get_total_erc1155_of_address(
        chain: EvmChain,
        token_address: Address,
        user_address: Address,
    ) -> Result<U256, BalancyError> {
        let body = get_address_tokens(chain, user_address).await?;

        let amount = body
            .erc1155
            .iter()
            .filter(|i| i.addr == token_address)
            .map(|token| token.amount)
            .reduce(|a, b| a + b)
            .unwrap_or_default();

        Ok(amount)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        address,
        evm::balancy::{get_address_tokens, BalancyProvider},
        evm::EvmChain,
    };
    use web3::types::U256;

    #[tokio::test]
    async fn balancy_address_tokens() {
        assert!(get_address_tokens(
            EvmChain::Ethereum,
            address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn balancy_total_erc1155_of_address() {
        assert_eq!(
            BalancyProvider::get_total_erc1155_of_address(
                EvmChain::Ethereum,
                address!("0x76be3b62873462d2142405439777e971754e8e77"),
                address!("0x283d678711daa088640c86a1ad3f12c00ec1252e")
            )
            .await
            .unwrap(),
            U256::from_dec_str("8110").unwrap()
        );
    }
}
