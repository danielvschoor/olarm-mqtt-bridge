use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub logging: LoggingConfig,
    pub olarm: OlarmConfig,
    pub home_assistant: HomeAssistantConfig,
    pub intervals: IntervalConfig,
    pub limits: LimitsConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub directory: String,
    pub debug_file: String,
    pub info_file: String,
    pub warn_file: String,
    pub error_file: String,
    pub console_level: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OlarmConfig {
    pub api_token: String,
    pub username: String,
    pub password: String,
    pub broker_url: String,
    pub broker_port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HomeAssistantConfig {
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub client_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IntervalConfig {
    pub status_tick_seconds: u64,
    pub reconnect_delay_seconds: u64,
    pub mqtt_keep_alive_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LimitsConfig {
    pub mqtt_queue_size: usize,
    pub command_channel_size: usize,
    pub max_concurrent_commands: usize,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_example(path: &str) -> Result<()> {
        let example_config = Config {
            logging: LoggingConfig {
                directory: "./logs".to_string(),
                debug_file: "log_debug.log".to_string(),
                info_file: "log_info.log".to_string(),
                warn_file: "log_warn.log".to_string(),
                error_file: "log_error.log".to_string(),
                console_level: "debug".to_string(),
            },
            olarm: OlarmConfig {
                api_token: "REPLACE_WITH_YOUR_API_TOKEN".to_string(),
                username: "REPLACE_WITH_YOUR_OLARM_USERNAME".to_string(),
                password: "REPLACE_WITH_YOUR_OLARM_PASSWORD".to_string(),
                broker_url: "wss://mqtt-ws.olarm.com:443".to_string(),
                broker_port: 443,
            },
            home_assistant: HomeAssistantConfig {
                mqtt_host: "192.168.1.40".to_string(),
                mqtt_port: 2883,
                mqtt_username: "homeassistant".to_string(),
                mqtt_password: "REPLACE_WITH_YOUR_HOMEASSISTANT_MQTT_PASSWORD".to_string(),
                client_id: "olarm-forwarder".to_string(),
            },
            intervals: IntervalConfig {
                status_tick_seconds: 10,
                reconnect_delay_seconds: 5,
                mqtt_keep_alive_seconds: 30,
            },
            limits: LimitsConfig {
                mqtt_queue_size: 100,
                command_channel_size: 10,
                max_concurrent_commands: 10,
            },
        };

        let toml_content = toml::to_string_pretty(&example_config)?;
        fs::write(path, toml_content)?;
        Ok(())
    }
}