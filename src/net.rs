//! TCP multiplayer: host code, join, cursor/ghost + place/remove sync.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::sim::{BuildingKind, Facing};

pub const DEFAULT_PORT: u16 = 7788;

#[derive(Clone, Debug)]
pub enum NetEvent {
    HostReady { code: String, addr: String },
    Joined { player_id: u8 },
    JoinFailed { reason: String },
    PeerCursor {
        id: u8,
        x: f32,
        y: f32,
        selected: Option<BuildingKind>,
        facing: Facing,
    },
    PeerPlace {
        id: u32,
        kind: BuildingKind,
        x: f32,
        y: f32,
        facing: Facing,
    },
    PeerRemove { id: u32 },
    PeerGone { id: u8 },
    Info(String),
}

#[derive(Clone, Debug)]
pub enum NetCommand {
    Stop,
    SetCursor {
        x: f32,
        y: f32,
        selected: Option<BuildingKind>,
        facing: Facing,
    },
    Place {
        id: u32,
        kind: BuildingKind,
        x: f32,
        y: f32,
        facing: Facing,
    },
    Remove {
        id: u32,
    },
}

pub struct NetHandle {
    pub rx: Receiver<NetEvent>,
    pub tx: Sender<NetCommand>,
    pub is_host: bool,
    pub code: String,
    pub join_addr: String,
}

fn gen_code() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("{:06}", t % 1_000_000)
}

fn local_ip_guess() -> String {
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".into()
}

fn line(parts: &[&str]) -> String {
    parts.join("|") + "\n"
}

fn kind_opt(k: Option<BuildingKind>) -> String {
    k.map(|k| k.as_u8().to_string())
        .unwrap_or_else(|| "-".into())
}

fn parse_kind(s: &str) -> Option<BuildingKind> {
    if s == "-" {
        None
    } else {
        s.parse::<u8>().ok().and_then(BuildingKind::from_u8)
    }
}

fn parse_incoming(pid_fallback: u8, raw: &str, ev: &Sender<NetEvent>) {
    let p: Vec<&str> = raw.trim().split('|').collect();
    match p.first().copied() {
        Some("CUR") if p.len() >= 6 => {
            let id = p[1].parse().unwrap_or(pid_fallback);
            let _ = ev.send(NetEvent::PeerCursor {
                id,
                x: p[2].parse().unwrap_or(0.0),
                y: p[3].parse().unwrap_or(0.0),
                selected: parse_kind(p[4]),
                facing: Facing::from_u8(p[5].parse().unwrap_or(0)),
            });
        }
        Some("PLACE") if p.len() >= 6 => {
            if let Some(kind) = BuildingKind::from_u8(p[2].parse().unwrap_or(255)) {
                let _ = ev.send(NetEvent::PeerPlace {
                    id: p[1].parse().unwrap_or(0),
                    kind,
                    x: p[3].parse().unwrap_or(0.0),
                    y: p[4].parse().unwrap_or(0.0),
                    facing: Facing::from_u8(p[5].parse().unwrap_or(0)),
                });
            }
        }
        Some("REM") if p.len() >= 2 => {
            if let Ok(id) = p[1].parse() {
                let _ = ev.send(NetEvent::PeerRemove { id });
            }
        }
        _ => {}
    }
}

type ClientList = Arc<Mutex<Vec<(u8, TcpStream)>>>;

fn broadcast(clients: &ClientList, msg: &str, except: Option<u8>) {
    let mut list = clients.lock().unwrap();
    list.retain_mut(|(id, s)| {
        if except == Some(*id) {
            return true;
        }
        s.write_all(msg.as_bytes()).is_ok()
    });
}

