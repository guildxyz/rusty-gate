lazy_static::lazy_static! {
    pub static ref ETHEREUM_RPC: &'static str = dotenv!("ETHEREUM_RPC");
}
