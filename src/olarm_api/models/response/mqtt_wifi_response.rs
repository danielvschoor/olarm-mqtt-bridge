use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct MqttWifiResponse {
    pub status: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub data: Data,
}

#[derive(Serialize, Deserialize)]
struct Data {
    #[serde(rename = "wifiStatus")]
    pub wifi_status: String,
    #[serde(rename = "wifiConnected")]
    pub wifi_connected: u8,
    #[serde(rename = "wifiSSID")]
    pub wifi_ssid: String,
    #[serde(rename = "wifiRSSI")]
    pub wifi_rssi: isize,
}

