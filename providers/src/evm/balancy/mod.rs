mod types;

use crate::{
    evm::balancy::types::{AddressTokenResponse, BalancyError, TokenType},
    evm::Chain,
    BalanceQuerier,
};
use async_trait::async_trait;
use std::collections::HashMap;
use web3::types::{Address, U256};
type Balance = f64;

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

pub async fn get_address_tokens(
    chain: Chain,
    address: Address,
) -> Result<AddressTokenResponse, BalancyError> {
    match CHAIN_IDS.get(&(chain as u32)) {
        None => Err(BalancyError::ChainNotSupported(format!("{:?}", chain))),
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

pub async fn get_erc_amount(
    chain: Chain,
    user_address: Address,
    token: TokenType,
) -> Result<U256, BalancyError> {
    let body = get_address_tokens(chain, user_address).await?;

    use TokenType::*;

    let balance = match token {
        Erc20 { address } => body
            .erc20
            .iter()
            .find(|i| i.address == address)
            .map(|token| token.amount)
            .unwrap_or_default(),
        Erc721 { address, id } => u128::from(body.erc721.iter().any(|i| {
            i.address == address
                && match id {
                    Some(id) => id == i.token_id,
                    None => true,
                }
        }))
        .into(),
        Erc1155 { address, id } => body
            .erc1155
            .iter()
            .find(|i| i.addr == address && i.token_id == id)
            .map(|token| token.amount)
            .unwrap_or_default(),
        _ => todo!(),
    };

    Ok(balance)
}

pub async fn get_erc20_balance(
    chain: Chain,
    token_address: Address,
    user_address: Address,
) -> Result<Balance, BalancyError> {
    let balance = get_erc_amount(
        chain,
        user_address,
        TokenType::Erc20 {
            address: token_address,
        },
    )
    .await?;

    Ok(balance.as_u128() as Balance / (10_u128.pow(18) as Balance))
}

pub async fn get_erc721_balance(
    chain: Chain,
    token_address: Address,
    token_id: Option<U256>,
    user_address: Address,
) -> Result<Balance, BalancyError> {
    let balance = get_erc_amount(
        chain,
        user_address,
        TokenType::Erc721 {
            address: token_address,
            id: token_id,
        },
    )
    .await?;

    Ok(balance.as_u128() as Balance)
}

pub async fn get_erc1155_balance(
    chain: Chain,
    token_address: Address,
    token_id: U256,
    user_address: Address,
) -> Result<Balance, BalancyError> {
    let balance = get_erc_amount(
        chain,
        user_address,
        TokenType::Erc1155 {
            address: token_address,
            id: token_id,
        },
    )
    .await?;

    Ok(balance.as_u128() as Balance)
}

#[cfg(test)]
mod test {
    use crate::{
        address,
        evm::balancy::{
            get_address_tokens, get_erc1155_balance, get_erc20_balance, get_erc721_balance, Balance,
        },
        evm::Chain,
    };
    use web3::types::U256;

    #[tokio::test]
    async fn balancy_address_tokens() {
        assert!(get_address_tokens(
            Chain::Ethereum,
            address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn balancy_erc20() {
        assert_eq!(
            get_erc20_balance(
                Chain::Bsc,
                address!("0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56"),
                address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")
            )
            .await
            .unwrap(),
            (423157234052929992066_u128 as Balance / (10_u128.pow(18) as Balance))
        );
    }

    #[tokio::test]
    async fn balancy_erc721() {
        assert_eq!(
            get_erc721_balance(
                Chain::Ethereum,
                address!("0x57f1887a8bf19b14fc0df6fd9b2acc9af147ea85"),
                Some(U256::from_dec_str(
                    "61313325075603536901663283754390960556726744542208800735045237225934362163454"
                )
                .unwrap()),
                address!("0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE")
            )
            .await
            .unwrap(),
            1.0
        );
    }

    #[tokio::test]
    async fn balancy_erc1155() {
        assert_eq!(
            get_erc1155_balance(
                Chain::Ethereum,
                address!("0x76be3b62873462d2142405439777e971754e8e77"),
                U256::from_dec_str("10527").unwrap(),
                address!("0x283d678711daa088640c86a1ad3f12c00ec1252e")
            )
            .await
            .unwrap(),
            595.0
        );
    }
}

pub struct BalancyProvider;

#[async_trait]
impl BalanceQuerier for BalancyProvider {
    type Address = Address;
    type Id = U256;
    type Balance = Balance;
    type Chain = Chain;
    type Error = BalancyError;

    async fn get_native_balance(
        _user: Self::Address,
        _chain: Self::Chain,
    ) -> Result<Self::Balance, Self::Error> {
        todo!()
    }

    async fn get_fungible_balance(
        user_address: Self::Address,
        token_address: Self::Address,
        chain: Self::Chain,
    ) -> Result<Self::Balance, Self::Error> {
        get_erc20_balance(chain, token_address, user_address).await
    }

    async fn get_non_fungible_balance(
        user_address: Self::Address,
        token_address: Self::Address,
        token_id: Option<Self::Id>,
        chain: Self::Chain,
    ) -> Result<Self::Balance, Self::Error> {
        get_erc721_balance(chain, token_address, token_id, user_address).await
    }

    async fn get_special_balance(
        user_address: Self::Address,
        token_address: Self::Address,
        token_id: Self::Id,
        chain: Self::Chain,
    ) -> Result<Self::Balance, Self::Error> {
        get_erc1155_balance(chain, token_address, token_id, user_address).await
    }
}