pub fn start_host() -> NetHandle {
    let code = gen_code();
    let ip = local_ip_guess();
    let addr = format!("{ip}:{DEFAULT_PORT}");
    let (ev_tx, ev_rx) = mpsc::channel();
    let (cmd_tx, cmd_rx) = mpsc::channel::<NetCommand>();
    let code_c = code.clone();
    let addr_c = addr.clone();
    let running = Arc::new(AtomicBool::new(true));
    let running2 = running.clone();

    thread::spawn(move || {
        let listener = match TcpListener::bind(("0.0.0.0", DEFAULT_PORT)) {
            Ok(l) => l,
            Err(e) => {
                let _ = ev_tx.send(NetEvent::JoinFailed {
                    reason: format!("bind: {e}"),
                });
                return;
            }
        };
        let _ = ev_tx.send(NetEvent::HostReady {
            code: code_c.clone(),
            addr: addr_c.clone(),
        });
        let _ = ev_tx.send(NetEvent::Joined { player_id: 0 });

        let clients: ClientList = Arc::new(Mutex::new(Vec::new()));
        let next_id = Arc::new(Mutex::new(1u8));

        // Accept loop
        let clients_a = clients.clone();
        let ev_a = ev_tx.clone();
        let code_a = code_c.clone();
        let next_a = next_id.clone();
        let running_a = running.clone();
        thread::spawn(move || {
            for stream in listener.incoming() {
                if !running_a.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(stream) = stream else { continue };
                let _ = stream.set_nodelay(true);
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut hello = String::new();
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                if reader.read_line(&mut hello).is_err() {
                    continue;
                }
                let parts: Vec<&str> = hello.trim().split('|').collect();
                let mut stream = reader.into_inner();
                if parts.first() == Some(&"JOIN") && parts.get(1) == Some(&code_a.as_str()) {
                    let id = {
                        let mut n = next_a.lock().unwrap();
                        let id = *n;
                        *n = n.saturating_add(1);
                        id
                    };
                    if stream
                        .write_all(line(&["OK", &id.to_string()]).as_bytes())
                        .is_ok()
                    {
                        let _ = ev_a.send(NetEvent::Info(format!("Player {id} joined")));
                        // reader thread
                        let ev_r = ev_a.clone();
                        let clients_r = clients_a.clone();
                        let stream_r = stream.try_clone().unwrap();
                        thread::spawn(move || {
                            let mut reader = BufReader::new(stream_r);
                            let mut buf = String::new();
                            while reader.read_line(&mut buf).ok().filter(|n| *n > 0).is_some() {
                                let raw = buf.clone();
                                parse_incoming(id, &raw, &ev_r);
                                broadcast(&clients_r, &raw, Some(id));
                                buf.clear();
                            }
                            let _ = ev_r.send(NetEvent::PeerGone { id });
                            clients_r.lock().unwrap().retain(|(i, _)| *i != id);
                        });
                        clients_a.lock().unwrap().push((id, stream));
                    }
                } else {
                    let _ = stream.write_all(line(&["FAIL", "bad_code"]).as_bytes());
                }
            }
        });

        // Command loop (host local → clients)
        while running2.load(Ordering::SeqCst) {
            match cmd_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(NetCommand::Stop) => {
                    running2.store(false, Ordering::SeqCst);
                    break;
                }
                Ok(NetCommand::SetCursor {
                    x,
                    y,
                    selected,
                    facing,
                }) => {
                    let msg = line(&[
                        "CUR",
                        "0",
                        &format!("{x:.2}"),
                        &format!("{y:.2}"),
                        &kind_opt(selected),
                        &facing.as_u8().to_string(),
                    ]);
                    broadcast(&clients, &msg, None);
                }
                Ok(NetCommand::Place {
                    id,
                    kind,
                    x,
                    y,
                    facing,
                }) => {
                    let msg = line(&[
                        "PLACE",
                        &id.to_string(),
                        &kind.as_u8().to_string(),
                        &format!("{x:.3}"),
                        &format!("{y:.3}"),
                        &facing.as_u8().to_string(),
                    ]);
                    broadcast(&clients, &msg, None);
                }
                Ok(NetCommand::Remove { id }) => {
                    broadcast(&clients, &line(&["REM", &id.to_string()]), None);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    NetHandle {
        rx: ev_rx,
        tx: cmd_tx,
        is_host: true,
        code,
        join_addr: addr,
    }
}

pub fn start_client(host_addr: &str, code: &str) -> NetHandle {
    let (ev_tx, ev_rx) = mpsc::channel();
    let (cmd_tx, cmd_rx) = mpsc::channel::<NetCommand>();
    let addr = host_addr.to_string();
    let code = code.to_string();
    let display = addr.clone();
    let code_ret = code.clone();

    thread::spawn(move || {
        let mut stream = match TcpStream::connect(&addr) {
            Ok(s) => s,
            Err(e) => {
                let _ = ev_tx.send(NetEvent::JoinFailed {
                    reason: format!("{e}"),
                });
                return;
            }
        };
        let _ = stream.set_nodelay(true);
        if stream
            .write_all(line(&["JOIN", &code]).as_bytes())
            .is_err()
        {
            let _ = ev_tx.send(NetEvent::JoinFailed {
                reason: "write failed".into(),
            });
            return;
        }
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut resp = String::new();
        let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
        if reader.read_line(&mut resp).is_err() {
            let _ = ev_tx.send(NetEvent::JoinFailed {
                reason: "no reply".into(),
            });
            return;
        }
        let parts: Vec<&str> = resp.trim().split('|').collect();
        if parts.first() != Some(&"OK") {
            let _ = ev_tx.send(NetEvent::JoinFailed {
                reason: "bad code".into(),
            });
            return;
        }
        let local_id: u8 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
        let _ = ev_tx.send(NetEvent::Joined {
            player_id: local_id,
        });

        let mut writer = stream.try_clone().unwrap();
        thread::spawn(move || {
            while let Ok(cmd) = cmd_rx.recv() {
                let msg = match cmd {
                    NetCommand::Stop => break,
                    NetCommand::SetCursor {
                        x,
                        y,
                        selected,
                        facing,
                    } => line(&[
                        "CUR",
                        &local_id.to_string(),
                        &format!("{x:.2}"),
                        &format!("{y:.2}"),
                        &kind_opt(selected),
                        &facing.as_u8().to_string(),
                    ]),
                    NetCommand::Place {
                        id,
                        kind,
                        x,
                        y,
                        facing,
                    } => line(&[
                        "PLACE",
                        &id.to_string(),
                        &kind.as_u8().to_string(),
                        &format!("{x:.3}"),
                        &format!("{y:.3}"),
                        &facing.as_u8().to_string(),
                    ]),
                    NetCommand::Remove { id } => line(&["REM", &id.to_string()]),
                };
                if writer.write_all(msg.as_bytes()).is_err() {
                    break;
                }
            }
        });

        let _ = stream.set_read_timeout(None);
        let mut reader = BufReader::new(stream);
        let mut buf = String::new();
        while reader.read_line(&mut buf).ok().filter(|n| *n > 0).is_some() {
            parse_incoming(0, &buf, &ev_tx);
            buf.clear();
        }
    });

    NetHandle {
        rx: ev_rx,
        tx: cmd_tx,
        is_host: false,
        code: code_ret,
        join_addr: display,
    }
}
