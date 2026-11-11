use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

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
    glow: crate::context::Glow,
    frame_clock: FrameClock,
) -> thread::JoinHandle<()> {
    let html = HTML_TEMPLATE
        .replace("__WGUI_TITLE__", &html_escape(title))
        .replace("__WGUI_GLOW__", glow.as_str());
    let site = Arc::new(Site { html, favicon, frame_clock });
    thread::Builder::new()
        .name("wgui-http".into())
        .spawn(move || run_http(shutdown, listener, site))
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

/// Time of the owning `Context`'s last `end_frame`.
#[derive(Clone)]
pub struct FrameClock {
    start: Instant,
    last_frame_ms: Arc<AtomicU64>,
}

impl FrameClock {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            last_frame_ms: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn mark_frame(&self) {
        self.last_frame_ms
            .store(self.start.elapsed().as_millis() as u64, Ordering::Relaxed);
    }

    fn idle_ms(&self) -> u64 {
        (self.start.elapsed().as_millis() as u64)
            .saturating_sub(self.last_frame_ms.load(Ordering::Relaxed))
    }
}

/// What the HTTP thread serves.
struct Site {
    html: String,
    favicon: Option<Vec<u8>>,
    frame_clock: FrameClock,
}

fn run_http(shutdown: Arc<AtomicBool>, listener: TcpListener, site: Arc<Site>) {
    let addr = listener.local_addr().expect("wgui: HTTP listener has no local addr");

    listener
        .set_nonblocking(true)
        .expect("failed to set HTTP listener non-blocking");

    log::info!("wgui: serving UI at http://{addr}");

    while !shutdown.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _addr)) => {
                // Each request gets its own short-lived thread, so a socket
                // that connects but never speaks (browser preconnect) cannot
                // stall the accept loop.
                let site = site.clone();
                let spawned = thread::Builder::new()
                    .name("wgui-http-request".into())
                    .spawn(move || {
                        if let Err(e) = serve_request(stream, &site) {
                            log::debug!("wgui: HTTP request failed: {e}");
                        }
                    });
                if let Err(e) = spawned {
                    log::error!("wgui: failed to spawn HTTP request thread: {e}");
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(15));
            }
            Err(e) => {
                log::error!("wgui HTTP accept error: {e}");
                thread::sleep(Duration::from_millis(15));
            }
        }
    }
}

