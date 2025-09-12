use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "eventTime")]
    pub event_time: u64,
    #[serde(rename = "eventAction")]
    pub event_action: String,
    #[serde(rename = "eventState")]
    pub event_state: String,
    #[serde(rename = "eventNum")]
    pub event_num: i64,
    #[serde(rename = "eventMsg")]
    pub event_msg: String,
    #[serde(rename = "userFullname")]
    pub user_fullname: String,
}
