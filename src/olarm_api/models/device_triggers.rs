use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceTriggers {
    pub ver: i64,
    #[serde(rename = "lastCheck")]
    pub last_check: i64,
    #[serde(rename = "areasRemind")]
    pub areas_remind: Vec<Vec<i64>>,
    #[serde(rename = "zonesIdle")]
    pub zones_idle: Vec<i64>,
    #[serde(rename = "zonesWatch")]
    pub zones_watch: Vec<Vec<i64>>,
}
