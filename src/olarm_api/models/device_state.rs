use crate::olarm_api::models::power::Power;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeviceState {
    pub timestamp: u64,
    #[serde(rename = "cmdRecv")]
    pub cmd_recv: i64,
    #[serde(rename = "type")]
    pub r#type: String,
    pub areas: Vec<String>,
    #[serde(rename = "areasDetail")]
    pub areas_detail: Vec<String>,
    #[serde(rename = "areasStamp")]
    pub areas_stamp: Vec<u64>,
    pub zones: Vec<String>,
    #[serde(rename = "zonesStamp")]
    pub zones_stamp: Vec<Option<u64>>,
    pub pgm: Vec<String>,
    #[serde(rename = "pgmOb")]
    pub pgm_ob: Vec<String>,
    pub power: Power,
}
