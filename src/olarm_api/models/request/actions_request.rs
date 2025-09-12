use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize)]
pub struct ActionsRequest {
    #[serde(rename = "actionCmd")]
    pub action_cmd: ActionCmd,
    #[serde(rename = "actionNum")]
    pub action_num: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ActionCmd {
    // Zones
    #[serde(rename = "zone-bypass")]
    ZoneBypass,
    #[serde(rename = "zone-unbypass")]
    ZoneUnBypass,

    // Areas
    #[serde(rename = "area-arm")]
    AreaArm,
    #[serde(rename = "area-sleep")]
    AreaSleep,
    #[serde(rename = "area-stay")]
    AreaStay,
    #[serde(rename = "area-disarm")]
    AreaDisarm,

    // PGM
    #[serde(rename = "pgm-close")]
    PgmClose,
    #[serde(rename = "pgm-open")]
    PgmOpen,
    #[serde(rename = "pgm-pulse")]
    PgmPulse,

    // Ukey
    #[serde(rename = "ukey-activate")]
    UkeyActivate,
}

impl Display for ActionCmd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Get the serde rename value by deserializing to a Value first
        let json_value = serde_json::to_value(self).map_err(|_| std::fmt::Error)?;

        if let Some(s) = json_value.as_str() {
            write!(f, "\"{}\"", s)
        } else {
            Err(std::fmt::Error)
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MqttRequest {
    pub method: MqttRequestMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<String>>,
}

impl MqttRequest {
    pub fn get()->Self{
        Self{
            method: MqttRequestMethod::GET,
            data: None,
        }
    }
    pub fn from(request: ActionsRequest) -> Option<Self> {
        let action_cmd = match request.action_cmd {
            ActionCmd::ZoneBypass => "bypass",
            ActionCmd::AreaArm => "arm",
            ActionCmd::AreaSleep => "sleep",
            ActionCmd::AreaStay => "stay",
            ActionCmd::AreaDisarm => "disarm",
            _ => return None,
        };
        Some(Self {
            method: MqttRequestMethod::POST,
            data: Some(vec![action_cmd.to_string(), request.action_num]),
        })
    }
}
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum MqttRequestMethod {
    #[serde(rename = "GET")]
    GET,
    #[serde(rename = "POST")]
    POST,
}
