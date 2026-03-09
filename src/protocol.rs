use serde::{Deserialize, Serialize};

use crate::element::{ElementDecl, ElementId, Value};

/// Messages sent from server to browser client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMsg {
    /// Full UI state — sent on connect/reconnect.
    #[serde(rename = "snapshot")]
    Snapshot { elements: Vec<ElementDecl> },

    /// A single element's value changed from the Rust side.
    #[serde(rename = "update")]
    Update {
        id: ElementId,
        value: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        meta: Option<crate::element::ElementMeta>,
    },

    /// An element was added.
    #[serde(rename = "add")]
    Add { element: ElementDecl },

    /// An element was removed (no longer declared this frame).
    #[serde(rename = "remove")]
    Remove { id: ElementId },
}

/// Messages sent from browser client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMsg {
    /// User changed a value in the browser UI.
    #[serde(rename = "set")]
    Set { id: ElementId, value: Value },
}
