use crate::olarm_api::models::response::device_response::DeviceResponse;
use crate::olarm_api::models::response::devices_response::DevicesResponse;
use crate::olarm_api::models::response::get_actions_response::GetActionsResponse;
use crate::olarm_api::models::response::login_via_user_credentials_response::LoginViaUserCredentialsResponse;
use crate::olarm_api::models::response::refresh_oauth_token_response::RefreshOAuthTokenResponse;
use crate::olarm_api::models::response::user_response::UserResponse;
use anyhow::Context;
use reqwest::header::HeaderMap;
use reqwest::{Method, Response};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::error;
use crate::olarm_api::models::request::actions_request::ActionsRequest;

#[derive(Clone)]
pub struct OlarmClient {
    client: reqwest::Client,
    login_via_user_credentials_response: Arc<RwLock<Option<LoginViaUserCredentialsResponse>>>,
    username: String,
    password: String,
}
impl OlarmClient {
    pub fn new(access_token: String, username: &str, password: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", access_token).parse().unwrap(),
        );

        Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
            // login_via_user_credentials_response: Default::default(),
            login_via_user_credentials_response: Arc::new(Default::default()),
            username: username.to_string(),
            password: password.to_string(),
        }
    }
    async fn login_via_user_credentials(&self) -> anyhow::Result<LoginViaUserCredentialsResponse> {
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/x-www-form-urlencoded".parse()?);

        let mut params = std::collections::HashMap::new();
        params.insert("userEmailPhone", &self.username);
        params.insert("userPass", &self.password);

        let request = client
            .request(
                reqwest::Method::POST,
                "https://auth.olarm.com/api/v4/oauth/login/mobile",
            )
            .headers(headers)
            .form(&params);
        let response = request.send().await?;

        let contents = response.text().await?;
        serde_json::from_str(&contents)
            .with_context(|| format!("Unable to deserialize response. Body was: \"{}\"", contents))
    }
}
impl OlarmApiTrait for OlarmClient {

    async fn get_user(&self, user_index: &str) -> anyhow::Result<UserResponse> {
        let url = format!("https://api-legacy.olarm.com/api/v2/users/{}", user_index);
        let access_token = self.get_oauth_response().await?.oat;
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();

        headers.insert(
            "Authorization",
            format!("Bearer {}", access_token).parse().unwrap(),
        );
        let request = client.request(reqwest::Method::GET, url).headers(headers);
        let response = request.send().await?;

        let contents = response.text().await?;
        serde_json::from_str(&contents)
            .with_context(|| format!("Unable to deserialize response. Body was: \"{}\"", contents))
    }

    async fn get_devices(&self) -> anyhow::Result<DevicesResponse> {
        let url = "https://apiv4.olarm.co/api/v4/devices";

        let response = self.client.get(url).send().await?;

        let contents = response.text().await?;
        serde_json::from_str(&contents)
            .with_context(|| format!("Unable to deserialize response. Body was: \"{}\"", contents))
    }
    // pub async fn login_via_user_credentials(
    //     &self,
    //     force_refresh: bool,
    // ) -> anyhow::Result<LoginViaUserCredentialsResponse> {
    //     if force_refresh {
    //         return Self::login_via_user_credentials_internal(&self.username, &self.password).await;
    //     }
    //
    //     let now = Utc::now().timestamp();
    //     let mut local_login_creds: Option<LoginViaUserCredentialsResponse> = None;
    //
    //     // Read scope
    //     let need_refresh = {
    //         let read = self.login_via_user_credentials_response.read().await;
    //         match &*read {
    //             None => true,
    //             Some(cached) => {
    //                 local_login_creds = Some(cached.clone());
    //                 (cached.oat_expire - now) <= Self::REFRESH_EARLY_SECONDS
    //             }
    //         }
    //     }; // read guard dropped here
    //
    //     if !need_refresh {
    //         return Ok(local_login_creds.unwrap());
    //     }
    //
    //     // Perform refresh without holding any lock
    //     let refreshed =
    //         Self::login_via_user_credentials_internal(&self.username, &self.password).await?;
    //
    //     // Write back
    //     let mut write = self.login_via_user_credentials_response.write().await;
    //     *write = Some(refreshed.clone());
    //     Ok(refreshed)
    // }

    async fn get_device(&self, device_id: &str) -> anyhow::Result<DeviceResponse> {
        let url = format!("https://apiv4.olarm.co/api/v4/devices/{}", device_id);
        let response = self.client.get(url).send().await?;
        let contents = response.text().await?;
        serde_json::from_str(&contents)
            .with_context(|| format!("Unable to deserialize response. Body was: \"{}\"", contents))
    }

    async fn send_action(&self, device_id: &str, payload: ActionsRequest) -> anyhow::Result<Response> {
        let url = format!(
            "https://apiv4.olarm.co/api/v4/devices/{}/actions",
            device_id
        );
        Ok(self
            .client
            .request(Method::POST, url)
            .json(&payload)
            .send()
            .await?)
    }

