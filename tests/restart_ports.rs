use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use w_gui::{Context, ContextOptions};

fn options(start_port: u16, public: bool) -> ContextOptions {
    ContextOptions {
        start_port,
        public,
        ..Default::default()
    }
}

fn restart_in_process(start_port: u16, public: bool, with_clients: bool) {
    let first = Context::with_options(options(start_port, public));
    assert_eq!(first.http_port(), start_port);
    let clients = with_clients.then(|| {
        let (ws, _) = tungstenite::connect(format!("ws://127.0.0.1:{}", first.ws_port())).unwrap();
        let http = TcpStream::connect(("127.0.0.1", first.http_port())).unwrap();
        (ws, http)
    });
    drop(first);
    let second = Context::with_options(options(start_port, public));
    assert_eq!(second.http_port(), start_port);
    drop(clients);
}

#[test]
fn restart_keeps_ports_localhost() {
    restart_in_process(19080, false, false);
}

#[test]
fn restart_keeps_ports_public() {
    restart_in_process(19180, true, false);
}

#[test]
fn restart_keeps_ports_localhost_with_clients() {
    restart_in_process(19280, false, true);
}

#[test]
fn restart_keeps_ports_public_with_clients() {
    restart_in_process(19380, true, true);
}

#[test]
fn waits_for_ports_of_instance_that_stopped_running_frames() {
    let start_port = 19480;
    let mut stalled = Context::with_options(options(start_port, true));
    stalled.end_frame();
    thread::sleep(Duration::from_millis(1200));
    let exit = thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        drop(stalled);
    });
    let successor = Context::with_options(options(start_port, true));
    assert_eq!(successor.http_port(), start_port);
    exit.join().unwrap();
}

#[test]
fn moves_past_ports_of_instance_that_runs_frames() {
    let start_port = 19580;
    let stop = Arc::new(AtomicBool::new(false));
    let mut running = Context::with_options(options(start_port, true));
    let frames = {
        let stop = stop.clone();
        thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                running.end_frame();
                thread::sleep(Duration::from_millis(16));
            }
        })
    };
    thread::sleep(Duration::from_millis(100));
    let second = Context::with_options(options(start_port, true));
    assert_eq!(second.http_port(), start_port + 2);
    stop.store(true, Ordering::Relaxed);
    frames.join().unwrap();
}
