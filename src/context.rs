use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::element::{ElementDecl, ElementId, Value};
use crate::protocol::ServerMsg;
use crate::server;
use crate::window::Window;

/// Options for creating a wgui [`Context`].
pub struct ContextOptions {
    /// Starting port for the HTTP server (WS gets port + 1).
    pub start_port: u16,
    /// Page title shown in the browser tab.
    pub title: String,
    /// Optional PNG favicon bytes. When `None`, no favicon is served.
    pub favicon: Option<Vec<u8>>,
    /// When `true`, bind to `0.0.0.0` (accessible on the network).
    /// When `false`, bind to `127.0.0.1` (localhost only).
    pub public: bool,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            start_port: 9080,
            title: "wgui".to_string(),
            favicon: None,
            public: false,
        }
    }
}

pub struct Context {
    // Sends batched ServerMsg diffs to the WS thread each frame.
    // `None` when no free port was found (headless / degraded mode).
    ws_tx: Option<mpsc::SyncSender<Vec<ServerMsg>>>,
    // Receives browser edits from WS thread (wrapped in Mutex for Sync).
    // `None` when no free port was found.
    edit_rx: Mutex<Option<mpsc::Receiver<(ElementId, Value)>>>,
    // Local cache of pending edits, drained from edit_rx on demand
    incoming_edits: HashMap<ElementId, Value>,
    // Signals HTTP thread to shut down
    shutdown: Arc<AtomicBool>,

    prev_frame: Vec<ElementDecl>,
    current_frame: Vec<ElementDecl>,
    http_port: u16,
    ws_port: u16,
    _http_handle: Option<JoinHandle<()>>,
    _ws_handle: Option<JoinHandle<()>>,
}

impl Context {
    /// Create a new wgui context with default options (localhost, port 9080, title "wgui").
    pub fn new() -> Self {
        Self::with_options(ContextOptions::default())
    }

    /// Create a new wgui context starting port search from `start_port`.
    pub fn with_port(start_port: u16) -> Self {
        Self::with_options(ContextOptions {
            start_port,
            ..Default::default()
        })
    }

    /// Create a new wgui context with the given options.
    /// If no free port pair is found, the context runs in degraded (headless) mode:
    /// UI calls still work locally, but nothing is served over the network.
    pub fn with_options(opts: ContextOptions) -> Self {
        let bind_addr = if opts.public { "0.0.0.0" } else { "127.0.0.1" };

        if let Some((http_port, ws_port)) = server::find_port_pair(opts.start_port, bind_addr) {
            // Create channels for inter-thread communication
            let (ws_tx, ws_rx) = mpsc::sync_channel::<Vec<ServerMsg>>(2);
            let (edit_tx, edit_rx) = mpsc::channel::<(ElementId, Value)>();
            let shutdown = Arc::new(AtomicBool::new(false));

            let http_handle =
                server::spawn_http(shutdown.clone(), http_port, bind_addr, &opts.title, opts.favicon);
            let ws_handle = server::spawn_ws(ws_rx, edit_tx, ws_port, bind_addr, shutdown.clone());

            println!("wgui: UI available at http://{bind_addr}:{http_port}");

            Self {
                ws_tx: Some(ws_tx),
                edit_rx: Mutex::new(Some(edit_rx)),
                incoming_edits: HashMap::new(),
                shutdown,
                prev_frame: Vec::new(),
                current_frame: Vec::new(),
                http_port,
                ws_port,
                _http_handle: Some(http_handle),
                _ws_handle: Some(ws_handle),
            }
        } else {
            log::warn!("wgui: running in headless mode (no free ports)");
            Self {
                ws_tx: None,
                edit_rx: Mutex::new(None),
                incoming_edits: HashMap::new(),
                shutdown: Arc::new(AtomicBool::new(false)),
                prev_frame: Vec::new(),
                current_frame: Vec::new(),
                http_port: 0,
                ws_port: 0,
                _http_handle: None,
                _ws_handle: None,
            }
        }
    }

    /// Returns the HTTP port the UI is served on, or `0` if running headless.
    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    /// Returns the WebSocket port, or `0` if running headless.
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
        let rx = self.edit_rx.lock().unwrap();
        if let Some(ref channel) = *rx {
            while let Ok((elem_id, value)) = channel.try_recv() {
                self.incoming_edits.insert(elem_id, value);
            }
        }
        drop(rx);
        self.incoming_edits.remove(id)
    }

    /// Record an element declaration for the current frame.
    pub(crate) fn declare(&mut self, decl: ElementDecl) {
        self.current_frame.push(decl);
    }

    /// Finish the current frame: reconcile with previous frame, send diffs over WS.
    /// In headless mode this is a no-op.
    pub fn end_frame(&mut self) {
        let outgoing = reconcile(&self.prev_frame, &self.current_frame);

        if !outgoing.is_empty() {
            if let Some(ref tx) = self.ws_tx {
                match tx.try_send(outgoing) {
                    Ok(()) => {}
                    Err(mpsc::TrySendError::Full(_)) => {
                        log::warn!("wgui: WS channel backpressure, skipping frame update");
                    }
                    Err(mpsc::TrySendError::Disconnected(_)) => {
                        log::warn!("wgui: WS thread disconnected");
                    }
                }
            }
        }

        // Swap frames
        self.prev_frame = std::mem::take(&mut self.current_frame);
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
    }
}

