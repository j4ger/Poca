use serde::{Deserialize, Serialize};
use serde_repr::*;

use crate::synchronizable::Synchronizable;

#[derive(Debug, Clone)]
pub enum Message {
    Set {
        key: String,
        data: Box<dyn Synchronizable>,
    },
    Get {
        //TODO: use client-id to only reply to the client that requested the data
        key: String,
        data: Box<dyn Synchronizable>,
    },
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
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
