use std::thread::JoinHandle;

use crate::element::{ElementDecl, Value};
use crate::protocol::ServerMsg;
use crate::server;
use crate::state::{self, Shared};
use crate::window::Window;

pub struct Context {
    shared: Shared,
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
        let shared = state::new_shared();

        let http_handle = server::spawn_http(shared.clone(), http_port);
        let ws_handle = server::spawn_ws(shared.clone(), ws_port);

        println!("wgui: UI available at http://127.0.0.1:{http_port}");

        Self {
            shared,
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
        let mut state = self.shared.lock().unwrap();
        state.incoming_edits.remove(id)
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
                    if prev_decl.value != decl.value
                        || prev_decl.meta != decl.meta
                        || prev_decl.kind != decl.kind
                    {
                        outgoing.push(ServerMsg::Update {
                            id: decl.id.clone(),
                            value: decl.value.clone(),
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

        // Push to shared state
        {
            let mut state = self.shared.lock().unwrap();
            state.outgoing_msgs.extend(outgoing);
            state.current_elements = self.current_frame.clone();
        }

        // Swap frames
        self.prev_frame = std::mem::take(&mut self.current_frame);
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        let mut state = self.shared.lock().unwrap();
        state.shutdown = true;
    }
}
