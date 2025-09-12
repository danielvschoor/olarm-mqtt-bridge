use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Action {
    #[serde(rename = "actionId")]
    pub action_id: String,
    #[serde(rename = "actionCmd")]
    pub action_cmd: String,
    #[serde(rename = "actionNum")]
    pub action_num: i64,
    #[serde(rename = "actionCreated")]
    pub action_created: i64,
    #[serde(rename = "actionStatus")]
    pub action_status: String,
    #[serde(rename = "actionMsg")]
    pub action_msg: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "deviceName")]
    pub device_name: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "userFullname")]
    pub user_fullname: String,
    #[serde(rename = "userEmail")]
    pub user_email: String,
}