fn serve_request(mut stream: TcpStream, site: &Site) -> std::io::Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_nodelay(true).ok();
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let mut head = Vec::new();
    let mut chunk = [0u8; 1024];
    while !head.windows(4).any(|w| w == b"\r\n\r\n") {
        let read = stream.read(&mut chunk)?;
        if read == 0 || head.len() > 16 * 1024 {
            return Ok(());
        }
        head.extend_from_slice(&chunk[..read]);
    }
    let head = String::from_utf8_lossy(&head);
    let target = head.split_whitespace().nth(1).unwrap_or("");
    let path = target.split('?').next().unwrap_or("");

    let ping;
    let (status, content_type, body): (&str, &str, &[u8]) = match path {
        // Liveness probe. The frontend polls it while disconnected and only
        // opens a WebSocket once it answers, so connection attempts never
        // hit a dead server and never feed the browser's per-endpoint
        // reconnect backoff (RFC 6455 §7.2.3). The body carries the time
        // since the last frame for `holder_stopped_running_frames`.
        "/ping" => {
            ping = format!("ok {}", site.frame_clock.idle_ms());
            ("200 OK", "text/plain", ping.as_bytes())
        }
        "/" | "/index.html" => ("200 OK", "text/html; charset=utf-8", site.html.as_bytes()),
        "/favicon.png" if site.favicon.is_some() => {
            ("200 OK", "image/png", site.favicon.as_deref().unwrap_or_default())
        }
        _ => ("404 Not Found", "text/plain", b"404 Not Found"),
    };

    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    stream.write_all(&response)?;
    stream.shutdown(Shutdown::Write)
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

    // Handshakes run on short-lived threads and deliver finished sockets
    // here, so a socket that connects but never speaks (browser preconnect,
    // an aborted tab) can only stall its own thread — the pump loop keeps
    // accepting and serving clients throughout.
    let (pending_tx, pending_rx) = mpsc::channel::<tungstenite::WebSocket<TcpStream>>();

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
                    stream.set_nodelay(true).ok();
                    // Bounds the handshake: a silent socket errors out after
                    // 5s and its thread exits. The 1ms poll timeout is applied
                    // once the client is adopted by the pump loop.
                    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                    let tx = pending_tx.clone();
                    let spawned = thread::Builder::new()
                        .name("wgui-ws-handshake".into())
                        .spawn(move || match tungstenite::accept(stream) {
                            Ok(ws) => {
                                let _ = tx.send(ws);
                            }
                            Err(e) => log::error!("wgui: WS handshake failed: {e}"),
                        });
                    if let Err(e) = spawned {
                        log::error!("wgui: failed to spawn WS handshake thread: {e}");
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    log::error!("wgui WS accept error: {e}");
                    break;
                }
            }
        }

        // Adopt clients whose handshake completed
        while let Ok(mut ws) = pending_rx.try_recv() {
            log::info!("wgui: new WebSocket client connected (total: {})", clients.len() + 1);
            // Bound writes so a slow or half-dead client (e.g. a tab being
            // refreshed) can never block this thread. On timeout the send
            // errors and the client is dropped below.
            ws.get_ref().set_write_timeout(Some(Duration::from_secs(2))).ok();
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

/// How long a new instance waits for the start ports while their holder exits.
const PREDECESSOR_EXIT_WAIT: Duration = Duration::from_secs(3);

/// A holder whose last frame is older than this is treated as exiting.
const FRAME_STALL_MS: u64 = 1000;

fn bind_port_pair(base: u16, bind_addr: &str) -> Option<(TcpListener, TcpListener)> {
    let http = TcpListener::bind(format!("{bind_addr}:{base}")).ok()?;
    let ws = TcpListener::bind(format!("{bind_addr}:{}", base.checked_add(1)?)).ok()?;
    Some((http, ws))
}

/// Whether `port` is served by a wgui instance that no longer runs frames:
/// a crashed or closing process that has not let go of its ports yet.
fn holder_stopped_running_frames(port: u16) -> bool {
    let timeout = Duration::from_millis(300);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, timeout) else {
        return false;
    };
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();
    if stream.write_all(b"GET /ping HTTP/1.1\r\nHost: localhost\r\n\r\n").is_err() {
        return false;
    }
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);
    response
        .split("\r\n\r\n")
        .nth(1)
        .and_then(|body| body.strip_prefix("ok "))
        .and_then(|idle_ms| idle_ms.trim().parse::<u64>().ok())
        .is_some_and(|idle_ms| idle_ms > FRAME_STALL_MS)
}

/// Find an available port pair (http, ws) starting from `start`.
/// HTTP gets the even port, WS gets the odd port.
/// Returns the two bound listeners so there is no race condition between
/// scanning and serving.
/// Returns `None` if no free pair is found after scanning 1000 ports.
pub fn find_port_pair(start: u16, bind_addr: &str) -> Option<(TcpListener, TcpListener)> {
    if let Some(pair) = bind_port_pair(start, bind_addr) {
        return Some(pair);
    }
    // A previous run that is still shutting down frees the start ports
    // shortly; an instance that is still running frames keeps them.
    if holder_stopped_running_frames(start) {
        log::info!("wgui: waiting for the previous instance to release port {start}");
        let deadline = Instant::now() + PREDECESSOR_EXIT_WAIT;
        while Instant::now() < deadline {
            thread::sleep(Duration::from_millis(50));
            if let Some(pair) = bind_port_pair(start, bind_addr) {
                return Some(pair);
            }
        }
    }
    for base in (start..start.saturating_add(1000)).step_by(2).skip(1) {
        if let Some(pair) = bind_port_pair(base, bind_addr) {
            return Some(pair);
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
            tab: None,
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
