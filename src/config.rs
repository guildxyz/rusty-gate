use std::collections::HashMap;
use tokio::sync::RwLock;
use web3_rpc::web3::Web3;

use crate::types::{Address, Chain};

pub struct MulticallProvider {
    pub rpc_url: String,
    pub address: Address,
}

pub struct Provider {
    pub single: Web3,
    pub multi: MulticallProvider,
}

impl Provider {
    pub fn new(rpc_url: String, address: String) -> Self {
        Self {
            single: Web3::new(rpc_url.clone()),
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
    pub static ref PROVIDERS: RwLock<HashMap<u8, Provider>> = RwLock::new({
        let mut providers = HashMap::new();

        providers.insert(
            Chain::Ethereum as u8,
            Provider::new(
                dotenv!("ETHEREUM_RPC"),
                "0x5ba1e12693dc8f9c48aad8770482f4739beed696".into()
            )
        );
        providers.insert(
            Chain::Polygon as u8,
            Provider::new(
                dotenv!("POLYGON_RPC").into(),
                "0x11ce4B23bD875D7F5C6a31084f55fDe1e9A87507".into()
            )
        );
        providers.insert(
            Chain::Bsc as u8,
            Provider::new(
                dotenv!("BSC_RPC").into(),
                "0x41263cba59eb80dc200f3e2544eda4ed6a90e76c".into()
            )
        );
        providers.insert(
            Chain::Gnosis as u8,
            Provider::new(
                dotenv!("GNOSIS_RPC").into(),
                "0xb5b692a88bdfc81ca69dcb1d924f59f0413a602a".into()
            )
        );
        providers.insert(
            Chain::Fantom as u8,
            Provider::new(
                dotenv!("FANTOM_RPC").into(),
                "0xD98e3dBE5950Ca8Ce5a4b59630a5652110403E5c".into()
            )
        );
        providers.insert(
            Chain::Avalanche as u8,
            Provider::new(
                dotenv!("AVALANCHE_RPC").into(),
                "0x98e2060F672FD1656a07bc12D7253b5e41bF3876".into()
            )
        );
        providers.insert(
            Chain::Arbitrum as u8,
            Provider::new(
                dotenv!("ARBITRUM_RPC").into(),
                "0x52bfe8fE06c8197a8e3dCcE57cE012e13a7315EB".into()
            )
        );
        providers.insert(
            Chain::Celo as u8,
            Provider::new(
                dotenv!("CELO_RPC").into(),
                "0xb74C3A8108F1534Fc0D9b776A9B487c84fe8eD06".into()
            )
        );
        providers.insert(
            Chain::Harmony as u8,
            Provider::new(
                dotenv!("HARMONY_RPC").into(),
                "0x34b415f4d3b332515e66f70595ace1dcf36254c5".into()
            )
        );
        providers.insert(
            Chain::Heco as u8,
            Provider::new(
                dotenv!("HECO_RPC").into(),
                "0x41C0A3059De6bE4f1913630db94d93aB5a2904B4".into()
            )
        );
        providers.insert(
            Chain::Goerli as u8,
            Provider::new(
                dotenv!("GOERLI_RPC").into(),
                "0x77dCa2C955b15e9dE4dbBCf1246B4B85b651e50e".into()
            )
        );
        providers.insert(
            Chain::Optimism as u8,
            Provider::new(
                dotenv!("OPTIMISM_RPC").into(),
                "0x2DC0E2aa608532Da689e89e237dF582B783E552C".into()
            )
        );
        providers.insert(
            Chain::Moonriver as u8,
            Provider::new(
                dotenv!("MOONRIVER_RPC").into(),
                "0x270f2F35bED92B7A59eA5F08F6B3fd34c8D9D9b5".into()
            )
        );
        providers.insert(
            Chain::Rinkeby as u8,
            Provider::new(
                dotenv!("RINKEBY_RPC").into(),
                "0x5ba1e12693dc8f9c48aad8770482f4739beed696".into()
            )
        );
        providers.insert(
            Chain::Metis as u8,
            Provider::new(
                dotenv!("METIS_RPC").into(),
                "0x1a2AFb22B8A90A77a93e80ceA61f89D04e05b796".into()
            )
        );
        providers.insert(
            Chain::Cronos as u8,
            Provider::new(
                dotenv!("CRONOS_RPC").into(),
                "0x0fA4d452693F2f45D28c4EC4d20b236C4010dA74".into()
            )
        );
        providers.insert(
            Chain::Boba as u8,
            Provider::new(
                dotenv!("BOBA_RPC").into(),
                "0xbe2Be647F8aC42808E67431B4E1D6c19796bF586".into()
            )
        );
        providers.insert(
            Chain::Palm as u8,
            Provider::new(
                dotenv!("PALM_RPC").into(),
                "0xfFE2FF36c5b8D948f788a34f867784828aa7415D".into()
            )
        );

        providers
    });
}
