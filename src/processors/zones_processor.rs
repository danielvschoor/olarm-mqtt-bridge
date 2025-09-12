use crate::ZoneObject;
use crate::home_assistant::binary_sensor::{BinarySensorDiscoveryPayload, Device};
use crate::home_assistant::switch::SwitchDiscoveryPayload;
use crate::olarm_api::models::device_profile::DeviceProfile;
use crate::olarm_api::models::response::mqtt_device_response::MqttDeviceResponse;
use crate::olarm_api::models::response::user_response::UserDevice;
use crate::processors::{MqttDeviceResponseProcessor, ProcessorState};
use chrono::DateTime;
use rumqttc::QoS;
use std::cmp::min;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::join;
use tokio::sync::RwLock;
use tracing::{error, trace};
use crate::home_assistant::availability::{Availability, AvailabilityState};
use crate::home_assistant::models::requests::zone_bypass::ZoneBypassRequest;

#[derive(Clone)]
pub struct ZonesProcessor {
    pub ha_client: rumqttc::AsyncClient,
}

impl ZonesProcessor {
    pub fn get_zones(
        payload: &MqttDeviceResponse,
        device_profile: &DeviceProfile,
    ) -> Vec<ZoneObject> {
        let zone_limit = device_profile.zones_limit as usize;

        // Compute safe bounds to avoid panics on mismatched lengths
        let zones_len = payload.data.zones.len();
        let stamps_len = payload.data.zones_stamp.len();
        let types_len = device_profile.zones_types.len();
        let labels_len = device_profile.zones_labels.len();
        let safe_count = min(
            min(min(zone_limit, zones_len), stamps_len),
            min(types_len, labels_len),
        );

        let mut zones: Vec<ZoneObject> = Vec::with_capacity(safe_count + 2);
        for i in 0..safe_count {
            let mut bypass_state = "off";
            let zone_state = match payload.data.zones[i].to_ascii_lowercase().as_str() {
                "a" => "on",
                "b" => {
                    bypass_state = "on";
                    "off"
                }
                _ => "off",
            };
            
            let last_changed_dt_opt =
                match payload.data.zones_stamp[i] {
                    None => None,
                    Some(x) => { DateTime::from_timestamp_millis(x as i64) }
                };
            let (zone_type, zone_friendly_type) = match device_profile.zones_types[i] {
                0 => ("motion", "Motion Sensor"),
                10 => ("door", "Door Sensor"),
                11 => ("window", "Window Sensor"),
                20 => ("motion", "Motion Sensor"),
                21 => ("motion", "Motion Sensor"),
                90 => ("problem", "Sensor Disabled"),
                50 => ("safety", "Panic Button"),
                51 => ("safety", "Panic Button"),
                1000 => ("plug", "Device Power Plug Status"),
                1001 => ("power", "Battery Powered"),
                _ => ("motion", "Motion Sensor"),
            };
            let mut zone_attributes: HashMap<String, String> = HashMap::with_capacity(3);
            zone_attributes.insert("zone_number".to_string(), i.to_string());
            if let Some(last_changed_dt) = last_changed_dt_opt {
                zone_attributes.insert(
                    "last_tripped_time".to_string(),
                    last_changed_dt.to_rfc3339(),
                );
            }
            zone_attributes.insert("zone_type".to_string(), zone_friendly_type.to_string());

            let zone = ZoneObject {
                name: device_profile.zones_labels[i].to_string(),
                state: zone_state.to_string(),
                last_changed: last_changed_dt_opt,
                r#type: zone_type.to_string(),
                zone_number: i + 1, // Starts at 1, not 0
                attributes: Some(zone_attributes),
                bypass_state: bypass_state.to_string(),
            };
            zones.push(zone);
        }
        zones
    }

    pub async fn handle_binary_sensor(
        &self,
        device: &UserDevice,
        unique_id: &str,
        zone: &ZoneObject,
        should_publish: bool,
    ) -> anyhow::Result<()> {
        let device_id = device.id.clone();

        let discovery_topic = format!("homeassistant/binary_sensor/{}/config", unique_id);
        let state_topic = format!("olarm/device/{}/zone/{}/state", device_id, zone.zone_number);
        let json_attributes_topic = format!(
            "olarm/device/{}/zone/{}/attributes",
            device_id, zone.zone_number
        );
        let global_availability_topic = format!("olarm/device/{}/availability", device_id);
        let device_availability_topic = format!("olarm/device/{}/zone/{}/availability", device_id, zone.zone_number);

        if should_publish {
            let discovery_object = BinarySensorDiscoveryPayload {
                device: Device {
                    identifiers: vec![device.imei.clone(), device.id.clone()],
                    manufacturer: "Daniel van Schoor".to_string(),
                    name: format!("Olarm Sensors({})", &device.name),
                    model: device.alarm_type.clone(),
                },
                name: zone.name.clone(),
                state_topic: state_topic.clone(),
                unique_id: unique_id.to_string(),
                payload_on: "on".to_string(),
                payload_off: "off".to_string(),
                device_class: zone.r#type.to_string(),
                off_delay: None, //if zone.r#type == "motion" { Some(2) } else { None },
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
            };
            let discovery_payload = serde_json::to_string(&discovery_object)?;
            trace!("{}", discovery_payload);

            self.ha_client
                .publish(&discovery_topic, QoS::AtLeastOnce, true, discovery_payload)
                .await?;
        }
        match join!(
            self.ha_client
                .publish(&state_topic, QoS::AtMostOnce, true, zone.state.clone()),

            self.ha_client.publish(
                &json_attributes_topic,
                QoS::AtMostOnce,
                false,
                serde_json::to_string(&zone.attributes)?,
            ),
            self.ha_client.publish(&device_availability_topic, QoS::AtLeastOnce, true, AvailabilityState::Online.as_serde_value())
        ) {
            (Err(e), _, _) | (_, Err(e),_) | (_, _,Err(e)) => {
                error!("Error publishing to {}: {:?}", state_topic, e);
                Err(anyhow::Error::from(e))
            }
            _ => Ok(()),
        }
    }

