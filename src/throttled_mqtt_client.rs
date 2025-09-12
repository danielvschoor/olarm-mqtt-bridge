use rumqttc::ClientError;
use rumqttc::{AsyncClient, QoS};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::sync::watch;

use tokio::sync::oneshot;
/// A wrapper around `AsyncClient` with throttling and response tracking.
#[derive(Clone)]
pub struct MqttThrottledClient {
    mqtt_client: AsyncClient,
    in_progress: Arc<Mutex<()>>, // Mutex for throttling messages.
    state: Arc<Mutex<Option<oneshot::Sender<()>>>>, // tracks pending response
}

impl MqttThrottledClient {
    pub fn new(mqtt_client: AsyncClient) -> Self {
        Self {
            mqtt_client,
            in_progress: Arc::new(Mutex::new(())),
            state: Arc::new(Mutex::new(None)),
        }
    }

    /// Publish a message and wait until `notify_response` is called
    pub async fn publish_and_wait<S, V>(
        &self,
        topic: S,
        qos: QoS,
        retain: bool,
        payload: V,
    ) -> anyhow::Result<()>
    where
        S: Into<String>,
        V: Into<Vec<u8>>,
    {
        // Lock to ensure only one message is in-flight
        let in_progress = self.in_progress.lock().await;
        let mut lock = self.state.lock().await;
        // Create oneshot channel for response
        let (tx, rx) = oneshot::channel();
        *lock = Some(tx); // store sender for external notification

        // Publish message
        self.mqtt_client
            .publish(topic, qos, retain, payload)
            .await?;
        drop(lock);
        // Wait for response notification
        let _ = tokio::time::timeout(Duration::from_secs(10), rx).await?;
        drop(in_progress);
        Ok(())
    }

    /// Notify the wrapper that the response for the in-flight message has been received
    pub async fn notify_response(&self) {
        let mut lock = self.state.lock().await;
        if let Some(tx) = lock.take() {
            let _ = tx.send(()); // ignore if receiver dropped
        }
    }
    
    pub async fn subscribe<S: Into<String>>(&self, topic: S, qos: QoS) -> Result<(), ClientError> {
        self.mqtt_client.subscribe(topic, qos).await
    }
    pub async fn publish<S, V>(
        &self,
        topic: S,
        qos: QoS,
        retain: bool,
        payload: V,
    ) -> Result<(), ClientError>
    where
        S: Into<String>,
        V: Into<Vec<u8>>,
    {
        self.mqtt_client.publish(topic, qos, retain, payload).await
    }
}
