use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use indexmap::IndexMap;

use crate::element::ElementDecl;
use crate::protocol::{ClientMsg, ServerMsg};

const HTML_TEMPLATE: &str = include_str!("frontend.html");

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Spawn the HTTP server thread. Returns the join handle.
pub fn spawn_http(
    shutdown: Arc<AtomicBool>,
    listener: TcpListener,
    title: &str,
    favicon: Option<Vec<u8>>,
) -> thread::JoinHandle<()> {
    let html = HTML_TEMPLATE.replace("__WGUI_TITLE__", &html_escape(title));
    thread::Builder::new()
        .name("wgui-http".into())
        .spawn(move || run_http(shutdown, listener, html, favicon))
        .expect("failed to spawn wgui HTTP thread")
}

/// Spawn the WebSocket server thread. Returns the join handle.
pub fn spawn_ws(
    ws_rx: mpsc::Receiver<Vec<ServerMsg>>,
    edit_tx: mpsc::Sender<(String, crate::element::Value)>,
    listener: TcpListener,
    shutdown: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("wgui-ws".into())
        .spawn(move || run_ws(ws_rx, edit_tx, listener, shutdown))
        .expect("failed to spawn wgui WS thread")
}

fn run_http(shutdown: Arc<AtomicBool>, listener: TcpListener, html: String, favicon: Option<Vec<u8>>) {
    let addr = listener.local_addr().expect("wgui: HTTP listener has no local addr");
    let server = tiny_http::Server::from_listener(listener, None)
        .expect("wgui: failed to create HTTP server from listener");

    log::info!("wgui: serving UI at http://{addr}");

    loop {
        if shutdown.load(Ordering::Acquire) {
            break;
        }

        // Use a timeout so we can check shutdown periodically
        match server.recv_timeout(Duration::from_millis(200)) {
            Ok(Some(request)) => {
                match request.url() {
                    "/" | "/index.html" => {
                        let response = tiny_http::Response::from_string(&html)
                            .with_header(
                                "Content-Type: text/html; charset=utf-8"
                                    .parse::<tiny_http::Header>()
                                    .unwrap(),
                            );
                        let _ = request.respond(response);
                    }
                    "/favicon.png" => {
                        if let Some(ref icon) = favicon {
                            let response = tiny_http::Response::from_data(icon.clone())
                                .with_header(
                                    "Content-Type: image/png"
                                        .parse::<tiny_http::Header>()
                                        .unwrap(),
                                );
                            let _ = request.respond(response);
                        } else {
                            let _ = request.respond(
                                tiny_http::Response::from_string("404 Not Found")
                                    .with_status_code(404),
                            );
                        }
                    }
                    _ => {
                        let _ = request.respond(
                            tiny_http::Response::from_string("404 Not Found")
                                .with_status_code(404),
                        );
                    }
                }
            }
            Ok(None) => {} // timeout, loop again
            Err(e) => {
                log::error!("wgui HTTP error: {e}");
                break;
            }
        }
    }
}

