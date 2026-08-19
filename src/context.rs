use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::element::{ElementDecl, ElementId, Value};
use crate::protocol::ServerMsg;
use crate::server;
use crate::window::{Tab, Window};

/// Strength of the accent glow on sliders, progress bars, status dots,
/// stat values and chart lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glow {
    Off,
    Subtle,
    Strong,
}

impl Default for Glow {
    fn default() -> Self {
        Glow::Off
    }
}

impl Glow {
    pub fn as_str(&self) -> &'static str {
        match self {
            Glow::Off => "off",
            Glow::Subtle => "subtle",
            Glow::Strong => "strong",
        }
    }
}

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
    /// Strength of the accent glow throughout the UI.
    pub glow: Glow,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            start_port: 9080,
            title: "wgui".to_string(),
            favicon: None,
            public: false,
            glow: Glow::default(),
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

        if let Some((http_listener, ws_listener)) = server::find_port_pair(opts.start_port, bind_addr) {
            let http_port = http_listener.local_addr().map(|a| a.port()).unwrap_or(0);
            let ws_port = ws_listener.local_addr().map(|a| a.port()).unwrap_or(0);

            // Create channels for inter-thread communication
            let (ws_tx, ws_rx) = mpsc::sync_channel::<Vec<ServerMsg>>(2);
            let (edit_tx, edit_rx) = mpsc::channel::<(ElementId, Value)>();
            let shutdown = Arc::new(AtomicBool::new(false));

            let http_handle =
                server::spawn_http(shutdown.clone(), http_listener, &opts.title, opts.favicon, opts.glow);
            let ws_handle = server::spawn_ws(ws_rx, edit_tx, ws_listener, shutdown.clone());

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
    /// Windows created here show on every tab.
    pub fn window(&mut self, name: &str) -> Window<'_> {
        Window::new(name.to_string(), None, self)
    }

    /// Get or create a named tab (page). Windows created via `Tab::window()`
    /// only show while that tab is active in the browser.
    pub fn tab(&mut self, name: &str) -> Tab<'_> {
        Tab::new(name.to_string(), self)
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
                        log::debug!("wgui: WS channel backpressure, skipping frame update");
                    }
                    Err(mpsc::TrySendError::Disconnected(_)) => {
                        log::debug!("wgui: WS thread disconnected");
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
        // Join the server threads so their listeners (and the ports) are fully
        // released before any replacement Context tries to bind. Without this,
        // a quick in-process restart can drift to the next port pair while an
        // already-open browser tab keeps pointing at the old port and never
        // reconnects. Threads observe `shutdown` within ~200ms (HTTP poll).
        if let Some(handle) = self._http_handle.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self._ws_handle.take() {
            let _ = handle.join();
        }
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

    // Elements whose tab changed are reissued as Remove + Add; the client
    // appends them like fresh adds, so order prediction treats them as new.
    let mut tab_moved: HashSet<&str> = HashSet::new();

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
                if prev_decl.tab != decl.tab {
                    tab_moved.insert(decl.id.as_str());
                    outgoing.push(ServerMsg::Remove { id: decl.id.clone() });
                    outgoing.push(ServerMsg::Add {
                        element: decl.clone(),
                    });
                    continue;
                }
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

    // Re-assert per-window element order when the client's post-batch order
    // would differ from the declared order. The client applies `Add` by
    // appending and `Remove` by deletion, preserving order otherwise — so after
    // a batch its order is the surviving previous elements followed by the newly
    // added ones. An element first declared *between* existing ones (e.g. a
    // table row that only appears once its data exists, landing under a later
    // section) therefore ends up in the wrong place; a Reorder carrying the
    // declared order fixes it. Pure reorders fall out of the same comparison,
    // and a plain append (new element declared last) predicts correctly and
    // emits nothing.
    let mut prev_order: HashMap<&str, Vec<&str>> = HashMap::new();
    for d in prev {
        prev_order.entry(d.window.as_ref()).or_default().push(&d.id);
    }
    let mut curr_windows: Vec<&str> = Vec::new();
    let mut curr_order: HashMap<&str, Vec<&str>> = HashMap::new();
    for d in current {
        let w = d.window.as_ref();
        if !curr_order.contains_key(w) {
            curr_windows.push(w);
        }
        curr_order.entry(w).or_default().push(&d.id);
    }
    let empty: Vec<&str> = Vec::new();
    for win in curr_windows {
        let desired = &curr_order[win];
        let prev_ids = prev_order.get(win).unwrap_or(&empty);
        let curr_set: HashSet<&str> = desired.iter().copied().collect();
        let prev_set: HashSet<&str> = prev_ids.iter().copied().collect();
        // Predicted client order: surviving previous ids, then the new ones.
        let mut predicted: Vec<&str> = prev_ids
            .iter()
            .copied()
            .filter(|id| curr_set.contains(id) && !tab_moved.contains(id))
            .collect();
        predicted.extend(
            desired
                .iter()
                .copied()
                .filter(|id| !prev_set.contains(id) || tab_moved.contains(id)),
        );
        if &predicted != desired {
            outgoing.push(ServerMsg::Reorder {
                window: win.to_string(),
                ids: desired.iter().map(|s| s.to_string()).collect(),
            });
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
            tab: None,
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
    fn reconcile_tab_move_reissues_element() {
        let prev = vec![make_decl("a", Value::Bool(true))];
        let mut moved = make_decl("a", Value::Bool(true));
        moved.tab = Some(Arc::from("Settings"));
        let msgs = reconcile(&prev, &[moved]);
        assert!(matches!(&msgs[0], ServerMsg::Remove { id } if id == "a"));
        assert!(matches!(&msgs[1], ServerMsg::Add { element } if element.tab.as_deref() == Some("Settings")));
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
