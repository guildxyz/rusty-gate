pub mod evm;

use async_trait::async_trait;

pub use evm::EvmChain;
pub use web3::types::{Address, U256};

#[async_trait]
pub trait BalanceQuerier {
    type Address;
    type Id;
    type Balance;
    type Chain;
    type Error;

    async fn get_native_balance(
        &self,
        user_addresses: &[Self::Address],
    ) -> Vec<Result<Self::Balance, Self::Error>>;

    async fn get_fungible_balance(
        &self,
        token_address: Self::Address,
        user_addresses: &[Self::Address],
    ) -> Vec<Result<Self::Balance, Self::Error>>;

    async fn get_non_fungible_balance(
        &self,
        token_address: Self::Address,
        token_id: Option<Self::Id>,
        user_addresses: &[Self::Address],
    ) -> Vec<Result<Self::Balance, Self::Error>>;

    async fn get_special_balance(
        &self,
        token_address: Self::Address,
        token_id: Option<Self::Id>,
        user_addresses: &[Self::Address],
    ) -> Vec<Result<Self::Balance, Self::Error>>;
}