/// Compare previous and current frames, producing the minimal set of
/// Add / Update / Remove messages needed to bring a client up to date.
fn reconcile(prev: &[ElementDecl], current: &[ElementDecl]) -> Vec<ServerMsg> {
    let mut outgoing = Vec::new();

    // Build index of previous frame for O(1) lookup
    let prev_index: HashMap<&str, usize> = prev
        .iter()
        .enumerate()
        .map(|(i, d)| (d.id.as_str(), i))
        .collect();

    // Detect added and updated elements
    for decl in current {
        match prev_index.get(decl.id.as_str()) {
            None => {
                outgoing.push(ServerMsg::Add {
                    element: decl.clone(),
                });
            }
            Some(&idx) => {
                let prev_decl = &prev[idx];
                let value_changed = prev_decl.value != decl.value || prev_decl.kind != decl.kind || prev_decl.label != decl.label;
                let meta_changed = prev_decl.meta != decl.meta;
                let label_changed = prev_decl.label != decl.label;
                if value_changed || meta_changed || label_changed {
                    outgoing.push(ServerMsg::Update {
                        id: decl.id.clone(),
                        value: decl.value.clone(),
                        label: if label_changed {
                            Some(decl.label.clone())
                        } else {
                            None
                        },
                        meta: if meta_changed {
                            Some(decl.meta.clone())
                        } else {
                            None
                        },
                    });
                }
            }
        }
    }

    // Detect removed elements
    let current_ids: HashSet<&str> = current.iter().map(|d| d.id.as_str()).collect();
    for prev_decl in prev {
        if !current_ids.contains(prev_decl.id.as_str()) {
            outgoing.push(ServerMsg::Remove {
                id: prev_decl.id.clone(),
            });
        }
    }

    // Detect reorder: same set of IDs per window, different order
    // Only emit if no adds/removes happened (pure reorder)
    let has_structural = outgoing.iter().any(|m| matches!(m, ServerMsg::Add { .. } | ServerMsg::Remove { .. }));
    if !has_structural && !prev.is_empty() {
        // Group by window and check order
        let mut prev_order: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut curr_order: HashMap<&str, Vec<&str>> = HashMap::new();
        for d in prev {
            prev_order.entry(d.window.as_ref()).or_default().push(&d.id);
        }
        for d in current {
            curr_order.entry(d.window.as_ref()).or_default().push(&d.id);
        }
        for (win, curr_ids) in &curr_order {
            if let Some(prev_ids) = prev_order.get(win) {
                if prev_ids.len() == curr_ids.len() && prev_ids != curr_ids {
                    outgoing.push(ServerMsg::Reorder {
                        window: win.to_string(),
                        ids: curr_ids.iter().map(|s| s.to_string()).collect(),
                    });
                }
            }
        }
    }

    outgoing
}

const _: () = {
    fn _assert_send_sync<T: Send + Sync>() {}
    fn _check() { _assert_send_sync::<Context>(); }
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ElementKind, ElementMeta, Value};
    use std::sync::Arc;

    fn make_decl(id: &str, value: Value) -> ElementDecl {
        ElementDecl {
            id: id.to_string(),
            kind: ElementKind::Label,
            label: id.to_string(),
            value,
            meta: ElementMeta::default(),
            window: Arc::from("test"),
        }
    }

    #[test]
    fn reconcile_detects_additions() {
        let msgs = reconcile(&[], &[make_decl("a", Value::Bool(true))]);
        assert_eq!(msgs.len(), 1);
        assert!(matches!(&msgs[0], ServerMsg::Add { element } if element.id == "a"));
    }

    #[test]
    fn reconcile_detects_removals() {
        let msgs = reconcile(&[make_decl("a", Value::Bool(true))], &[]);
        assert_eq!(msgs.len(), 1);
        assert!(matches!(&msgs[0], ServerMsg::Remove { id } if id == "a"));
    }

    #[test]
    fn reconcile_detects_updates() {
        let prev = vec![make_decl("a", Value::Bool(true))];
        let current = vec![make_decl("a", Value::Bool(false))];
        let msgs = reconcile(&prev, &current);
        assert_eq!(msgs.len(), 1);
        assert!(matches!(&msgs[0], ServerMsg::Update { id, .. } if id == "a"));
    }

    #[test]
    fn reconcile_unchanged() {
        let prev = vec![make_decl("a", Value::Bool(true))];
        let current = vec![make_decl("a", Value::Bool(true))];
        assert!(reconcile(&prev, &current).is_empty());
    }

    #[test]
    fn reconcile_mixed() {
        let prev = vec![
            make_decl("keep", Value::Bool(true)),
            make_decl("update", Value::Float(1.0)),
            make_decl("remove", Value::Bool(false)),
        ];
        let current = vec![
            make_decl("keep", Value::Bool(true)),
            make_decl("update", Value::Float(2.0)),
            make_decl("add", Value::Bool(true)),
        ];
        let msgs = reconcile(&prev, &current);
        assert_eq!(msgs.len(), 3); // update + remove + add
    }
}
