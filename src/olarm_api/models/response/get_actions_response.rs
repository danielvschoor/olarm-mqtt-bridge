use crate::olarm_api::models::action::Action;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetActionsResponse {
    pub actions: Vec<Action>,
}
