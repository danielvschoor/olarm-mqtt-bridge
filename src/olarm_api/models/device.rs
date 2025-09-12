use crate::olarm_api::models::device_profile::DeviceProfile;
use crate::olarm_api::models::device_state::DeviceState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Device {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "deviceName")]
    pub device_name: String,
    #[serde(rename = "deviceSerial")]
    pub device_serial: String,
    #[serde(rename = "deviceAlarmType")]
    pub device_alarm_type: String,
    #[serde(rename = "deviceTimestamp")]
    pub device_timestamp: i64,
    #[serde(rename = "deviceStatus")]
    pub device_status: String,
    #[serde(rename = "deviceState")]
    pub device_state: DeviceState,
    #[serde(rename = "deviceProfile")]
    pub device_profile: DeviceProfile,
}
