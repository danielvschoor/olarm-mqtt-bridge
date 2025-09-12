use crate::home_assistant::binary_sensor::Device;
use serde::{Deserialize, Serialize};
use crate::home_assistant::availability::{Availability, AvailabilityMode};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SwitchDiscoveryPayload {
    pub device: Device,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_off: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_off: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_on: Option<String>,
    pub state_topic: String,
    pub unique_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_attributes_topic: Option<String>,
    pub command_topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<Vec<Availability>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_mode: Option<AvailabilityMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimistic: Option<bool>,
}
