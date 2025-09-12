use crate::{AreaObject};
use crate::olarm_api::models::device_profile::DeviceProfile;
use crate::olarm_api::models::response::mqtt_device_response::MqttDeviceResponse;
use crate::processors::{MqttDeviceResponseProcessor, ProcessorState};
use chrono::DateTime;
use rumqttc::{QoS};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{warn};
use crate::home_assistant::alarm_control_panel::{AlarmControlPanelDiscoveryPayload, AlarmFeature, AlarmState};
use crate::home_assistant::availability::{Availability, AvailabilityState};
use crate::olarm_api::models::request::actions_request::ActionCmd;
use crate::olarm_api::olarm_client::{OlarmApiTrait};

#[derive(Clone)]
pub struct PanelProcessor<T> where  T: OlarmApiTrait + Clone + Send + Sync + 'static {
    pub ha_client: rumqttc::AsyncClient,
    pub olarm_client: T,
}

impl<T: OlarmApiTrait + Clone + Send + Sync> PanelProcessor<T> {
    pub fn get_areas(payload: &MqttDeviceResponse, device_profile: &DeviceProfile) -> Vec<AreaObject> {
        let olarm_state = &payload.data;
        let mut areas_labels = device_profile.areas_labels.clone();
        let area_count = device_profile.areas_limit as usize;
        let mut panel_data: Vec<AreaObject> = Vec::new();
        for (area_num, area_label) in areas_labels.iter_mut().enumerate().take(area_count) {
            if area_label.is_empty() {
                warn!(
                "This device's area names have not been set up in Olarm, generating automatically"
            );
                *area_label = format!("Area {}", area_num + 1);
            }
            if olarm_state.areas.len() > area_num {
                let area_state = match olarm_state.areas[area_num].as_str() {
                    "notready" => Some(AlarmState::Disarmed.as_serde_value()),
                    "countdown" => Some(AlarmState::Arming.as_serde_value()),
                    "sleep" => Some(AlarmState::ArmedNight.as_serde_value()),
                    "stay" => Some(AlarmState::ArmedHome.as_serde_value()),
                    "arm" => Some(AlarmState::ArmedAway.as_serde_value()),
                    "alarm" => Some(AlarmState::Triggered.as_serde_value()),
                    "fire" => Some(AlarmState::Triggered.as_serde_value()),
                    "emergency" => Some(AlarmState::Triggered.as_serde_value()),
                    "disarm" => Some(AlarmState::Disarmed.as_serde_value()),
                    _ => None,
                };
                panel_data.push(AreaObject {
                    name: area_label.to_string(),
                    state: area_state,
                    area_number: area_num + 1,
                })
            }
        }
        panel_data
    }
}
impl<T:OlarmApiTrait + Sync + Send + Clone> MqttDeviceResponseProcessor for PanelProcessor<T> {
    async fn handle(
        &self,
        msg: MqttDeviceResponse,
        processor_state: Arc<RwLock<ProcessorState>>,
    ) -> anyhow::Result<()> {
        let (device_profile, device) = {
            let read_lock = processor_state.read().await;
            (read_lock.device_profile.clone(), read_lock.device.clone())
        };
        let device_id = device.id.clone();

        for area in Self::get_areas(&msg, &device_profile) {
            let discovery_topic = format!(
                "homeassistant/alarm_control_panel/{}_area_{}/config", device_id, area.area_number
            );
            let unique_id = format!("olarm_{}_area_{}", device_id, area.area_number);
            
            let state_topic = format!("olarm/device/{}/area/{}/state", device_id, area.area_number);
            let control_topic = format!("olarm/device/{}/area/{}/set", device_id, area.area_number);
            let json_attributes_topic = format!("olarm/device/{}/area/{}/attributes", device_id, area.area_number);
            let global_availability_topic = format!("olarm/device/{}/availability", device_id);
            let device_availability_topic = format!("olarm/device/{}/area/{}/availability", device_id, area.area_number);
            
            // Atomically check-and-insert without holding a lock across .await
            let should_publish = processor_state.write().await.published_discovery.insert(unique_id.clone());

            if should_publish {
                let discovery_object = AlarmControlPanelDiscoveryPayload {
                    name: format!("Olarm {} Area {}", device.name, area.area_number),
                    unique_id: unique_id.clone(),
                    state_topic: state_topic.clone(),
                    command_topic: control_topic.clone(),
                    payload_arm_away: ActionCmd::AreaArm.to_string(),
                    payload_arm_home: ActionCmd::AreaStay.to_string(),
                    payload_arm_night: ActionCmd::AreaSleep.to_string(),
                    payload_disarm: ActionCmd::AreaDisarm.to_string(),
                    code: None,
                    code_arm_required: Some(false),
                    code_disarm_required: Some(false),
                    code_trigger_required: Some(false),
                    command_template: None,
                    json_attributes_topic: Some(json_attributes_topic.clone()),
                    availability: Some(vec![
                        Availability{
                            payload_available: Some(AvailabilityState::Online.as_serde_value()),
                            payload_not_available: Some(AvailabilityState::Offline.as_serde_value()),
                            topic: global_availability_topic.to_string(),
                            value_template: None,
                        },
                        Availability{
                            payload_available: Some(AvailabilityState::Online.as_serde_value()),
                            payload_not_available: Some(AvailabilityState::Offline.as_serde_value()),
                            topic: device_availability_topic.to_string(),
                            value_template: None,
                        }
                    ]),
                    availability_mode: None, //defaults to "latest"
                    supported_features: Some(vec![
                        AlarmFeature::ArmAway,
                        AlarmFeature::ArmHome,
                        AlarmFeature::ArmNight
                    ]),
                };

                self.ha_client
                    .publish(&discovery_topic, QoS::AtLeastOnce, true, serde_json::to_string(&discovery_object)?)
                    .await?;

                self.ha_client
                    .subscribe(&control_topic, QoS::AtLeastOnce)
                    .await?;

            }

            if let Some(area_state) = &area.state {
                self.ha_client
                    .publish(&state_topic, QoS::AtMostOnce, true, area_state.to_string())
                    .await?;
            }

            self.ha_client.publish(&device_availability_topic, QoS::AtLeastOnce, true, AvailabilityState::Online.as_serde_value()).await?;

            if let Ok(device_actions) = self.olarm_client.get_actions(&device.id).await {
                let mut attributes: HashMap<String, String> = HashMap::new();
                let mut user_fullname = "No User".to_string();
                let mut action_created: i64 = 0;
                let mut action_cmd: Option<String> = None;


                for change in device_actions.actions {
                    if change.action_cmd != "zone-bypass" &&
                        change.action_cmd != "pgm-open" &&
                        change.action_cmd != "pgm-close" &&
                        change.action_cmd != "pgm-pulse" &&
                        change.action_cmd != "ukey-activate"
                        && (change.action_num as usize) == area.area_number
                        && action_created < change.action_created {
                        user_fullname = change.user_fullname;
                        action_created = change.action_created;
                        action_cmd = Some(change.action_cmd);
                    }
                }


                attributes.insert("userFullname".to_string(), user_fullname.to_string());

                if let Some(last_changed_dt) = DateTime::from_timestamp_millis(action_created) {
                    attributes.insert("actionCreated".to_string(), last_changed_dt.to_string());
                }

                if let Some(cmd) = action_cmd {
                    attributes.insert("actionCmd".to_string(), cmd.to_string());
                }

                self.ha_client.publish(&json_attributes_topic, QoS::AtMostOnce, true, serde_json::to_string(&attributes).unwrap()).await?;
            }
           
        }
        Ok(())
    }
}