    pub async fn handle_bypass_switch(
        &self,
        device: &UserDevice,
        unique_id: &str,
        zone: &ZoneObject,
        should_publish: bool,
    ) -> anyhow::Result<()> {
        if zone.r#type == "safety"{
            return Ok(());
        }
        let device_id = device.id.clone();
        let discovery_topic = format!("homeassistant/switch/{}/config", unique_id);
        let state_topic = format!(
            "olarm/device/{}/zone/{}/bypass/state",
            device_id, zone.zone_number
        );
        let json_attributes_topic = format!(
            "olarm/device/{}/zone/{}/bypass/attributes",
            device_id, zone.zone_number
        );
        let command_topic = format!(
            "olarm/device/{}/zone/{}/bypass/set",
            device_id, zone.zone_number
        );
        let global_availability_topic = format!("olarm/device/{}/availability", device_id);
        let device_availability_topic = format!("olarm/device/{}/zone/{}/availability", device_id, zone.zone_number);

        if should_publish {
            let discovery_object = SwitchDiscoveryPayload {
                device: Device {
                    identifiers: vec![device.imei.clone(), device.id.clone()],
                    manufacturer: "Daniel van Schoor".to_string(),
                    name: format!("Olarm Sensors({})", &device.name),
                    model: device.alarm_type.clone(),
                },
                name: format!("{} Bypass ({})", zone.name, &device.name),
                payload_off: Some(ZoneBypassRequest::new(false).to_payload()),
                payload_on: Some(ZoneBypassRequest::new(true).to_payload()),
                state_off: Some("off".to_string()),
                state_on: Some("on".to_string()),
                state_topic: state_topic.clone(),
                unique_id: unique_id.to_string(),
                json_attributes_topic: Some(json_attributes_topic.clone()),
                command_topic: command_topic.clone(),
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
                optimistic: Some(true),
            };
            let discovery_payload = serde_json::to_string(&discovery_object)?;
            trace!("{}", discovery_payload);

            self.ha_client
                .publish(&discovery_topic, QoS::AtLeastOnce, true, discovery_payload)
                .await?;

            self.ha_client
                .subscribe(&command_topic, QoS::AtLeastOnce)
                .await?;
        }
        match join!(
            self.ha_client.publish(
                &state_topic,
                QoS::AtMostOnce,
                true,
                zone.bypass_state.clone()
            ),
            //todo: bypass attributes
            self.ha_client.publish(
                &json_attributes_topic,
                QoS::AtMostOnce,
                true,
                serde_json::to_string(&zone.attributes)?,
            )
        ) {
            (Err(e), _) | (_, Err(e)) => {
                error!("Error publishing to {}: {:?}", state_topic, e);
                Err(anyhow::Error::from(e))
            }
            _ => Ok(()),
        }
    }

    fn build_binary_sensor_unique_id(device_id: &str, zone_number: usize) -> String {
        format!("{}_{}_binary", device_id, zone_number)
    }

    fn build_bypass_switch_unique_id(device_id: &str, zone_number: usize) -> String {
        format!("{}_{}_bypass", device_id, zone_number)
    }
}
impl MqttDeviceResponseProcessor for ZonesProcessor {
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
        let zones = Self::get_zones(&msg, &device_profile);

        for zone in zones {
            trace!("{:?}", &zone);
            let binary_unique_id =
                Self::build_binary_sensor_unique_id(&device_id, zone.zone_number);
            let bypass_unique_id =
                Self::build_bypass_switch_unique_id(&device_id, zone.zone_number);

            let (binary_publish, bypass_publish) =
                // Atomically check-and-insert without holding a lock across .await
                {
                    let state = processor_state.write().await;
                    (state.published_discovery.insert(binary_unique_id.clone()),
                     state.published_discovery.insert(bypass_unique_id.clone()))
                };
            match join!(
                self.handle_binary_sensor(&device, &binary_unique_id, &zone, binary_publish),
                self.handle_bypass_switch(&device, &bypass_unique_id, &zone, bypass_publish)
            ) {
                (Err(e), _) | (_, Err(e)) => {
                    error!("Error processing zone {}: {:?}", binary_unique_id, e);
                    return Err(e);
                }
                _ => {}
            }
        }

        Ok(())
    }
}
