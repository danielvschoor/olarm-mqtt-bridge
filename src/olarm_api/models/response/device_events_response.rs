use crate::olarm_api::models::event::Event;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct DeviceEventsResponse {
    pub page: i64,
    #[serde(rename = "pageLength")]
    pub page_length: i64,
    #[serde(rename = "pageCount")]
    pub page_count: i64,
    pub limit: i64,
    pub until: i64,
    pub since: i64,
    pub after: String,
    pub filter: String,
    pub data: Vec<Event>,
}
