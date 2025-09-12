use std::sync::{Arc};
use dashmap::DashSet;
use tokio::sync::RwLock;
use crate::olarm_api::models::response::mqtt_device_response::MqttDeviceResponse;
use crate::olarm_api::models::response::user_response::UserDevice;

pub mod ha_processor;
pub mod zones_processor;
pub mod panel_processor;

/// Trait for a processor that handles MQTT messages
pub trait MqttDeviceResponseProcessor: Send + Sync + 'static {
    async fn handle(&self, msg: MqttDeviceResponse, processor_state: Arc<RwLock<ProcessorState>>) -> anyhow::Result<()>;
}

pub struct ProcessorState {
    pub device_profile: crate::olarm_api::models::device_profile::DeviceProfile,
    pub published_discovery: Arc<DashSet<String>>,
    pub device: UserDevice
}