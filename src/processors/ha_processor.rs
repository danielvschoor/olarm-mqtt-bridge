use crate::MqttCommand;
use crate::olarm_api::models::request::actions_request::{
    ActionCmd, ActionsRequest, MqttRequest,
};
use crate::olarm_api::olarm_client::OlarmApiTrait;
use crate::processors::ProcessorState;
use rumqttc::QoS;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use crate::throttled_mqtt_client::MqttThrottledClient;

#[derive(Clone)]
pub struct HaProcessor<T>
where
    T: OlarmApiTrait + Clone + Send + Sync + 'static,
{
    pub mqtt_olarm_client: MqttThrottledClient,
    pub http_olarm_client: T,
    pub(crate) processor_state: Arc<RwLock<ProcessorState>>,
}

impl<T: OlarmApiTrait + Clone + Send + Sync> HaProcessor<T> {
    pub async fn handle_action(
        &self,
        device_id: &str,
        imei: &str,
        action_cmd: ActionCmd,
        action_num: usize,
    ) -> anyhow::Result<()> {
        let payload = ActionsRequest {
            action_cmd,
            action_num: action_num.to_string(),
        };
        if let ActionCmd::ZoneBypass = action_cmd {
            let mqtt_payload = MqttRequest::from(payload).unwrap();
        let s_payload = serde_json::to_string(&mqtt_payload).unwrap();
            let control_topic = format!("si/app/v2/{}/control", imei);
            self.mqtt_olarm_client
                .publish_and_wait(
                    &control_topic,
                    QoS::AtLeastOnce,
                    false,
                    s_payload,
                )
                .await?;

        } else {
            let _ = self
                .http_olarm_client
                .send_action(device_id, payload)
                .await?
                .error_for_status()?;
        }

        Ok(())
    }
    pub async fn process_ha_command(
        &self,
        cmd: MqttCommand,
    ) -> anyhow::Result<()> {
        let imei = {
            self.processor_state.read().await.device.imei.clone()
        };
        match cmd {
            MqttCommand::SetArea {
                device_id,
                area_number,
                action_cmd,
            } => {
                self.handle_action(&device_id, &imei, action_cmd, area_number)
                    .await?;
            }
            MqttCommand::SetZoneBypass {
                device_id,
                zone_number,
                action_cmd,
                payload: _payload
            } => {
                self.handle_action(&device_id, &imei, action_cmd, zone_number)
                    .await?;
            }
        }

        Ok(())
    }
}
