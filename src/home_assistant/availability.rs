
use serde::{Deserialize, Serialize};

/// Represents an availability configuration for a Home Assistant entity or similar use case.
///
/// The `Availability` struct is used to define the availability of a device or entity by
/// monitoring a specific topic for payload values indicating whether the entity is
/// "available" or "not available".
///
/// # Fields
///
/// * `payload_available` *(Option<String>)*:
///   The payload that represents the available state.
///
/// * `payload_not_available` *(Option<String>)*:
///   The payload that represents the unavailable state.
///
/// * `topic` *(String)*:
///   An MQTT topic subscribed to receive availability (online/offline) updates.
///
/// * `value_template` *(Option<String>)*:
///   Defines a template to extract device’s availability from the topic.
///   To determine the device’s availability result of this template will be compared to
///   payload_available and payload_not_available.
///
/// # Example
///
/// ```rust
/// let availability = Availability {
///     payload_available: Some("online".to_string()),
///     payload_not_available: Some("offline".to_string()),
///     topic: "device/status".to_string(),
///     value_template: None,
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Availability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_available: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_not_available: Option<String>,
    pub topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_template: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AvailabilityMode {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "latest")]
    Latest,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum AvailabilityState {
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "Offline")]
    Offline,
}

impl AvailabilityState{
    pub fn to_serde_value(&self) -> String{
        let json_value = serde_json::to_value(self).unwrap();
        format!("{}", json_value.as_str().unwrap())
    }
}
