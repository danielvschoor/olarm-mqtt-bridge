use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceProfile {
    #[serde(rename = "areasLimit")]
    pub areas_limit: i64,
    #[serde(rename = "areasLabels")]
    pub areas_labels: Vec<String>,
    #[serde(rename = "zonesLimit")]
    pub zones_limit: i64,
    #[serde(rename = "zonesLabels")]
    pub zones_labels: Vec<String>,
    #[serde(rename = "zonesTypes")]
    pub zones_types: Vec<i64>,
    #[serde(rename = "pgmLimit")]
    pub pgm_limit: i64,
    #[serde(rename = "pgmLabels")]
    pub pgm_labels: Vec<String>,
    #[serde(rename = "pgmControl")]
    pub pgm_control: Vec<String>,
    #[serde(rename = "ukeysLimit")]
    pub ukeys_limit: i64,
    #[serde(rename = "ukeysLabels")]
    pub ukeys_labels: Vec<String>,
    #[serde(rename = "ukeysControl")]
    pub ukeys_control: Vec<i64>,
    #[serde(rename = "pgmObLimit")]
    pub pgm_ob_limit: i64,
    #[serde(rename = "pgmObLabels")]
    pub pgm_ob_labels: Vec<String>,
    #[serde(rename = "pgmObControl")]
    pub pgm_ob_control: Vec<String>,
    #[serde(rename = "doorsLimit")]
    pub doors_limit: i64,
    #[serde(rename = "doorsLabels")]
    pub doors_labels: Vec<String>,
    pub ver: i64,
}