fn run_ws(
    ws_rx: mpsc::Receiver<Vec<ServerMsg>>,
    edit_tx: mpsc::Sender<(String, crate::element::Value)>,
    listener: TcpListener,
    shutdown: Arc<AtomicBool>,
) {
    let addr = listener.local_addr().expect("wgui: WS listener has no local addr");

    listener
        .set_nonblocking(true)
        .expect("failed to set WS listener non-blocking");

    log::info!("wgui: WebSocket listening on ws://{addr}");

    // Local mirror of UI state, updated from incoming ServerMsg
    let mut mirror: IndexMap<String, ElementDecl> = IndexMap::new();
    let mut clients: Vec<tungstenite::WebSocket<TcpStream>> = Vec::new();

    loop {
        // Check shutdown signal
        if shutdown.load(Ordering::Acquire) {
            log::info!("wgui: WS server shutting down");
            for ws in &mut clients {
                let _ = ws.close(None);
            }
            break;
        }

        // Accept new connections
        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    stream.set_nonblocking(false).ok();
                    stream.set_read_timeout(Some(Duration::from_millis(1))).ok();
                    match tungstenite::accept(stream) {
                        Ok(mut ws) => {
                            log::info!("wgui: new WebSocket client connected (total: {})", clients.len() + 1);
                            // Send snapshot to new client
                            let snapshot = ServerMsg::Snapshot {
                                elements: mirror.values().cloned().collect(),
                            };
                            if let Ok(json) = serde_json::to_string(&snapshot) {
                                let _ = ws.send(tungstenite::Message::Text(json.into()));
                            }
                            ws.get_ref().set_read_timeout(Some(Duration::from_millis(1))).ok();
                            clients.push(ws);
                        }
                        Err(e) => log::error!("wgui: WS handshake failed: {e}"),
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    log::error!("wgui WS accept error: {e}");
                    break;
                }
            }
        }

        // Drain pending messages from game loop, apply to mirror, broadcast
        let mut got_msgs = false;
        let mut all_msgs = Vec::new();
        loop {
            match ws_rx.try_recv() {
                Ok(msgs) => {
                    got_msgs = true;
                    apply_messages_to_mirror(&mut mirror, &msgs);
                    all_msgs.extend(msgs);
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    for ws in &mut clients {
                        let _ = ws.close(None);
                    }
                    return;
                }
            }
        }

        // Send all messages as a single batch (one WebSocket frame)
        if !all_msgs.is_empty() {
            if let Ok(json) = serde_json::to_string(&all_msgs) {
                let text = tungstenite::Message::Text(json.into());
                clients.retain_mut(|ws| {
                    ws.send(text.clone()).is_ok()
                });
            }
        }

        // Read incoming messages from all clients
        let mut broadcast_msgs: Vec<String> = Vec::new();
        clients.retain_mut(|ws| {
            loop {
                match ws.read() {
                    Ok(tungstenite::Message::Text(text)) => {
                        if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&text) {
                            match client_msg {
                                ClientMsg::Set { id, value } => {
                                    let _ = edit_tx.send((id, value));
                                }
                                ClientMsg::ReorderWindow { from, to } => {
                                    // Reorder windows: move 'from' window before 'to' window
                                    let window_names: Vec<String> = mirror
                                        .values()
                                        .map(|e| e.window.to_string())
                                        .collect::<std::collections::HashSet<_>>()
                                        .into_iter()
                                        .collect();
                                    
                                    // For now, just log the reorder request
                                    // Full implementation would require persisting window order
                                    log::info!("wgui: window reorder request: {} -> {}", from, to);
                                    
                                    // Build reorder message to broadcast to all clients
                                    // This tells clients to reorder their windows
                                    let reorder_msg = serde_json::json!({
                                        "type": "reorder_windows",
                                        "order": window_names
                                    });
                                    broadcast_msgs.push(reorder_msg.to_string());
                                }
                            }
                        }
                    }
                    Ok(tungstenite::Message::Close(_)) => {
                        let _ = ws.close(None);
                        log::info!("wgui: WebSocket client disconnected");
                        return false;
                    }
                    Ok(_) => {} // ping/pong/binary
                    Err(tungstenite::Error::Io(ref e))
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        return true; // No more data, keep client
                    }
                    Err(tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed) => {
                        log::info!("wgui: WebSocket client disconnected");
                        return false;
                    }
                    Err(e) => {
                        log::warn!("wgui: WS read error: {e}");
                        return false;
                    }
                }
            }
        });
        
        // Broadcast any messages collected from client handling
        for msg_text in broadcast_msgs {
            let text = tungstenite::Message::Text(msg_text.into());
            clients.retain_mut(|ws| ws.send(text.clone()).is_ok());
        }

        if !got_msgs && clients.is_empty() {
            thread::sleep(Duration::from_millis(50));
        } else if !got_msgs {
            thread::sleep(Duration::from_millis(8));
        }
    }
}

