use crate::{
    address,
    evm::{
        balancy::{types::BalancyError, BalancyProvider},
        Chain, ERC1155_ABI, ERC20_ABI, ERC721_ABI,
    },
    BalanceQuerier,
};
use async_trait::async_trait;
use futures::future::join_all;
use std::{collections::HashMap, sync::Arc};
use web3::{
    contract::{Contract, Options},
    transports::Http,
    types::{Address, U256},
    Web3,
};

type Balance = f64;

pub struct MulticallParams {
    pub rpc_url: String,
    pub address: Address,
}

pub struct Provider {
    chain: Chain,
    pub single: Web3<Http>,
    pub multi: MulticallParams,
}

impl Provider {
    pub fn new(chain: Chain, rpc_url: String, address: Address) -> Self {
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
    type Chain = Chain;
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
        let mut providers = HashMap::new();

        providers.insert(
            Chain::Ethereum as u8,
            Provider::new(
                Chain::Ethereum,
                dotenv!("ETHEREUM_RPC"),
                address!("0x5ba1e12693dc8f9c48aad8770482f4739beed696")
            )
        );
        providers.insert(
            Chain::Polygon as u8,
            Provider::new(
                Chain::Polygon,
                dotenv!("POLYGON_RPC"),
                address!("0x11ce4B23bD875D7F5C6a31084f55fDe1e9A87507")
            )
        );
        providers.insert(
            Chain::Bsc as u8,
            Provider::new(
                Chain::Bsc,
                dotenv!("BSC_RPC"),
                address!("0x41263cba59eb80dc200f3e2544eda4ed6a90e76c")
            )
        );
        providers.insert(
            Chain::Gnosis as u8,
            Provider::new(
                Chain::Gnosis,
                dotenv!("GNOSIS_RPC"),
                address!("0xb5b692a88bdfc81ca69dcb1d924f59f0413a602a")
            )
        );
        providers.insert(
            Chain::Fantom as u8,
            Provider::new(
                Chain::Fantom,
                dotenv!("FANTOM_RPC"),
                address!("0xD98e3dBE5950Ca8Ce5a4b59630a5652110403E5c")
            )
        );
        providers.insert(
            Chain::Avalanche as u8,
            Provider::new(
                Chain::Avalanche,
                dotenv!("AVALANCHE_RPC"),
                address!("0x98e2060F672FD1656a07bc12D7253b5e41bF3876")
            )
        );
        providers.insert(
            Chain::Arbitrum as u8,
            Provider::new(
                Chain::Arbitrum,
                dotenv!("ARBITRUM_RPC"),
                address!("0x52bfe8fE06c8197a8e3dCcE57cE012e13a7315EB")
            )
        );
        providers.insert(
            Chain::Celo as u8,
            Provider::new(
                Chain::Celo,
                dotenv!("CELO_RPC"),
                address!("0xb74C3A8108F1534Fc0D9b776A9B487c84fe8eD06")
            )
        );
        providers.insert(
            Chain::Harmony as u8,
            Provider::new(
                Chain::Harmony,
                dotenv!("HARMONY_RPC"),
                address!("0x34b415f4d3b332515e66f70595ace1dcf36254c5")
            )
        );
        providers.insert(
            Chain::Heco as u8,
            Provider::new(
                Chain::Heco,
                dotenv!("HECO_RPC"),
                address!("0x41C0A3059De6bE4f1913630db94d93aB5a2904B4")
            )
        );
        providers.insert(
            Chain::Goerli as u8,
            Provider::new(
                Chain::Goerli,
                dotenv!("GOERLI_RPC"),
                address!("0x77dCa2C955b15e9dE4dbBCf1246B4B85b651e50e")
            )
        );
        providers.insert(
            Chain::Optimism as u8,
            Provider::new(
                Chain::Optimism,
                dotenv!("OPTIMISM_RPC"),
                address!("0x2DC0E2aa608532Da689e89e237dF582B783E552C")
            )
        );
        providers.insert(
            Chain::Moonriver as u8,
            Provider::new(
                Chain::Moonriver,
                dotenv!("MOONRIVER_RPC"),
                address!("0x270f2F35bED92B7A59eA5F08F6B3fd34c8D9D9b5")
            )
        );
        providers.insert(
            Chain::Rinkeby as u8,
            Provider::new(
                Chain::Rinkeby,
                dotenv!("RINKEBY_RPC"),
                address!("0x5ba1e12693dc8f9c48aad8770482f4739beed696")
            )
        );
        providers.insert(
            Chain::Metis as u8,
            Provider::new(
                Chain::Metis,
                dotenv!("METIS_RPC"),
                address!("0x1a2AFb22B8A90A77a93e80ceA61f89D04e05b796")
            )
        );
        providers.insert(
            Chain::Cronos as u8,
            Provider::new(
                Chain::Cronos,
                dotenv!("CRONOS_RPC"),
                address!("0x0fA4d452693F2f45D28c4EC4d20b236C4010dA74")
            )
        );
        providers.insert(
            Chain::Boba as u8,
            Provider::new(
                Chain::Boba,
                dotenv!("BOBA_RPC"),
                address!("0xbe2Be647F8aC42808E67431B4E1D6c19796bF586")
            )
        );
        providers.insert(
            Chain::Palm as u8,
            Provider::new(
                Chain::Palm,
                dotenv!("PALM_RPC"),
                address!("0xfFE2FF36c5b8D948f788a34f867784828aa7415D")
            )
        );

        providers
    });
}
