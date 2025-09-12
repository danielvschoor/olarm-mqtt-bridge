use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Power {
    #[serde(rename = "AC")]
    pub ac: String,
    #[serde(rename = "Batt")]
    pub batt: String,
}
