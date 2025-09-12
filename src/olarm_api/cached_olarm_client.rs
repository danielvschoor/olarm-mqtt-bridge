use crate::olarm_api::models::response::login_via_user_credentials_response::LoginViaUserCredentialsResponse;
use crate::olarm_api::models::response::refresh_oauth_token_response::RefreshOAuthTokenResponse;
use crate::olarm_api::models::response::{
    device_response::DeviceResponse, devices_response::DevicesResponse,
    get_actions_response::GetActionsResponse, user_response::UserResponse,
};
use crate::olarm_api::olarm_client::{OlarmApiTrait, OlarmClient};
use anyhow::Result;
use moka::future::Cache;
use reqwest::Response;
use std::time::Duration;
use crate::olarm_api::models::request::actions_request::ActionsRequest;

#[derive(Clone)]
pub struct CachedOlarmClient<T>
where
    T: OlarmApiTrait,
{
    client: T,
    devices_cache: Cache<&'static str, DevicesResponse>,
    device_cache: Cache<String, DeviceResponse>,
    actions_cache: Cache<String, GetActionsResponse>,
    user_cache: Cache<String, UserResponse>
}
impl OlarmApiTrait for CachedOlarmClient<OlarmClient> {
        async fn get_user(&self, user_id: &str) -> Result<UserResponse> {
        let key = user_id.to_string();

        self.user_cache
            .try_get_with(key.clone(), async move { self.client.get_user(&key).await })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch user: {}", e))
    }

    async fn get_devices(&self) -> Result<DevicesResponse> {
        self.devices_cache
            .try_get_with("devices", async { self.client.get_devices().await })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch devices: {}", e))
    }

    async fn get_device(&self, device_id: &str) -> Result<DeviceResponse> {
        let key = device_id.to_string();

        self.device_cache
            .try_get_with(
                key.clone(),
                async move { self.client.get_device(&key).await },
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch device: {}", e))
    }

    async fn send_action(&self, device_id: &str, payload: ActionsRequest) -> Result<Response> {
        // don't cache this, it's a POST request
        self.client.send_action(device_id, payload).await
    }

    async fn get_actions(&self, device_id: &str) -> Result<GetActionsResponse> {
        let key = device_id.to_string();

        self.actions_cache
            .try_get_with(
                key.clone(),
                async move { self.client.get_actions(&key).await },
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch actions: {}", e))
    }

    async fn get_oauth_response(&self) -> Result<LoginViaUserCredentialsResponse> {
        self.client.get_oauth_response().await
    }

    async fn refresh_oauth_token(&self, refresh_token: &str) -> Result<RefreshOAuthTokenResponse> {
        self.client.refresh_oauth_token(refresh_token).await
    }
}
impl<T> CachedOlarmClient<T>
where
    T: OlarmApiTrait,
{
    pub fn new(client: T) -> Self {
        Self {
            client,
            devices_cache: Cache::builder()
                .time_to_live(Duration::from_secs(60)) // refresh every 60s
                .build(),
            device_cache: Cache::builder()
                .time_to_live(Duration::from_secs(30))
                .build(),
            actions_cache: Cache::builder()
                .time_to_live(Duration::from_secs(30))
                .build(),
            user_cache: Cache::builder()
                .time_to_live(Duration::from_secs(300))
                .build(),
           
        }
    }
}


