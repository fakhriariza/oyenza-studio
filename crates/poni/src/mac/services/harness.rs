//! DeepSeek Harness and AI Agent notification listener on macOS.
//!
//! Listens on http://127.0.0.1:48123 for notifications from AI agent harnesses
//! (like DeepSeek, SWE-bench, benchmark runners, or terminal scripts).
//!
//! Supports:
//!   POST /notify (JSON body: {"title": "...", "subtitle": "...", "status": "acc"|"done"|"info"})
//!   GET /notify?title=...&subtitle=...&status=acc|done|info

use crate::events::{AppEvent, EventSink};
use serde::Deserialize;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub const DEFAULT_PORT: u16 = 48123;

#[derive(Debug, Deserialize)]
pub struct HarnessPayload {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub status: Option<String>,
}

pub struct HarnessService {
    _handle: thread::JoinHandle<()>,
}

impl HarnessService {
    pub fn start(sink: EventSink) -> Option<Self> {
        let listener = match TcpListener::bind(("127.0.0.1", DEFAULT_PORT)) {
            Ok(l) => l,
            Err(e) => {
                pwarn!("harness: couldn't bind 127.0.0.1:{DEFAULT_PORT}: {e}");
                return None;
            }
        };

        pinfo!("harness: notification listener running on http://127.0.0.1:{DEFAULT_PORT}");

        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    let sink_clone = sink.clone();
                    thread::spawn(move || {
                        handle_connection(stream, sink_clone);
                    });
                }
            }
        });

        Some(Self { _handle: handle })
    }
}

fn handle_connection(mut stream: TcpStream, sink: EventSink) {
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req = String::from_utf8_lossy(&buf[..n]);
    let mut lines = req.lines();
    let first_line = lines.next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");

    let mut payload: Option<HarnessPayload> = None;

    if method == "POST" {
        if let Some(body_start) = req.find("\r\n\r\n") {
            let body = &req[body_start + 4..];
            payload = serde_json::from_str(body).ok();
        }
    } else if method == "GET" && path.starts_with("/notify") {
        if let Some(q_idx) = path.find('?') {
            let query = &path[q_idx + 1..];
            let mut title = None;
            let mut subtitle = None;
            let mut status = None;
            for param in query.split('&') {
                let mut kv = param.split('=');
                let k = kv.next().unwrap_or("");
                let v = kv.next().unwrap_or("");
                let decoded = v.replace('+', " ");
                match k {
                    "title" => title = Some(decoded),
                    "subtitle" | "msg" | "message" => subtitle = Some(decoded),
                    "status" | "type" => status = Some(decoded),
                    _ => {}
                }
            }
            payload = Some(HarnessPayload { title, subtitle, status });
        }
    }

    if let Some(p) = payload {
        let title = p.title.unwrap_or_else(|| "DeepSeek Harness".to_string());
        let status = p.status.unwrap_or_else(|| "info".to_string());
        let subtitle = p.subtitle.unwrap_or_else(|| {
            if status == "acc" {
                "Needs your approval (acc)".to_string()
            } else if status == "done" {
                "Task completed / AI udah beres".to_string()
            } else {
                "Status update".to_string()
            }
        });

        sink.send(AppEvent::HarnessNotice {
            title,
            subtitle,
            status,
        });

        let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"ok\":true}\r\n";
        let _ = stream.write_all(resp.as_bytes());
    } else {
        let resp = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nBad Request\r\n";
        let _ = stream.write_all(resp.as_bytes());
    }
}
