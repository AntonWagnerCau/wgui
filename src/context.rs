use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread::JoinHandle;

use crate::element::{ElementDecl, ElementId, Value};
use crate::protocol::ServerMsg;
use crate::server;
use crate::window::Window;

pub struct Context {
    // Sends batched ServerMsg diffs to the WS thread each frame
    ws_tx: mpsc::SyncSender<Vec<ServerMsg>>,
    // Receives browser edits from WS thread
    edit_rx: mpsc::Receiver<(ElementId, Value)>,
    // Local cache of pending edits, drained from edit_rx on demand
    incoming_edits: HashMap<ElementId, Value>,
    // Signals HTTP thread to shut down
    shutdown: Arc<AtomicBool>,

    prev_frame: Vec<ElementDecl>,
    current_frame: Vec<ElementDecl>,
    http_port: u16,
    ws_port: u16,
    _http_handle: JoinHandle<()>,
    _ws_handle: JoinHandle<()>,
}

impl Context {
    /// Create a new wgui context. Starts HTTP + WS servers on localhost.
    /// Prints the URL to stdout and logs via `log` crate.
    pub fn new() -> Self {
        Self::with_port(9080)
    }

    /// Create a new wgui context starting port search from `start_port`.
    pub fn with_port(start_port: u16) -> Self {
        let (http_port, ws_port) = server::find_port_pair(start_port);

        // Create channels for inter-thread communication
        let (ws_tx, ws_rx) = mpsc::sync_channel::<Vec<ServerMsg>>(2);
        let (edit_tx, edit_rx) = mpsc::channel::<(ElementId, Value)>();
        let shutdown = Arc::new(AtomicBool::new(false));

        let http_handle = server::spawn_http(shutdown.clone(), http_port);
        let ws_handle = server::spawn_ws(ws_rx, edit_tx, ws_port);

        println!("wgui: UI available at http://127.0.0.1:{http_port}");

        Self {
            ws_tx,
            edit_rx,
            incoming_edits: HashMap::new(),
            shutdown,
            prev_frame: Vec::new(),
            current_frame: Vec::new(),
            http_port,
            ws_port,
            _http_handle: http_handle,
            _ws_handle: ws_handle,
        }
    }

    /// Returns the HTTP port the UI is served on.
    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    /// Returns the WebSocket port.
    pub fn ws_port(&self) -> u16 {
        self.ws_port
    }

    /// Get or create a named window. Call widget methods on the returned `Window`.
    pub fn window(&mut self, name: &str) -> Window<'_> {
        Window::new(name.to_string(), self)
    }

    /// Consume a pending browser edit for the given element id, if any.
    pub(crate) fn consume_edit(&mut self, id: &str) -> Option<Value> {
        // Drain all pending edits from the channel into the local cache
        while let Ok((elem_id, value)) = self.edit_rx.try_recv() {
            self.incoming_edits.insert(elem_id, value);
        }
        self.incoming_edits.remove(id)
    }

    /// Record an element declaration for the current frame.
    pub(crate) fn declare(&mut self, decl: ElementDecl) {
        self.current_frame.push(decl);
    }

    /// Number of elements declared so far this frame (for generating unique separator ids).
    pub(crate) fn current_frame_len(&self) -> usize {
        self.current_frame.len()
    }

    /// Finish the current frame: reconcile with previous frame, send diffs over WS.
    pub fn end_frame(&mut self) {
        let mut outgoing = Vec::new();

        // Detect added and updated elements
        for decl in &self.current_frame {
            let prev = self.prev_frame.iter().find(|p| p.id == decl.id);
            match prev {
                None => {
                    // New element
                    outgoing.push(ServerMsg::Add {
                        element: decl.clone(),
                    });
                }
                Some(prev_decl) => {
                    // Check if value changed from Rust side
                    let value_changed = prev_decl.value != decl.value || prev_decl.kind != decl.kind;
                    let meta_changed = prev_decl.meta != decl.meta;
                    if value_changed || meta_changed {
                        outgoing.push(ServerMsg::Update {
                            id: decl.id.clone(),
                            value: decl.value.clone(),
                            meta: if meta_changed { Some(decl.meta.clone()) } else { None },
                        });
                    }
                }
            }
        }

        // Detect removed elements
        for prev_decl in &self.prev_frame {
            if !self.current_frame.iter().any(|d| d.id == prev_decl.id) {
                outgoing.push(ServerMsg::Remove {
                    id: prev_decl.id.clone(),
                });
            }
        }

        // Send outgoing messages to WS thread
        if !outgoing.is_empty() {
            match self.ws_tx.try_send(outgoing) {
                Ok(()) => {}
                Err(mpsc::TrySendError::Full(_)) => {
                    log::warn!("wgui: WS channel backpressure, skipping frame update");
                }
                Err(mpsc::TrySendError::Disconnected(_)) => {
                    log::warn!("wgui: WS thread disconnected");
                }
            }
        }

        // Swap frames
        self.prev_frame = std::mem::take(&mut self.current_frame);
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}
