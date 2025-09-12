use crate::olarm_api::models::device::Device;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct DevicesResponse {
    pub page: i64,
    #[serde(rename = "pageLength")]
    pub page_length: i64,
    #[serde(rename = "pageCount")]
    pub page_count: i64,
    pub search: String,
    pub data: Vec<Device>,
}