/// Apply a batch of ServerMsg to the local mirror.
fn apply_messages_to_mirror(mirror: &mut IndexMap<String, ElementDecl>, msgs: &[ServerMsg]) {
    for msg in msgs {
        match msg {
            ServerMsg::Snapshot { elements } => {
                mirror.clear();
                for elem in elements {
                    mirror.insert(elem.id.clone(), elem.clone());
                }
            }
            ServerMsg::Add { element } => {
                mirror.insert(element.id.clone(), element.clone());
            }
            ServerMsg::Update { id, value, label, meta } => {
                if let Some(elem) = mirror.get_mut(id) {
                    elem.value = value.clone();
                    if let Some(l) = label {
                        elem.label = l.clone();
                    }
                    if let Some(m) = meta {
                        elem.meta = m.clone();
                    }
                }
            }
            ServerMsg::Remove { id } => {
                mirror.shift_remove(id);
            }
            ServerMsg::Reorder { ids, .. } => {
                let mut new_mirror = IndexMap::with_capacity(mirror.len());
                for id in ids {
                    if let Some(elem) = mirror.get(id) {
                        new_mirror.insert(id.clone(), elem.clone());
                    }
                }
                // Keep any elements not in the reorder list at the end
                for (id, elem) in mirror.iter() {
                    if !new_mirror.contains_key(id) {
                        new_mirror.insert(id.clone(), elem.clone());
                    }
                }
                *mirror = new_mirror;
            }
        }
    }
}

/// Find an available port pair (http, ws) starting from `start`.
/// HTTP gets the even port, WS gets the odd port.
/// Returns the two bound listeners so there is no race condition between
/// scanning and serving.
/// Returns `None` if no free pair is found after scanning 1000 ports.
pub fn find_port_pair(start: u16, bind_addr: &str) -> Option<(TcpListener, TcpListener)> {
    for base in (start..start.saturating_add(1000)).step_by(2) {
        let http_ok = TcpListener::bind(format!("{bind_addr}:{base}"));
        let ws_ok = TcpListener::bind(format!("{bind_addr}:{}", base + 1));
        if let (Ok(http), Ok(ws)) = (http_ok, ws_ok) {
            return Some((http, ws));
        }
    }
    log::warn!("wgui: could not find available port pair starting from {start}");
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ElementDecl, ElementKind, ElementMeta, Value};
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

    fn into_mirror(decls: Vec<ElementDecl>) -> IndexMap<String, ElementDecl> {
        decls.into_iter().map(|d| (d.id.clone(), d)).collect()
    }

    #[test]
    fn mirror_add() {
        let mut mirror = IndexMap::new();
        let msgs = vec![ServerMsg::Add {
            element: make_decl("a", Value::Bool(true)),
        }];
        apply_messages_to_mirror(&mut mirror, &msgs);
        assert_eq!(mirror.len(), 1);
        assert_eq!(mirror["a"].id, "a");
    }

    #[test]
    fn mirror_update() {
        let mut mirror = into_mirror(vec![make_decl("a", Value::Bool(true))]);
        let msgs = vec![ServerMsg::Update {
            id: "a".to_string(),
            value: Value::Bool(false),
            label: None,
            meta: None,
        }];
        apply_messages_to_mirror(&mut mirror, &msgs);
        assert_eq!(mirror["a"].value, Value::Bool(false));
    }

    #[test]
    fn mirror_remove() {
        let mut mirror = into_mirror(vec![make_decl("a", Value::Bool(true))]);
        let msgs = vec![ServerMsg::Remove {
            id: "a".to_string(),
        }];
        apply_messages_to_mirror(&mut mirror, &msgs);
        assert!(mirror.is_empty());
    }

    #[test]
    fn mirror_snapshot() {
        let mut mirror = into_mirror(vec![make_decl("a", Value::Bool(true))]);
        let msgs = vec![ServerMsg::Snapshot {
            elements: vec![make_decl("b", Value::Bool(false))],
        }];
        apply_messages_to_mirror(&mut mirror, &msgs);
        assert_eq!(mirror.len(), 1);
        assert_eq!(mirror["b"].id, "b");
    }

    #[test]
    fn html_escape_special_chars() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a&b"), "a&amp;b");
        assert_eq!(html_escape("\"hi\""), "&quot;hi&quot;");
        assert_eq!(html_escape("plain"), "plain");
    }
}
