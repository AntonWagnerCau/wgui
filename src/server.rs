use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;


use crate::protocol::{ClientMsg, ServerMsg};
use crate::state::Shared;

const HTML: &str = include_str!("frontend.html");

/// Spawn the HTTP server thread. Returns the join handle.
pub fn spawn_http(shared: Shared, port: u16) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("wgui-http".into())
        .spawn(move || run_http(shared, port))
        .expect("failed to spawn wgui HTTP thread")
}

/// Spawn the WebSocket server thread. Returns the join handle.
pub fn spawn_ws(shared: Shared, port: u16) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("wgui-ws".into())
        .spawn(move || run_ws(shared, port))
        .expect("failed to spawn wgui WS thread")
}

fn run_http(shared: Shared, port: u16) {
    let server = tiny_http::Server::http(format!("127.0.0.1:{port}"))
        .unwrap_or_else(|e| panic!("wgui: failed to bind HTTP on port {port}: {e}"));

    log::info!("wgui: serving UI at http://127.0.0.1:{port}");

    loop {
        // Check shutdown
        if shared.lock().unwrap().shutdown {
            break;
        }

        // Use a timeout so we can check shutdown periodically
        match server.recv_timeout(Duration::from_millis(200)) {
            Ok(Some(request)) => {
                let response = match request.url() {
                    "/" | "/index.html" => tiny_http::Response::from_string(HTML)
                        .with_header(
                            "Content-Type: text/html; charset=utf-8"
                                .parse::<tiny_http::Header>()
                                .unwrap(),
                        ),
                    _ => tiny_http::Response::from_string("404 Not Found")
                        .with_status_code(404),
                };
                let _ = request.respond(response);
            }
            Ok(None) => {} // timeout, loop again
            Err(e) => {
                log::error!("wgui HTTP error: {e}");
                break;
            }
        }
    }
}

fn run_ws(shared: Shared, port: u16) {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
        .unwrap_or_else(|e| panic!("wgui: failed to bind WS on port {port}: {e}"));

    listener
        .set_nonblocking(true)
        .expect("failed to set WS listener non-blocking");

    log::info!("wgui: WebSocket listening on ws://127.0.0.1:{port}");

    loop {
        if shared.lock().unwrap().shutdown {
            break;
        }

        match listener.accept() {
            Ok((stream, _addr)) => {
                log::info!("wgui: new WebSocket client connected");
                handle_ws_client(&shared, stream);
                log::info!("wgui: WebSocket client disconnected");
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                log::error!("wgui WS accept error: {e}");
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

fn handle_ws_client(shared: &Shared, stream: TcpStream) {
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

    // Signal that we need a snapshot
    {
        let mut state = shared.lock().unwrap();
        state.needs_snapshot = true;
    }

    loop {
        // Check shutdown
        if shared.lock().unwrap().shutdown {
            let _ = ws.close(None);
            break;
        }

        // Send outgoing messages
        {
            let mut state = shared.lock().unwrap();

            if state.needs_snapshot {
                let snapshot = ServerMsg::Snapshot {
                    elements: state.current_elements.clone(),
                };
                if let Ok(json) = serde_json::to_string(&snapshot) {
                    if ws
                        .send(tungstenite::Message::Text(json.into()))
                        .is_err()
                    {
                        return;
                    }
                }
                state.needs_snapshot = false;
            }

            for msg in state.outgoing_msgs.drain(..) {
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

        // Read incoming messages (non-blocking with timeout)
        match ws.read() {
            Ok(tungstenite::Message::Text(text)) => {
                if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&text) {
                    match client_msg {
                        ClientMsg::Set { id, value } => {
                            let mut state = shared.lock().unwrap();
                            state.incoming_edits.insert(id, value);
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

/// Find an available port pair (http, ws) starting from `start`.
/// HTTP gets the even port, WS gets the odd port.
pub fn find_port_pair(start: u16) -> (u16, u16) {
    for base in (start..start + 100).step_by(2) {
        let http_ok = TcpListener::bind(format!("127.0.0.1:{base}")).is_ok();
        let ws_ok = TcpListener::bind(format!("127.0.0.1:{}", base + 1)).is_ok();
        if http_ok && ws_ok {
            return (base, base + 1);
        }
    }
    panic!("wgui: could not find available port pair starting from {start}");
}
