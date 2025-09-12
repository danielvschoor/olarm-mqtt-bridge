pub(crate) use crate::home_assistant::device::Device;
use serde::{Deserialize, Serialize};
use crate::home_assistant::availability::{Availability, AvailabilityMode};

#[derive(Serialize, Deserialize)]
pub struct BinarySensorDiscoveryPayload {
    pub device: Device,
    pub device_class: String,
    pub name: String,
    pub payload_off: String,
    pub payload_on: String,
    pub state_topic: String,
    pub unique_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub off_delay: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_attributes_topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<Vec<Availability>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_mode: Option<AvailabilityMode>
}
