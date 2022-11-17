use crate::types::{Address, AddressTokenResponse, BalancyError, Chain};
use std::collections::HashMap;

// Balancy
const BASE_URL: &str = "https://balancy.guild.xyz/api";
const ADDRESS_TOKENS: &str = "addressTokens?address=";
const BALANCY_CHAIN: &str = "&chain=";

lazy_static::lazy_static! {
    static ref CHAIN_IDS: HashMap<u32, u32> = {
        let mut h = HashMap::new();

        h.insert(Chain::Ethereum as u32, 1);
        h.insert(Chain::Bsc as u32, 56);
        h.insert(Chain::Gnosis as u32, 100);
        h.insert(Chain::Polygon as u32, 137);

        h
    };
}

#[allow(dead_code)]
pub async fn get_address_tokens(
    chain: Chain,
    address: Address,
) -> Result<AddressTokenResponse, BalancyError> {
    match CHAIN_IDS.get(&(chain as u32)) {
        None => return Err(BalancyError::ChainNotSupported(format!("{:?}", chain))),
        Some(id) => {
            let body: AddressTokenResponse = reqwest::get(format!(
                "{BASE_URL}/{ADDRESS_TOKENS}{:#x}{BALANCY_CHAIN}{id}",
                address
            ))
            .await?
            .json()
            .await?;

            Ok(body)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{address, providers::balancy::get_address_tokens, types::Chain};

    #[tokio::test]
    async fn balancy_address_tokens() {
        assert!(get_address_tokens(
            Chain::Ethereum,
            address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")
        )
        .await
        .is_ok());
    }
}
