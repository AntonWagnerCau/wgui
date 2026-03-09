use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::element::{ElementDecl, ElementId, Value};
use crate::protocol::ServerMsg;

pub struct SharedState {
    /// Pending value changes from browser, consumed by next frame's widget calls.
    pub incoming_edits: HashMap<ElementId, Value>,

    /// Outgoing messages queued by reconciliation, drained by the WS thread.
    pub outgoing_msgs: Vec<ServerMsg>,

    /// Current full element state (insertion-ordered), used for snapshots on reconnect.
    pub current_elements: Vec<ElementDecl>,

    /// Set to true when a new client connects and needs a full snapshot.
    pub needs_snapshot: bool,

    /// Set to true to signal server threads to shut down.
    pub shutdown: bool,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            incoming_edits: HashMap::new(),
            outgoing_msgs: Vec::new(),
            current_elements: Vec::new(),
            needs_snapshot: false,
            shutdown: false,
        }
    }
}

pub type Shared = Arc<Mutex<SharedState>>;

pub fn new_shared() -> Shared {
    Arc::new(Mutex::new(SharedState::new()))
}
