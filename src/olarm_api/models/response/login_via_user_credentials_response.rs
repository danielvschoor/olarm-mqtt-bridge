use std::time::SystemTime;
use serde::Deserialize;
use crate::olarm_api::models::response::refresh_oauth_token_response::RefreshOAuthTokenResponse;

#[derive(Deserialize, Clone)]
pub struct LoginViaUserCredentialsResponse {
    #[serde(rename = "userIndex")]
    pub user_index: i64,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub oat: String,
    #[serde(rename = "oatExpire")]
    pub oat_expire: u64,
    pub ort: String,
}

impl LoginViaUserCredentialsResponse {
    /// Check if the token is expired (with a 30s buffer)
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now + 30 >= self.oat_expire
    }
    
    pub fn update_from_refresh_response(&mut self, refresh_response: &RefreshOAuthTokenResponse) {
        self.oat = refresh_response.oat.clone();
        self.oat_expire = refresh_response.oat_expire;
    }
}