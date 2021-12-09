use serde::{Deserialize, Serialize};

use crate::synchronizable::Synchronizable;

#[derive(Debug, Clone)]
pub enum Message {
    Set {
        key: String,
        data: Box<dyn Synchronizable>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WSMessageType {
    Set = 1,
    Emit = 2,
    Get = 3,
    Error = 4,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WSMessage {
    pub message_type: WSMessageType,
    pub key: Option<String>,
    pub data: Option<String>,
}
