use crate::{
    address,
    evm::{
        balancy::{types::BalancyError, BalancyProvider},
        EvmChain, ERC1155_ABI, ERC20_ABI, ERC721_ABI,
    },
    Address, BalanceQuerier, U256,
};
use async_trait::async_trait;
use futures::future::join_all;
use std::{collections::HashMap, sync::Arc};
use web3::{
    contract::{Contract, Options},
    transports::Http,
    Web3,
};

type Balance = f64;

pub struct MulticallParams {
    pub rpc_url: String,
    pub address: Address,
}

pub struct Provider {
    chain: EvmChain,
    pub single: Web3<Http>,
    pub multi: MulticallParams,
}

impl Provider {
    pub fn new(chain: EvmChain, rpc_url: String, address: Address) -> Self {
        Self {
            chain,
            single: match Http::new(&rpc_url) {
                Ok(transport) => Web3::new(transport),
                Err(e) => panic!("{e}"),
            },
            multi: MulticallParams { rpc_url, address },
        }
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error(transparent)]
    Balancy(#[from] BalancyError),
    #[error(transparent)]
    Web3Contract(#[from] web3::contract::Error),
    #[error(transparent)]
    Web3(#[from] web3::Error),
    #[error("{0}")]
    Other(String),
}

const DECIMALS: u32 = 18;
const DIVISOR: Balance = 10_u128.pow(DECIMALS) as Balance;

#[async_trait]
impl BalanceQuerier for Provider {
    type Address = Address;
    type Id = U256;
    type Balance = Balance;
    type Chain = EvmChain;
    type Error = ProviderError;

    async fn get_native_balance(
        &self,
        user_addresses: &[Self::Address],
    ) -> Vec<Result<Self::Balance, Self::Error>> {
        join_all(user_addresses.iter().map(|ua| async {
            self.single
                .eth()
                .balance(*ua, None)
                .await
                .map_err(ProviderError::Web3)
                .map(|v| v.as_u128() as Balance / DIVISOR)
        }))
        .await
    }

    async fn get_fungible_balance(
        &self,
        token_address: Self::Address,
        user_addresses: &[Self::Address],
    ) -> Vec<Result<Self::Balance, Self::Error>> {
        let contract =
            Arc::new(Contract::from_json(self.single.eth(), token_address, ERC20_ABI).unwrap());

        let decimals: u8 = contract
            .query("decimals", (), None, Options::default(), None)
            .await
            .unwrap();

        join_all(user_addresses.iter().map(|ua| async {
            let contract = Arc::clone(&contract);

            contract
                .query("balanceOf", (*ua,), None, Options::default(), None)
                .await
                .map_err(ProviderError::Web3Contract)
                .map(|v: U256| v.as_u128() as Balance / 10_u128.pow(decimals as u32) as Balance)
        }))
        .await
    }

    async fn get_non_fungible_balance(
        &self,
        token_address: Self::Address,
        token_id: Option<Self::Id>,
        user_addresses: &[Self::Address],
    ) -> Vec<Result<Self::Balance, Self::Error>> {
        let contract =
            Arc::new(Contract::from_json(self.single.eth(), token_address, ERC721_ABI).unwrap());

        join_all(user_addresses.iter().map(|ua| async {
            let contract = Arc::clone(&contract);

            let response: Result<U256, web3::contract::Error> = match token_id {
                Some(id) => {
                    let owner_res: Result<Address, web3::contract::Error> = contract
                        .clone()
                        .query("ownerOf", (id,), None, Options::default(), None)
                        .await;

                    let res = match owner_res {
                        Ok(owner) => i32::from(owner == *ua).into(),
                        Err(_) => 0.into(),
                    };

                    Ok(res)
                }
                None => {
                    contract
                        .clone()
                        .query("balanceOf", (*ua,), None, Options::default(), None)
                        .await
                }
            };

            response
                .map_err(ProviderError::Web3Contract)
                .map(|v: U256| v.as_u128() as Balance)
        }))
        .await
    }

    async fn get_special_balance(
        &self,
        token_address: Self::Address,
        token_id: Option<Self::Id>,
        user_addresses: &[Self::Address],
    ) -> Vec<Result<Self::Balance, Self::Error>> {
        let contract = Contract::from_json(self.single.eth(), token_address, ERC1155_ABI).unwrap();

        match token_id {
            Some(id) => {
                let balances: Result<Vec<U256>, web3::contract::Error> = contract
                    .clone()
                    .query(
                        "balanceOfBatch",
                        (user_addresses.to_vec(), vec![id]),
                        None,
                        Options::default(),
                        None,
                    )
                    .await;

                match balances {
                    Ok(balances) => balances
                        .iter()
                        .map(|b| Ok(b.as_u128() as Balance))
                        .collect(),
                    Err(e) => user_addresses
                        .iter()
                        .map(|_| Err(ProviderError::Other(e.to_string())))
                        .collect(),
                }
            }
            None => {
                join_all(user_addresses.iter().map(|ua| async {
                    let response = BalancyProvider::get_total_erc1155_of_address(
                        self.chain,
                        token_address,
                        *ua,
                    )
                    .await;

                    response
                        .map_err(ProviderError::Balancy)
                        .map(|v: U256| v.as_u128() as Balance)
                }))
                .await
            }
        }
    }
}

macro_rules! dotenv {
    ($var: expr) => {
        match std::env::var($var) {
            Ok(val) => val,
            Err(_) => panic!("Environment variable `{}` not found", $var),
        }
    };
}

lazy_static::lazy_static! {
    pub static ref PROVIDERS: Arc<HashMap<u8, Provider>> = Arc::new({
        dotenv::dotenv().ok();

        let mut providers = HashMap::new();

        providers.insert(
            EvmChain::Ethereum as u8,
            Provider::new(
                EvmChain::Ethereum,
                dotenv!("ETHEREUM_RPC"),
                address!("0x5ba1e12693dc8f9c48aad8770482f4739beed696")
            )
        );
        providers.insert(
            EvmChain::Polygon as u8,
            Provider::new(
                EvmChain::Polygon,
                dotenv!("POLYGON_RPC"),
                address!("0x11ce4B23bD875D7F5C6a31084f55fDe1e9A87507")
            )
        );
        providers.insert(
            EvmChain::Bsc as u8,
            Provider::new(
                EvmChain::Bsc,
                dotenv!("BSC_RPC"),
                address!("0x41263cba59eb80dc200f3e2544eda4ed6a90e76c")
            )
        );
        providers.insert(
            EvmChain::Gnosis as u8,
            Provider::new(
                EvmChain::Gnosis,
                dotenv!("GNOSIS_RPC"),
                address!("0xb5b692a88bdfc81ca69dcb1d924f59f0413a602a")
            )
        );
        providers.insert(
            EvmChain::Arbitrum as u8,
            Provider::new(
                EvmChain::Arbitrum,
                dotenv!("ARBITRUM_RPC"),
                address!("0x52bfe8fE06c8197a8e3dCcE57cE012e13a7315EB")
            )
        );
        providers.insert(
            EvmChain::Goerli as u8,
            Provider::new(
                EvmChain::Goerli,
                dotenv!("GOERLI_RPC"),
                address!("0x77dCa2C955b15e9dE4dbBCf1246B4B85b651e50e")
            )
        );
        providers
    });
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn eth_balance() {
        let provider = PROVIDERS.get(&(EvmChain::Ethereum as u8)).unwrap();
        let maybe_balances = provider
            .get_native_balance(&[address!("0x0000000000000000000000000000000000000000")])
            .await;
        let Some(Ok(balance)) = maybe_balances.get(0) else {
            panic!("should be ok")
        };

        assert!(balance > &0.0)
    }

    #[tokio::test]
    async fn polygon_balance() {
        let provider = PROVIDERS.get(&(EvmChain::Polygon as u8)).unwrap();
        let maybe_balances = provider
            .get_native_balance(&[address!("0x0000000000000000000000000000000000000000")])
            .await;
        let Some(Ok(balance)) = maybe_balances.get(0) else {
            panic!("should be ok")
        };

        assert!(balance > &0.0)
    }

    #[tokio::test]
    async fn bsc_balance() {
        let provider = PROVIDERS.get(&(EvmChain::Bsc as u8)).unwrap();
        let maybe_balances = provider
            .get_native_balance(&[address!("0x0000000000000000000000000000000000000000")])
            .await;
        let Some(Ok(balance)) = maybe_balances.get(0) else {
            panic!("should be ok")
        };

        assert!(balance > &0.0)
    }

    #[tokio::test]
    async fn arbitrum_balance() {
        let provider = PROVIDERS.get(&(EvmChain::Arbitrum as u8)).unwrap();
        let maybe_balances = provider
            .get_native_balance(&[address!("0x0000000000000000000000000000000000000000")])
            .await;
        let Some(Ok(balance)) = maybe_balances.get(0) else {
            panic!("should be ok")
        };

        assert!(balance > &0.0)
    }

    #[tokio::test]
    async fn gnosis_balance() {
        let provider = PROVIDERS.get(&(EvmChain::Gnosis as u8)).unwrap();
        let maybe_balances = provider
            .get_native_balance(&[address!("0x0000000000000000000000000000000000000000")])
            .await;
        let Some(Ok(balance)) = maybe_balances.get(0) else {
            panic!("should be ok")
        };

        assert!(balance > &0.0)
    }

    #[tokio::test]
    async fn goerli_balance() {
        let provider = PROVIDERS.get(&(EvmChain::Goerli as u8)).unwrap();
        let maybe_balances = provider
            .get_native_balance(&[address!("0x0000000000000000000000000000000000000000")])
            .await;
        let Some(Ok(balance)) = maybe_balances.get(0) else {
            panic!("should be ok")
        };

        assert!(balance > &0.0)
    }
}
