#![recursion_limit = "256"]
mod config;
mod home_assistant;
pub mod olarm_api;
mod processors;
mod throttled_mqtt_client;

use tracing::{debug, error, trace, warn};
use tracing_appender::rolling;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt};

use crate::config::Config;
use crate::home_assistant::models::requests::zone_bypass::ZoneBypassRequest;
use crate::olarm_api::cached_olarm_client::CachedOlarmClient;
use crate::olarm_api::models::device_profile::DeviceProfile;
use crate::olarm_api::models::request::actions_request::{ActionCmd, MqttRequest};
use crate::olarm_api::models::response::mqtt_device_response::MqttDeviceResponse;
use crate::olarm_api::models::response::mqtt_wifi_response::MqttWifiResponse;
use crate::olarm_api::models::response::user_response::UserDevice;
use crate::olarm_api::olarm_client::{OlarmApiTrait, OlarmClient};
use crate::processors::panel_processor::PanelProcessor;
use crate::processors::zones_processor::ZonesProcessor;
use crate::processors::{MqttDeviceResponseProcessor, ProcessorState};
use crate::throttled_mqtt_client::MqttThrottledClient;
use chrono::{DateTime, Utc};
use dashmap::DashSet;
use processors::ha_processor::HaProcessor;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS, TlsConfiguration, Transport};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tokio::join;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{RwLock, mpsc};
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::from_file("config.toml").or_else(|e| {
        println!("Config file not found. Creating example config.toml...");
        Config::save_example("config.toml")?;
        println!("Please edit config.toml with your settings and restart the application.");
        Err(e)
    })?;

    // Directory for logs
    let log_dir = &config.logging.directory;

    // One file per level
    let debug_file = rolling::daily(log_dir, &config.logging.debug_file);
    let info_file = rolling::daily(log_dir, &config.logging.info_file);
    let warn_file = rolling::daily(log_dir, &config.logging.warn_file);
    let error_file = rolling::daily(log_dir, &config.logging.error_file);

    // Build layers, filtering each level
    let debug_layer = fmt::layer()
        .with_writer(debug_file)
        .with_ansi(false)
        .with_filter(EnvFilter::new("debug"));

    let info_layer = fmt::layer()
        .with_writer(info_file)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

    let warn_layer = fmt::layer()
        .with_writer(warn_file)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::LevelFilter::WARN);

    let error_layer = fmt::layer()
        .with_writer(error_file)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::LevelFilter::ERROR);

    // Console pretty logger (like pretty_env_logger)
    let console_layer = fmt::layer()
        .pretty()
        .with_filter(EnvFilter::new(&config.logging.console_level));

    // Compose subscriber
    tracing_subscriber::registry()
        .with(console_layer)
        .with(debug_layer)
        .with(info_layer)
        .with(warn_layer)
        .with(error_layer)
        .init();

    // Let's get our JWT access token first
    let olarm_client = Arc::new(CachedOlarmClient::new(OlarmClient::new(
        config.olarm.api_token.clone(),
        &config.olarm.username,
        &config.olarm.password,
    )));

    let login_response = olarm_client.get_oauth_response().await?; //todo: retries

    let user_devices = olarm_client
        .get_user(&login_response.user_index.to_string())
        .await?;

    let mut ha_options = MqttOptions::new(
        &config.home_assistant.client_id,
        &config.home_assistant.mqtt_host,
        config.home_assistant.mqtt_port,
    );
    ha_options.set_credentials(
        &config.home_assistant.mqtt_username,
        &config.home_assistant.mqtt_password,
    );
    let (ha_client, mut ha_eventloop) =
        AsyncClient::new(ha_options, config.limits.command_channel_size);

    let published_discovery = Arc::new(DashSet::new());

    //Shared map: IMEI → command sender
    let senders: SenderMap = Arc::new(RwLock::new(HashMap::new()));

    // Run HA event loop in background
    let senders_router = senders.clone();
    let ha_published_discovery = published_discovery.clone();
    tokio::spawn(async move {
        loop {
            match ha_eventloop.poll().await {
                Ok(event) => {
                    if let Event::Incoming(Packet::Publish(p)) = event {
                        let payload = String::from_utf8_lossy(&p.payload).to_string();

                        match command_topic_parser(&p.topic, &payload) {
                            None => {
                                warn!("Failed to parse topic: {:?}", p.topic);
                            }
                            Some(parse_result) => {
                                match senders_router.read().await.get(&parse_result.device_id) {
                                    None => {
                                        error!(
                                            "Received command for device_id {} that is not in the senders map",
                                            parse_result.device_id
                                        );
                                    }
                                    Some(sender) => {
                                        if let Err(e) = sender.send(parse_result.command).await {
                                            error!("Failed to send command: {:?}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "HA event loop failed: {:?}. Forcing rediscovery and resubscriptions",
                        e
                    );
                    ha_published_discovery.clear();
                }
            }
        }
    });

    for dev in user_devices.devices {
        let local_olarm_client = olarm_client.clone();
        let local_ha_client = ha_client.clone();
        let local_senders = senders.clone();
        let device_id = dev.id.clone();
        let local_config = config.clone();
        let local_published_discovery = published_discovery.clone();

        tokio::spawn(async move {
            loop {
                let (tx, rx) =
                    mpsc::channel::<MqttCommand>(local_config.limits.command_channel_size);
                local_senders.write().await.insert(device_id.clone(), tx);
                if let Err(e) = run_alarm_client(
                    &local_config.olarm.broker_url,
                    local_config.olarm.broker_port,
                    dev.clone(),
                    local_olarm_client.clone(),
                    local_ha_client.clone(),
                    rx,
                    &local_config,
                    local_published_discovery.clone(),
                )
                .await
                {
                    error!("Alarm client {} failed: {:?}", dev.id, e);
                    tokio::time::sleep(Duration::from_secs(
                        local_config.intervals.reconnect_delay_seconds,
                    ))
                    .await;
                }
            }
        });
    }

    // Prevent main from exiting
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
async fn get_device_profile<T>(olarm_client: &T, device_id: &str) -> DeviceProfile
where
    T: OlarmApiTrait,
{
    match olarm_client.get_device(device_id).await {
        Ok(r) => r.device_profile,
        Err(e) => {
            warn!(
                "Error occurred while getting device, using fallback. {:?}",
                e
            );
            let user_id;
            match olarm_client.get_oauth_response().await {
                Ok(response) => {
                    user_id = response.user_index;
                }
                Err(e) => {
                    error!("{:?}", e);
                    std::process::exit(1);
                }
            }

            let devices = olarm_client
                .get_user(&user_id.to_string())
                .await
                .unwrap()
                .devices;

            devices
                .iter()
                .find(|x| x.id == device_id)
                .unwrap()
                .profile
                .clone()
        }
    }
}

async fn run_alarm_client<T>(
    host: &str,
    port: u16,
    device: UserDevice,
    olarm_client: Arc<T>,
    ha_client: AsyncClient,
    mut rx: Receiver<MqttCommand>,
    config: &Config,
    published_discovery: Arc<DashSet<String>>,
) -> anyhow::Result<()>
where
    T: OlarmApiTrait + Clone + Send + Sync + 'static,
{
    let imei = device.imei.clone();
    let client_id = format!("native-app-oauth-{}", imei);
    let mqtt_password = &config.olarm.api_token; //olarm_client.get_oauth_response().await?.oat;
    let device_state_topic = format!("so/app/v1/{}", imei);
    let status_topic = format!("si/app/v2/{}/status", imei);
    let ha_availability_topic = format!("olarm/device/{}/availability", &device.id);
    const MQTT_USERNAME: &str = "native_app";

    // --- Per-device MQTT connection ---
    let mut mqttoptions = MqttOptions::new(client_id, host, port);
    mqttoptions.set_clean_session(true);
    mqttoptions.set_credentials(MQTT_USERNAME, mqtt_password);
    mqttoptions.set_keep_alive(Duration::from_secs(
        config.intervals.mqtt_keep_alive_seconds,
    ));
    mqttoptions.set_transport(Transport::Wss(TlsConfiguration::default()));

    let (raw_client, mut event_loop) = AsyncClient::new(mqttoptions, config.limits.mqtt_queue_size);

    let client = MqttThrottledClient::new(raw_client);

    debug!("Subscribing to {}", &device_state_topic);
    client
        .subscribe(&device_state_topic, QoS::AtLeastOnce)
        .await?;

    // Publish status requests on the same task to avoid leaked background publishers on reconnect
    let mut status_tick =
        tokio::time::interval(Duration::from_secs(config.intervals.status_tick_seconds));
    // if let Err(e) = client
    //     .publish(
    //         &status_topic,
    //         QoS::AtLeastOnce,
    //         false,
    //         serde_json::to_string(&MqttRequest::get())?,
    //     )
    //     .await
    // {
    //     error!("Error occurred while publishing: {:?}", e);
    // }
    let device_profile = get_device_profile(&*olarm_client, &device.id).await;

    // Track which discovery configs we've already published
    let processor_state = Arc::new(RwLock::new(ProcessorState {
        device_profile: device_profile.clone(),
        published_discovery: published_discovery.clone(),
        device: device.clone(),
    }));

    let ha_processor = HaProcessor {
        mqtt_olarm_client: client.clone(),
        http_olarm_client: olarm_client.clone(),
        processor_state: processor_state.clone(),
    };
    let zone_processor = ZonesProcessor {
        ha_client: ha_client.clone(),
    };

    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            if let Err(e) = ha_processor.process_ha_command(cmd).await {
                error!("Command processing failed: {}", e);
            }
        }
    });

    let local_client = client.clone();
    let local_client2 = client.clone();
    let local_ha_client = ha_client.clone();
    // Run both loops as futures and short-circuit on the first error
    let reader = async move {
        let mut prev_message_hash: Option<u64> = None; // Store the hash of the previous message
        loop {
            match event_loop.poll().await {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    let payload_str = String::from_utf8_lossy(&p.payload);
                    let mut hasher = DefaultHasher::new();
                    payload_str.hash(&mut hasher);
                    let current_hash = hasher.finish();
                    // Check if the hash matches the previous payload's hash
                    if let Some(prev_hash) = prev_message_hash
                        && prev_hash == current_hash
                    {
                        continue; // Skip processing duplicate payload
                    }
                    prev_message_hash = Some(current_hash);

                    if let Ok(payload) = serde_json::from_str::<MqttDeviceResponse>(&payload_str) {
                        local_client.notify_response().await;
                        // debug!("{:?}", &payload);

                        // Process zones
                        let local_processor_state = processor_state.clone();
                        let local_zones_processor = zone_processor.clone();
                        let payload_for_zones = payload.clone();
                        let zones_handle = tokio::spawn(async move {
                            if let Err(e) = local_zones_processor
                                .handle(payload_for_zones, local_processor_state)
                                .await
                            {
                                error!("Error occurred while processing zone data: {:?}", e);
                            }
                        });
                        // Process panels
                        let local_processor_state = processor_state.clone();
                        let panel_processor = PanelProcessor {
                            ha_client: ha_client.clone(),
                            olarm_client: olarm_client.clone(),
                        };
                        let payload_for_panel = payload.clone();

                        let panel_handle = tokio::spawn(async move {
                            if let Err(e) = panel_processor
                                .handle(payload_for_panel, local_processor_state)
                                .await
                            {
                                error!("Error occurred while processing panel data: {:?}", e);
                            }
                        });

                        let _ = join!(zones_handle, panel_handle);
                    } else if serde_json::from_str::<MqttWifiResponse>(&payload_str).is_ok() {
                        // wifi status message, intentionally ignored
                    } else {
                        error!(
                            "Unable to deserialize response. Body was: \"{}\"",
                            &payload_str
                        )
                    }
                }
                Err(e) => {
                    // Bubble up to trigger restart
                    error!("MQTT event loop error: {:?}", e);
                    return Err::<(), anyhow::Error>(anyhow::Error::from(e));
                }
                e => {
                    trace!("{:?}", e)
                }
            }
        }
    };

    let ticker = async move {
        loop {
            let _ = status_tick.tick().await;
            // If either publish fails, return the error to trigger restart
            if let Err(e) = local_ha_client
                .publish(&ha_availability_topic, QoS::AtLeastOnce, true, "online")
                .await
            {
                error!(
                    "Error occurred while publishing to {}: {:?}",
                    &ha_availability_topic, e
                );
                return Err::<(), anyhow::Error>(anyhow::Error::from(e));
            }
            if let Err(e) = local_client2
                .publish_and_wait(
                    &status_topic,
                    QoS::AtLeastOnce,
                    false,
                    serde_json::to_string(&MqttRequest::get()).unwrap(),
                )
                .await
            {
                error!(
                    "Error occurred while publishing to {}: {:?}",
                    &status_topic, e
                );
                return Err(e);
            }
        }
    };

    // If either future returns Err, this returns Err immediately.
    tokio::try_join!(reader, ticker).map(|_: (_, _)| ())
}
#[derive(Debug, Clone)]
pub struct ZoneObject {
    pub name: String,
    pub state: String,
    pub last_changed: Option<DateTime<Utc>>,
    pub r#type: String,
    pub zone_number: usize,
    pub attributes: Option<HashMap<String, String>>,
    pub bypass_state: String,
}

pub struct AreaObject {
    pub name: String,
    pub state: Option<String>,
    pub area_number: usize,
}

#[derive(Debug, Clone)]
pub enum MqttCommand {
    SetArea {
        device_id: String,
        area_number: usize,
        action_cmd: ActionCmd,
    },
    SetZoneBypass {
        device_id: String,
        zone_number: usize,
        payload: ZoneBypassRequest,
        action_cmd: ActionCmd,
    },
}

pub struct TopicParseResult {
    pub device_id: String,
    pub command: MqttCommand,
}
pub fn command_topic_parser(topic: &str, payload: &str) -> Option<TopicParseResult> {
    let mut parts: Vec<&str> = topic.split('/').collect();
    parts.resize(10, "");

    if payload.is_empty() {
        error!("Empty payload for topic: {:?}", topic);
        // No command
        return None;
    }

    match (
        parts[0], parts[1], parts[2], parts[3], parts[4], parts[5], parts[6], parts[7], parts[8],
        parts[9],
    ) {
        ("olarm", "device", device_id, "area", area_num, "set", ..) => {
            if let Ok(command) = serde_json::from_str::<ActionCmd>(payload) {
                let area_number = area_num.parse::<usize>().ok()?;
                Some(TopicParseResult {
                    device_id: device_id.to_string(),
                    command: MqttCommand::SetArea {
                        device_id: device_id.to_string(),
                        area_number,
                        action_cmd: command,
                    },
                })
            } else {
                error!(
                    "Unable to deserialize payload: {:?} for topic: {:?}",
                    payload, topic
                );
                None
            }
        }
        //olarm/device/UUID/zone/2/bypass/set
        ("olarm", "device", device_id, "zone", zone_num, "bypass", "set", ..) => {
            if let Ok(request) = serde_json::from_str::<ZoneBypassRequest>(payload) {
                let zone_number = zone_num.parse::<usize>().ok()?;
                Some(TopicParseResult {
                    device_id: device_id.to_string(),
                    command: MqttCommand::SetZoneBypass {
                        device_id: device_id.to_string(),
                        zone_number,
                        payload: request,
                        action_cmd: ActionCmd::ZoneBypass,
                    },
                })
            } else {
                error!(
                    "Unable to deserialize payload: {:?} for topic: {:?}",
                    payload, topic
                );
                None
            }
        }
        _ => None,
    }
}

type SenderMap = Arc<RwLock<HashMap<String, mpsc::Sender<MqttCommand>>>>;
