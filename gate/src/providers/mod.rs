pub mod balancy;
pub mod general;

use async_trait::async_trait;

#[async_trait]
pub trait BalanceQuerier {
    type Address;
    type Id;
    type Balance;
    type Chain;
    type Error;

    async fn get_native_balance(
        user: Self::Address,
        chain: Self::Chain,
    ) -> Result<Self::Balance, Self::Error>;
    async fn get_fungible_balance(
        user_address: Self::Address,
        token_address: Self::Address,
        chain: Self::Chain,
    ) -> Result<Self::Balance, Self::Error>;
    async fn get_non_fungible_balance(
        user_address: Self::Address,
        token_address: Self::Address,
        token_id: Option<Self::Id>,
        chain: Self::Chain,
    ) -> Result<Self::Balance, Self::Error>;
    async fn get_special_balance(
        user: Self::Address,
        token: Self::Address,
        token_id: Self::Id,
        chain: Self::Chain,
    ) -> Result<Self::Balance, Self::Error>;
}
