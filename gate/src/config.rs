use crate::{
    address,
    types::{Address, Chain},
};
use std::{collections::HashMap, sync::Arc};
use web3::{transports::Http, Web3};

pub struct MulticallProvider {
    pub rpc_url: String,
    pub address: Address,
}

pub struct Provider {
    pub single: Web3<Http>,
    pub multi: MulticallProvider,
}

impl Provider {
    pub fn new(rpc_url: String, address: Address) -> Self {
        Self {
            single: match Http::new(&rpc_url) {
                Ok(transport) => Web3::new(transport),
                Err(e) => panic!("{e}"),
            },
            multi: MulticallProvider { rpc_url, address },
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
                dotenv!("ETHEREUM_RPC"),
                address!("0x5ba1e12693dc8f9c48aad8770482f4739beed696")
            )
        );
        providers.insert(
            Chain::Polygon as u8,
            Provider::new(
                dotenv!("POLYGON_RPC"),
                address!("0x11ce4B23bD875D7F5C6a31084f55fDe1e9A87507")
            )
        );
        providers.insert(
            Chain::Bsc as u8,
            Provider::new(
                dotenv!("BSC_RPC"),
                address!("0x41263cba59eb80dc200f3e2544eda4ed6a90e76c")
            )
        );
        providers.insert(
            Chain::Gnosis as u8,
            Provider::new(
                dotenv!("GNOSIS_RPC"),
                address!("0xb5b692a88bdfc81ca69dcb1d924f59f0413a602a")
            )
        );
        providers.insert(
            Chain::Fantom as u8,
            Provider::new(
                dotenv!("FANTOM_RPC"),
                address!("0xD98e3dBE5950Ca8Ce5a4b59630a5652110403E5c")
            )
        );
        providers.insert(
            Chain::Avalanche as u8,
            Provider::new(
                dotenv!("AVALANCHE_RPC"),
                address!("0x98e2060F672FD1656a07bc12D7253b5e41bF3876")
            )
        );
        providers.insert(
            Chain::Arbitrum as u8,
            Provider::new(
                dotenv!("ARBITRUM_RPC"),
                address!("0x52bfe8fE06c8197a8e3dCcE57cE012e13a7315EB")
            )
        );
        providers.insert(
            Chain::Celo as u8,
            Provider::new(
                dotenv!("CELO_RPC"),
                address!("0xb74C3A8108F1534Fc0D9b776A9B487c84fe8eD06")
            )
        );
        providers.insert(
            Chain::Harmony as u8,
            Provider::new(
                dotenv!("HARMONY_RPC"),
                address!("0x34b415f4d3b332515e66f70595ace1dcf36254c5")
            )
        );
        providers.insert(
            Chain::Heco as u8,
            Provider::new(
                dotenv!("HECO_RPC"),
                address!("0x41C0A3059De6bE4f1913630db94d93aB5a2904B4")
            )
        );
        providers.insert(
            Chain::Goerli as u8,
            Provider::new(
                dotenv!("GOERLI_RPC"),
                address!("0x77dCa2C955b15e9dE4dbBCf1246B4B85b651e50e")
            )
        );
        providers.insert(
            Chain::Optimism as u8,
            Provider::new(
                dotenv!("OPTIMISM_RPC"),
                address!("0x2DC0E2aa608532Da689e89e237dF582B783E552C")
            )
        );
        providers.insert(
            Chain::Moonriver as u8,
            Provider::new(
                dotenv!("MOONRIVER_RPC"),
                address!("0x270f2F35bED92B7A59eA5F08F6B3fd34c8D9D9b5")
            )
        );
        providers.insert(
            Chain::Rinkeby as u8,
            Provider::new(
                dotenv!("RINKEBY_RPC"),
                address!("0x5ba1e12693dc8f9c48aad8770482f4739beed696")
            )
        );
        providers.insert(
            Chain::Metis as u8,
            Provider::new(
                dotenv!("METIS_RPC"),
                address!("0x1a2AFb22B8A90A77a93e80ceA61f89D04e05b796")
            )
        );
        providers.insert(
            Chain::Cronos as u8,
            Provider::new(
                dotenv!("CRONOS_RPC"),
                address!("0x0fA4d452693F2f45D28c4EC4d20b236C4010dA74")
            )
        );
        providers.insert(
            Chain::Boba as u8,
            Provider::new(
                dotenv!("BOBA_RPC"),
                address!("0xbe2Be647F8aC42808E67431B4E1D6c19796bF586")
            )
        );
        providers.insert(
            Chain::Palm as u8,
            Provider::new(
                dotenv!("PALM_RPC"),
                address!("0xfFE2FF36c5b8D948f788a34f867784828aa7415D")
            )
        );

        providers
    });
}
