use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ZoneBypassRequest {
    pub bypass: bool
}
impl ZoneBypassRequest {
    pub fn new(bypass: bool) -> Self {
        Self { bypass }
    }
    
    pub fn to_payload(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    
}

