lazy_static::lazy_static! {
    pub static ref ETHERSCAN_API_KEY: &'static str = dotenv!("ETHERSCAN_API_KEY");
}
