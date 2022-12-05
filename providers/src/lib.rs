pub mod evm;

use async_trait::async_trait;

#[async_trait]
pub trait BalanceQuerier {
    type Address;
    type Id;
    type Balance;
    type Chain;
    type Error;

    async fn get_native_balance(
        chain: Self::Chain,
        user: Self::Address,
    ) -> Result<Self::Balance, Self::Error>;

    async fn get_fungible_balance(
        chain: Self::Chain,
        token_address: Self::Address,
        user_address: Self::Address,
    ) -> Result<Self::Balance, Self::Error>;

    async fn get_non_fungible_balance(
        chain: Self::Chain,
        token_address: Self::Address,
        token_id: Option<Self::Id>,
        user_address: Self::Address,
    ) -> Result<Self::Balance, Self::Error>;

    async fn get_special_balance(
        chain: Self::Chain,
        token_address: Self::Address,
        token_id: Self::Id,
        user_address: Self::Address,
    ) -> Result<Self::Balance, Self::Error>;
}
