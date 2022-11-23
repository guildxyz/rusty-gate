use super::NumberId;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlatformName {
    Discord = 1,
    Telegram = 2,
    Github = 3,
    Google = 4,
    Twitter = 5,
    Steam = 6,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlatformUserData {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlatformUser {
    pub platform_id: NumberId,
    pub platform_name: PlatformName,
    pub platform_user_id: String,
    pub platform_user_data: Option<PlatformUserData>,
}
