use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct RefreshOAuthTokenResponse {
    pub oat: String,
    #[serde(rename = "oatExpire")]
    pub oat_expire: u64
}