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
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub const DEFAULT_PORT: u16 = 48123;
/// How long `/ask` holds its request open: half an hour for someone to answer.
const ASK_WAIT: Duration = Duration::from_secs(1800);

/// The question whose answer the island owes, if one is up.
static PENDING: Mutex<Option<Sender<usize>>> = Mutex::new(None);

/// The answer someone picked in the island (0-based, the option's index).
pub fn answer(i: usize) {
    if let Ok(mut g) = PENDING.lock() {
        if let Some(tx) = g.take() {
            let _ = tx.send(i);
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HarnessPayload {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub status: Option<String>,
    /// `/ask` only: what the person may answer, in the order shown.
    #[serde(default)]
    pub options: Vec<String>,
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

/// `POST /ask`: show the question in the island and hold the request open until
/// someone answers. The reply is `{"answer": 0, "label": "..."}`, or `{"answer":
/// null}` once nobody answered in time.
fn answer_ask(mut stream: TcpStream, req: &str, sink: EventSink) {
    let body = req.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or("");
    let p = match serde_json::from_str::<HarnessPayload>(body) {
        Ok(p) if p.options.len() >= 2 => p,
        _ => {
            let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":\"options: at least two\"}\r\n");
            return;
        }
    };
    let title = p.title.unwrap_or_else(|| "DeepSeek Harness".to_string());
    let question = p.subtitle.unwrap_or_else(|| "Needs your answer".to_string());

    let (tx, rx) = channel();
    if let Ok(mut g) = PENDING.lock() {
        *g = Some(tx);
    }
    crate::state::set_ask(crate::state::Ask {
        title: title.clone(),
        question: question.clone(),
        options: p.options.clone(),
        until: crate::clock::mono() + ASK_WAIT.as_secs_f64(),
    });
    sink.send(AppEvent::HarnessNotice { title, subtitle: question.clone(), status: "ask".into() });

    pinfo!("harness: asked \"{question}\" with {} answers", p.options.len());
    let picked = rx.recv_timeout(ASK_WAIT).ok();
    if picked.is_none() {
        crate::state::take_ask();  // nobody answered: don't leave it on screen
    }
    pinfo!("harness: answered {picked:?}");
    let body = match picked.and_then(|i| p.options.get(i).map(|l| (i, l.clone()))) {
        Some((i, label)) => serde_json::json!({ "answer": i, "label": label }).to_string(),
        None => "{\"answer\":null}".to_string(),
    };
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(resp.as_bytes());
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

    if method == "GET" && path.starts_with("/answer") {
        // Answering over HTTP, for a script or a phone: the island's own pills
        // take this same path.
        let i = path.split_once("i=").and_then(|(_, v)| v.split('&').next()).and_then(|v| v.parse::<usize>().ok());
        if let Some(i) = i {
            answer(i);
        }
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"ok\":true}\r\n");
        return;
    }

    if method == "POST" && path.starts_with("/ask") {
        answer_ask(stream, &req, sink);
        return;
    }

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
            payload = Some(HarnessPayload { title, subtitle, status, options: Vec::new() });
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