    async fn get_actions(&self, device_id: &str) -> anyhow::Result<GetActionsResponse> {
        let url = format!(
            "https://apiv4.olarm.co/api/v4/devices/{}/actions",
            device_id
        );
        let response = self.client.get(url).send().await?;
        let contents = response.text().await?;
        serde_json::from_str(&contents)
            .with_context(|| format!("Unable to deserialize response. Body was: \"{}\"", contents))
    }

    async fn get_oauth_response(&self) -> anyhow::Result<LoginViaUserCredentialsResponse> {
        // First, check if we have a valid non-expired token
        {
            let lock = self.login_via_user_credentials_response.read().await;
            if let Some(ref response) = *lock
                && !response.is_expired() {
                    return Ok(response.clone());
                }
        }
        
        // If we reach here, we either have no token or it's expired
        let refresh_result = {
            let lock = self.login_via_user_credentials_response.read().await;
            match *lock {
                Some(ref response) => {
                    // Try to refresh with existing refresh token
                    Some(self.refresh_oauth_token(&response.ort).await)
                }
                None => None, // No existing token, need full login
            }
        };
        
        match refresh_result {
            Some(Ok(refresh_response)) => {
                // Successfully refreshed, update stored response
                let mut write_lock = self.login_via_user_credentials_response.write().await;
                if let Some(ref mut stored_response) = *write_lock {
                    stored_response.update_from_refresh_response(&refresh_response);
                    return Ok(stored_response.clone());
                }
                // This shouldn't happen, but handle gracefully
                error!("Lost stored response during refresh");
            }
            Some(Err(refresh_error)) => {
                error!("Failed to refresh oauth token: {}", refresh_error);
                // Fall through to full re-login
            }
            None => {
                // No existing token, proceed with full login
            }
        }
        
        // Either no token exists or refresh failed - do full login
        match self.login_via_user_credentials().await {
            Ok(login_response) => {
                // Store the new response
                let mut write_lock = self.login_via_user_credentials_response.write().await;
                *write_lock = Some(login_response.clone());
                Ok(login_response)
            }
            Err(login_error) => {
                error!("Failed to get oauth token via login: {}", login_error);
                Err(login_error)
            }
        }
    }

    async fn refresh_oauth_token(
        &self,
        refresh_token: &str,
    ) -> anyhow::Result<RefreshOAuthTokenResponse> {
        let url = "https://auth.olarm.com/api/v4/oauth/refresh";

        let response = self
            .client
            .post(url)
            .form(&[("ort", refresh_token)])
            .send()
            .await?;
        let contents = response.text().await?;

        serde_json::from_str(&contents)
            .with_context(|| format!("Unable to deserialize response. Body was: \"{}\"", contents))
    }
}

pub trait OlarmApiTrait {
    fn get_user(
        &self,
        user_index: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<UserResponse>> + Send;
    fn get_devices(
        &self,
    ) -> impl std::future::Future<Output = anyhow::Result<DevicesResponse>> + Send;
    fn get_device(
        &self,
        device_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<DeviceResponse>> + Send;
    fn send_action(
        &self,
        device_id: &str,
        payload: ActionsRequest,
    ) -> impl Future<Output = anyhow::Result<Response>> + Send;
    fn get_actions(
        &self,
        device_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<GetActionsResponse>> + Send;
    fn get_oauth_response(&self) -> impl std::future::Future<Output = anyhow::Result<LoginViaUserCredentialsResponse>> + Send;
    fn refresh_oauth_token(
        &self,
        refresh_token: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<RefreshOAuthTokenResponse>> + Send;
}
// Implement OlarmApiTrait for Arc<T> where T: OlarmApiTrait
impl<T> OlarmApiTrait for Arc<T>
where
    T: OlarmApiTrait + Send + Sync,
{

    async fn get_user(&self, user_index: &str) -> anyhow::Result<UserResponse> {
        self.as_ref().get_user(user_index).await
    }

    async fn get_devices(&self) -> anyhow::Result<DevicesResponse> {
        self.as_ref().get_devices().await
    }

    async fn get_device(&self, device_id: &str) -> anyhow::Result<DeviceResponse> {
        self.as_ref().get_device(device_id).await
    }

    async fn send_action(&self, device_id: &str, payload: ActionsRequest) -> anyhow::Result<Response> {
        self.as_ref().send_action(device_id, payload).await
    }

    async fn get_actions(&self, device_id: &str) -> anyhow::Result<GetActionsResponse> {
        self.as_ref().get_actions(device_id).await
    }

    async fn get_oauth_response(&self) -> anyhow::Result<LoginViaUserCredentialsResponse> {
        self.as_ref().get_oauth_response().await
    }

    async fn refresh_oauth_token(&self, refresh_token: &str) -> anyhow::Result<RefreshOAuthTokenResponse> {
        self.as_ref().refresh_oauth_token(refresh_token).await
    }
}
