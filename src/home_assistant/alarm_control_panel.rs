use serde::{Deserialize, Serialize};
use crate::home_assistant::availability::{Availability, AvailabilityMode};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlarmControlPanelDiscoveryPayload {
    pub name: String,
    pub unique_id: String,
    pub state_topic: String,
    pub command_topic: String,
    pub payload_arm_away: String,
    pub payload_arm_home: String,
    pub payload_arm_night: String,
    pub payload_disarm: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_arm_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_disarm_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_trigger_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_attributes_topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<Vec<Availability>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_mode: Option<AvailabilityMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_features: Option<Vec<AlarmFeature>>,
}


#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum AlarmState {
    #[serde(rename = "armed_away")]
    ArmedAway,
    #[serde(rename = "armed_custom_bypass")]
    ArmedCustomBypass,
    #[serde(rename = "armed_home")]
    ArmedHome,
    #[serde(rename = "armed_night")]
    ArmedNight,
    #[serde(rename = "armed_vacation")]
    ArmedVacation,
    #[serde(rename = "arming")]
    Arming,
    #[serde(rename = "disarmed")]
    Disarmed,
    #[serde(rename = "disarming")]
    Disarming,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "triggered")]
    Triggered
}
impl AlarmState {
    pub fn to_serde_value(&self) -> String{
        let json_value = serde_json::to_value(self).unwrap();
        format!("{}", json_value.as_str().unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum AlarmFeature {
    #[serde(rename = "arm_away")]
    ArmAway,
    #[serde(rename = "arm_custom_bypass")]
    ArmCustomBypass,
    #[serde(rename = "arm_home")]
    ArmHome,
    #[serde(rename = "arm_night")]
    ArmNight,
    #[serde(rename = "arm_vacation")]
    ArmVacation,
    #[serde(rename = "trigger")]
    Trigger
}