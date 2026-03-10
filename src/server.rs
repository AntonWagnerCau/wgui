use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
    port: u16,
    bind_addr: &str,
    title: &str,
    favicon: Option<Vec<u8>>,
) -> thread::JoinHandle<()> {
    let bind = bind_addr.to_string();
    let html = HTML_TEMPLATE.replace("__WGUI_TITLE__", &html_escape(title));
    thread::Builder::new()
        .name("wgui-http".into())
        .spawn(move || run_http(shutdown, port, &bind, html, favicon))
        .expect("failed to spawn wgui HTTP thread")
}

/// Spawn the WebSocket server thread. Returns the join handle.
pub fn spawn_ws(
    ws_rx: mpsc::Receiver<Vec<ServerMsg>>,
    edit_tx: mpsc::Sender<(String, crate::element::Value)>,
    port: u16,
    bind_addr: &str,
) -> thread::JoinHandle<()> {
    let bind = bind_addr.to_string();
    thread::Builder::new()
        .name("wgui-ws".into())
        .spawn(move || run_ws(ws_rx, edit_tx, port, &bind))
        .expect("failed to spawn wgui WS thread")
}

fn run_http(shutdown: Arc<AtomicBool>, port: u16, bind_addr: &str, html: String, favicon: Option<Vec<u8>>) {
    let server = tiny_http::Server::http(format!("{bind_addr}:{port}"))
        .unwrap_or_else(|e| panic!("wgui: failed to bind HTTP on port {port}: {e}"));

    log::info!("wgui: serving UI at http://{bind_addr}:{port}");

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
    port: u16,
    bind_addr: &str,
) {
    let listener = TcpListener::bind(format!("{bind_addr}:{port}"))
        .unwrap_or_else(|e| panic!("wgui: failed to bind WS on port {port}: {e}"));

    listener
        .set_nonblocking(true)
        .expect("failed to set WS listener non-blocking");

    log::info!("wgui: WebSocket listening on ws://{bind_addr}:{port}");

    // Local mirror of UI state, updated from incoming ServerMsg
    let mut mirror: Vec<ElementDecl> = Vec::new();

    loop {
        match listener.accept() {
            Ok((stream, _addr)) => {
                log::info!("wgui: new WebSocket client connected");
                handle_ws_client(&ws_rx, &edit_tx, stream, &mut mirror);
                log::info!("wgui: WebSocket client disconnected");
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Try to drain pending messages from game loop while waiting for connections
                while let Ok(msgs) = ws_rx.try_recv() {
                    apply_messages_to_mirror(&mut mirror, &msgs);
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                log::error!("wgui WS accept error: {e}");
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

fn handle_ws_client(
    ws_rx: &mpsc::Receiver<Vec<ServerMsg>>,
    edit_tx: &mpsc::Sender<(String, crate::element::Value)>,
    stream: TcpStream,
    mirror: &mut Vec<ElementDecl>,
) {
    stream
        .set_nonblocking(false)
        .expect("failed to set stream blocking");
    stream
        .set_read_timeout(Some(Duration::from_millis(16)))
        .ok();

    let mut ws = match tungstenite::accept(stream) {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("wgui: WS handshake failed: {e}");
            return;
        }
    };

    // Send full snapshot on connect
    let snapshot = ServerMsg::Snapshot {
        elements: mirror.clone(),
    };
    if let Ok(json) = serde_json::to_string(&snapshot) {
        if ws.send(tungstenite::Message::Text(json.into())).is_err() {
            return;
        }
    }

    loop {
        // Drain pending messages from game loop and apply to mirror
        loop {
            match ws_rx.try_recv() {
                Ok(msgs) => {
                    apply_messages_to_mirror(mirror, &msgs);
                    // Send all messages to client
                    for msg in &msgs {
                        if let Ok(json) = serde_json::to_string(&msg) {
                            if ws
                                .send(tungstenite::Message::Text(json.into()))
                                .is_err()
                            {
                                return;
                            }
                        }
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Game loop dropped the sender, shut down
                    let _ = ws.close(None);
                    return;
                }
            }
        }

        // Read incoming messages from client (non-blocking with timeout)
        match ws.read() {
            Ok(tungstenite::Message::Text(text)) => {
                if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&text) {
                    match client_msg {
                        ClientMsg::Set { id, value } => {
                            // Forward to game loop
                            let _ = edit_tx.send((id, value));
                        }
                    }
                }
            }
            Ok(tungstenite::Message::Close(_)) => {
                let _ = ws.close(None);
                break;
            }
            Ok(_) => {} // ping/pong/binary — ignore
            Err(tungstenite::Error::Io(ref e))
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // No data available, continue loop
            }
            Err(tungstenite::Error::ConnectionClosed) => break,
            Err(tungstenite::Error::AlreadyClosed) => break,
            Err(e) => {
                log::warn!("wgui: WS read error: {e}");
                break;
            }
        }
    }
}

/// Apply a batch of ServerMsg to the local mirror.
fn apply_messages_to_mirror(mirror: &mut Vec<ElementDecl>, msgs: &[ServerMsg]) {
    for msg in msgs {
        match msg {
            ServerMsg::Snapshot { elements } => {
                *mirror = elements.clone();
            }
            ServerMsg::Add { element } => {
                mirror.push(element.clone());
            }
            ServerMsg::Update { id, value, label, meta } => {
                if let Some(elem) = mirror.iter_mut().find(|e| &e.id == id) {
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
                mirror.retain(|e| &e.id != id);
            }
        }
    }
}

/// Find an available port pair (http, ws) starting from `start`.
/// HTTP gets the even port, WS gets the odd port.
pub fn find_port_pair(start: u16, bind_addr: &str) -> (u16, u16) {
    for base in (start..start + 100).step_by(2) {
        let http_ok = TcpListener::bind(format!("{bind_addr}:{base}")).is_ok();
        let ws_ok = TcpListener::bind(format!("{bind_addr}:{}", base + 1)).is_ok();
        if http_ok && ws_ok {
            return (base, base + 1);
        }
    }
    panic!("wgui: could not find available port pair starting from {start}");
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

    #[test]
    fn mirror_add() {
        let mut mirror = vec![];
        let msgs = vec![ServerMsg::Add {
            element: make_decl("a", Value::Bool(true)),
        }];
        apply_messages_to_mirror(&mut mirror, &msgs);
        assert_eq!(mirror.len(), 1);
        assert_eq!(mirror[0].id, "a");
    }

    #[test]
    fn mirror_update() {
        let mut mirror = vec![make_decl("a", Value::Bool(true))];
        let msgs = vec![ServerMsg::Update {
            id: "a".to_string(),
            value: Value::Bool(false),
            label: None,
            meta: None,
        }];
        apply_messages_to_mirror(&mut mirror, &msgs);
        assert_eq!(mirror[0].value, Value::Bool(false));
    }

    #[test]
    fn mirror_remove() {
        let mut mirror = vec![make_decl("a", Value::Bool(true))];
        let msgs = vec![ServerMsg::Remove {
            id: "a".to_string(),
        }];
        apply_messages_to_mirror(&mut mirror, &msgs);
        assert!(mirror.is_empty());
    }

    #[test]
    fn mirror_snapshot() {
        let mut mirror = vec![make_decl("a", Value::Bool(true))];
        let msgs = vec![ServerMsg::Snapshot {
            elements: vec![make_decl("b", Value::Bool(false))],
        }];
        apply_messages_to_mirror(&mut mirror, &msgs);
        assert_eq!(mirror.len(), 1);
        assert_eq!(mirror[0].id, "b");
    }

    #[test]
    fn html_escape_special_chars() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a&b"), "a&amp;b");
        assert_eq!(html_escape("\"hi\""), "&quot;hi&quot;");
        assert_eq!(html_escape("plain"), "plain");
    }
}
