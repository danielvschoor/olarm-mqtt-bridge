use serde::{Deserialize, Serialize};

use crate::olarm_api::models::device_state::DeviceState;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MqttDeviceResponse {
    pub status: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub data: DeviceState,
    #[serde(rename = "gsmStamp")]
    pub gsm_stamp: Option<u64>,
    #[serde(rename = "wifiStamp")]
    pub wifi_stamp: Option<u64>,
    #[serde(rename = "ethernetStamp")]
    pub ethernet_stamp: Option<u64>,
    #[serde(rename = "_bypassRedis")]
    pub _bypass_redis: Option<bool>
}