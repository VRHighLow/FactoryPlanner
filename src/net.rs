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
pub const DISCOVER_PORT: u16 = 7789;

#[derive(Clone, Debug)]
pub struct LanGame {
    pub name: String,
    pub code: String,
    pub addr: String, // host:port for TCP
    pub last_seen_ms: u128,
}

#[derive(Clone, Debug)]
pub enum NetEvent {
    HostReady { code: String, addr: String },
    Joined { player_id: u8 },
    JoinFailed { reason: String },
    LanGames(Vec<LanGame>),
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

fn now_ms() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn advertise_loop(running: Arc<AtomicBool>, code: String, tcp_port: u16, host_name: String) {
    thread::spawn(move || {
        let sock = match std::net::UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(_) => return,
        };
        let _ = sock.set_broadcast(true);
        let payload = format!("FP1|{code}|{tcp_port}|{host_name}");
        while running.load(Ordering::SeqCst) {
            let _ = sock.send_to(payload.as_bytes(), format!("255.255.255.255:{DISCOVER_PORT}"));
            // Also try subnet broadcast via connected iface guess
            let _ = sock.send_to(payload.as_bytes(), format!("10.0.0.255:{DISCOVER_PORT}"));
            let _ = sock.send_to(payload.as_bytes(), format!("192.168.0.255:{DISCOVER_PORT}"));
            let _ = sock.send_to(payload.as_bytes(), format!("192.168.1.255:{DISCOVER_PORT}"));
            thread::sleep(Duration::from_millis(800));
        }
    });
}

/// Background LAN browser (Ark-style server list). Call once on Join screen.
pub fn start_lan_browser() -> Receiver<NetEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let sock = match std::net::UdpSocket::bind(("0.0.0.0", DISCOVER_PORT)) {
            Ok(s) => s,
            Err(e) => {
                let _ = tx.send(NetEvent::JoinFailed {
                    reason: format!("LAN browse bind failed: {e}"),
                });
                return;
            }
        };
        let _ = sock.set_broadcast(true);
        let _ = sock.set_read_timeout(Some(Duration::from_millis(400)));
        let mut games: std::collections::HashMap<String, LanGame> = std::collections::HashMap::new();
        let mut buf = [0u8; 512];
        loop {
            match sock.recv_from(&mut buf) {
                Ok((n, from)) => {
                    let msg = String::from_utf8_lossy(&buf[..n]);
                    let p: Vec<&str> = msg.trim().split('|').collect();
                    // FP1|code|port|name
                    if p.len() >= 3 && p[0] == "FP1" {
                        let code = p[1].to_string();
                        let port: u16 = p[2].parse().unwrap_or(DEFAULT_PORT);
                        let name = p
                            .get(3)
                            .map(|s| s.to_string())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| "Factory".into());
                        let addr = format!("{}:{}", from.ip(), port);
                        games.insert(
                            code.clone(),
                            LanGame {
                                name,
                                code: code.clone(),
                                addr,
                                last_seen_ms: now_ms(),
                            },
                        );
                    }
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(_) => {}
            }
            let cutoff = now_ms().saturating_sub(3000);
            games.retain(|_, g| g.last_seen_ms >= cutoff);
            let mut list: Vec<LanGame> = games.values().cloned().collect();
            list.sort_by(|a, b| a.name.cmp(&b.name).then(a.code.cmp(&b.code)));
            if tx.send(NetEvent::LanGames(list)).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(200));
        }
    });
    rx
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
    let host_name = format!("Host-{code}");

    advertise_loop(running.clone(), code.clone(), DEFAULT_PORT, host_name);

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
        let _ = ev_tx.send(NetEvent::Info(
            "Broadcasting on LAN — friends can Join and click your game.".into(),
        ));

        let clients: ClientList = Arc::new(Mutex::new(Vec::new()));
        let next_id = Arc::new(Mutex::new(1u8));

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
                // JOIN|code  OR  JOIN|*  (LAN click — trust discovery, still check code if provided)
                let ok_code = parts.first() == Some(&"JOIN")
                    && (parts.get(1) == Some(&code_a.as_str()) || parts.get(1) == Some(&"*"));
                if ok_code {
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
        use std::net::ToSocketAddrs;
        let socket_addr = match addr.to_socket_addrs() {
            Ok(mut it) => it.next(),
            Err(e) => {
                let _ = ev_tx.send(NetEvent::JoinFailed {
                    reason: format!("bad address: {e}"),
                });
                return;
            }
        };
        let Some(socket_addr) = socket_addr else {
            let _ = ev_tx.send(NetEvent::JoinFailed {
                reason: "could not resolve address".into(),
            });
            return;
        };

        let mut stream = match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(4)) {
            Ok(s) => s,
            Err(_) => {
                let _ = ev_tx.send(NetEvent::JoinFailed {
                    reason: "Can't reach host (timeout). Same Wi‑Fi? Guest Wi‑Fi blocks this. Host must click Host Game first.".into(),
                });
                return;
            }
        };
        let _ = stream.set_nodelay(true);
        let join_code = if code.is_empty() { "*" } else { code.as_str() };
        if stream
            .write_all(line(&["JOIN", join_code]).as_bytes())
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
                reason: "host did not reply".into(),
            });
            return;
        }
        let parts: Vec<&str> = resp.trim().split('|').collect();
        if parts.first() != Some(&"OK") {
            let _ = ev_tx.send(NetEvent::JoinFailed {
                reason: "rejected by host (bad code)".into(),
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
