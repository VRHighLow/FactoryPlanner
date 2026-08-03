//! Online multiplayer via public MQTT broker (UK↔USA, code only).
//!
//! Design (fixes choppy cursors + place desync on lossy public brokers):
//! - Host is authoritative for the world (place/remove/move/rotate/links)
//! - Clients send requests; host applies + broadcasts
//! - Host sends a full SNAP periodically so peers always converge
//! - Cursors use an unreliable channel; receivers extrapolate with velocity

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use rumqttc::{Client, Event, MqttOptions, Packet, QoS};

use crate::sim::{BuildingKind, Facing};

const DEFAULT_MQTT_HOST: &str = "broker.emqx.io";
const DEFAULT_MQTT_PORT: u16 = 1883;

#[derive(Clone, Debug)]
pub enum NetEvent {
    HostReady { code: String, addr: String },
    Joined { player_id: u8 },
    JoinFailed { reason: String },
    PeerHello { id: u8 },
    /// Client→host: please place this building (host assigns id).
    PlaceRequest {
        kind: BuildingKind,
        x: f32,
        y: f32,
        facing: Facing,
    },
    RemoveRequest { id: u32 },
    MoveRequest { id: u32, x: f32, y: f32 },
    RotateRequest { id: u32, facing: Facing },
    LinkRequest {
        power: bool,
        from_node: u32,
        from_port: usize,
        to_node: u32,
        to_port: usize,
    },
    /// Ask host for a snapshot (clients).
    WantSnap,
    PeerCursor {
        id: u8,
        x: f32,
        y: f32,
        selected: Option<BuildingKind>,
        facing: Facing,
        t_ms: f32,
    },
    PeerPlace {
        id: u32,
        kind: BuildingKind,
        x: f32,
        y: f32,
        facing: Facing,
    },
    PeerRemove { id: u32 },
    PeerMove { id: u32, x: f32, y: f32 },
    PeerRotate { id: u32, facing: Facing },
    PeerLink {
        power: bool,
        from_node: u32,
        from_port: usize,
        to_node: u32,
        to_port: usize,
    },
    /// Full world wipe then rebuild from following PeerPlace/PeerLink until SnapEnd.
    SnapBegin,
    SnapEnd,
    PeerGone { id: u8 },
    Info(String),
}

#[derive(Clone, Debug)]
pub enum NetCommand {
    Stop,
    Announce,
    WantSnap,
    SetCursor {
        x: f32,
        y: f32,
        selected: Option<BuildingKind>,
        facing: Facing,
        t_ms: f32,
    },
    /// Host→all or client→host request (see `as_request`).
    Place {
        id: u32,
        kind: BuildingKind,
        x: f32,
        y: f32,
        facing: Facing,
        /// If true, this is a client request (no id yet / ignore id).
        request: bool,
    },
    Remove { id: u32, request: bool },
    Move {
        id: u32,
        x: f32,
        y: f32,
        request: bool,
    },
    Rotate {
        id: u32,
        facing: Facing,
        request: bool,
    },
    Link {
        power: bool,
        from_node: u32,
        from_port: usize,
        to_node: u32,
        to_port: usize,
        request: bool,
    },
    SnapBegin,
    SnapEnd,
}

pub struct NetHandle {
    pub rx: Receiver<NetEvent>,
    pub tx: Sender<NetCommand>,
    pub is_host: bool,
    pub code: String,
    pub join_addr: String,
}

fn mqtt_endpoint() -> (String, u16) {
    let host = std::env::var("FACTORY_MQTT_HOST").unwrap_or_else(|_| DEFAULT_MQTT_HOST.into());
    let port = std::env::var("FACTORY_MQTT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_MQTT_PORT);
    (host, port)
}

fn gen_code() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:06}", (t % 1_000_000) as u32)
}

fn topic_world(code: &str) -> String {
    format!("factoryplanner/v4/{code}/w")
}

fn topic_cursor(code: &str) -> String {
    format!("factoryplanner/v4/{code}/c")
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

fn encode(parts: &[&str]) -> String {
    parts.join("|")
}

fn parse_peer(raw: &str, local_id: u8, is_host: bool, ev: &Sender<NetEvent>) {
    let p: Vec<&str> = raw.trim().split('|').collect();
    match p.first().copied() {
        Some("CUR") if p.len() >= 7 => {
            let id = p[1].parse().unwrap_or(255);
            if id == local_id {
                return;
            }
            let _ = ev.send(NetEvent::PeerCursor {
                id,
                x: p[2].parse().unwrap_or(0.0),
                y: p[3].parse().unwrap_or(0.0),
                selected: parse_kind(p[4]),
                facing: Facing::from_u8(p[5].parse().unwrap_or(0)),
                t_ms: p[6].parse().unwrap_or(0.0),
            });
        }
        // Client place request — host only
        Some("PREQ") if p.len() >= 6 && is_host => {
            let owner: u8 = p[1].parse().unwrap_or(255);
            if owner == local_id {
                return;
            }
            if let Some(kind) = BuildingKind::from_u8(p[2].parse().unwrap_or(255)) {
                let _ = ev.send(NetEvent::PlaceRequest {
                    kind,
                    x: p[3].parse().unwrap_or(0.0),
                    y: p[4].parse().unwrap_or(0.0),
                    facing: Facing::from_u8(p[5].parse().unwrap_or(0)),
                });
            }
        }
        Some("RREQ") if p.len() >= 3 && is_host => {
            let owner: u8 = p[1].parse().unwrap_or(255);
            if owner == local_id {
                return;
            }
            if let Ok(id) = p[2].parse() {
                let _ = ev.send(NetEvent::RemoveRequest { id });
            }
        }
        Some("MREQ") if p.len() >= 5 && is_host => {
            let owner: u8 = p[1].parse().unwrap_or(255);
            if owner == local_id {
                return;
            }
            let _ = ev.send(NetEvent::MoveRequest {
                id: p[2].parse().unwrap_or(0),
                x: p[3].parse().unwrap_or(0.0),
                y: p[4].parse().unwrap_or(0.0),
            });
        }
        Some("TREQ") if p.len() >= 4 && is_host => {
            let owner: u8 = p[1].parse().unwrap_or(255);
            if owner == local_id {
                return;
            }
            let _ = ev.send(NetEvent::RotateRequest {
                id: p[2].parse().unwrap_or(0),
                facing: Facing::from_u8(p[3].parse().unwrap_or(0)),
            });
        }
        Some("LREQ") if p.len() >= 7 && is_host => {
            let owner: u8 = p[1].parse().unwrap_or(255);
            if owner == local_id {
                return;
            }
            let _ = ev.send(NetEvent::LinkRequest {
                power: p[2] == "P",
                from_node: p[3].parse().unwrap_or(0),
                from_port: p[4].parse().unwrap_or(0),
                to_node: p[5].parse().unwrap_or(0),
                to_port: p[6].parse().unwrap_or(0),
            });
        }
        Some("WANT") if p.len() >= 2 && is_host => {
            let owner: u8 = p[1].parse().unwrap_or(255);
            if owner != local_id {
                let _ = ev.send(NetEvent::WantSnap);
            }
        }
        // Authoritative world ops from host (everyone applies; host skips own echo)
        Some("PLACE") if p.len() >= 7 => {
            let owner: u8 = p[1].parse().unwrap_or(255);
            if owner == local_id {
                return;
            }
            if let Some(kind) = BuildingKind::from_u8(p[3].parse().unwrap_or(255)) {
                let _ = ev.send(NetEvent::PeerPlace {
                    id: p[2].parse().unwrap_or(0),
                    kind,
                    x: p[4].parse().unwrap_or(0.0),
                    y: p[5].parse().unwrap_or(0.0),
                    facing: Facing::from_u8(p[6].parse().unwrap_or(0)),
                });
            }
        }
        Some("REM") if p.len() >= 3 => {
            let owner: u8 = p[1].parse().unwrap_or(255);
            if owner == local_id {
                return;
            }
            if let Ok(id) = p[2].parse() {
                let _ = ev.send(NetEvent::PeerRemove { id });
            }
        }
        Some("MOVE") if p.len() >= 5 => {
            let owner: u8 = p[1].parse().unwrap_or(255);
            if owner == local_id {
                return;
            }
            let _ = ev.send(NetEvent::PeerMove {
                id: p[2].parse().unwrap_or(0),
                x: p[3].parse().unwrap_or(0.0),
                y: p[4].parse().unwrap_or(0.0),
            });
        }
        Some("ROT") if p.len() >= 4 => {
            let owner: u8 = p[1].parse().unwrap_or(255);
            if owner == local_id {
                return;
            }
            let _ = ev.send(NetEvent::PeerRotate {
                id: p[2].parse().unwrap_or(0),
                facing: Facing::from_u8(p[3].parse().unwrap_or(0)),
            });
        }
        Some("LINK") if p.len() >= 7 => {
            let owner: u8 = p[1].parse().unwrap_or(255);
            if owner == local_id {
                return;
            }
            let _ = ev.send(NetEvent::PeerLink {
                power: p[2] == "P",
                from_node: p[3].parse().unwrap_or(0),
                from_port: p[4].parse().unwrap_or(0),
                to_node: p[5].parse().unwrap_or(0),
                to_port: p[6].parse().unwrap_or(0),
            });
        }
        Some("SNAP0") => {
            let owner: u8 = p.get(1).and_then(|s| s.parse().ok()).unwrap_or(255);
            if owner != local_id {
                let _ = ev.send(NetEvent::SnapBegin);
            }
        }
        Some("SNAP1") => {
            let owner: u8 = p.get(1).and_then(|s| s.parse().ok()).unwrap_or(255);
            if owner != local_id {
                let _ = ev.send(NetEvent::SnapEnd);
            }
        }
        Some("HELLO") if p.len() >= 3 => {
            let id = p[2].parse().unwrap_or(255);
            if id != local_id {
                let _ = ev.send(NetEvent::PeerHello { id });
                let _ = ev.send(NetEvent::Info(format!("Player {id} connected")));
            }
        }
        Some("BYE") if p.len() >= 2 => {
            if let Ok(id) = p[1].parse::<u8>() {
                if id != local_id {
                    let _ = ev.send(NetEvent::PeerGone { id });
                }
            }
        }
        _ => {}
    }
}

fn encode_cmd(local_id: u8, is_host: bool, cmd: &NetCommand) -> Option<(bool, String)> {
    Some(match cmd {
        NetCommand::Stop | NetCommand::Announce => return None,
        NetCommand::WantSnap => (
            false,
            encode(&["WANT", &local_id.to_string()]),
        ),
        NetCommand::SnapBegin => (false, encode(&["SNAP0", &local_id.to_string()])),
        NetCommand::SnapEnd => (false, encode(&["SNAP1", &local_id.to_string()])),
        NetCommand::SetCursor {
            x,
            y,
            selected,
            facing,
            t_ms,
        } => (
            true,
            encode(&[
                "CUR",
                &local_id.to_string(),
                &format!("{x:.2}"),
                &format!("{y:.2}"),
                &kind_opt(*selected),
                &facing.as_u8().to_string(),
                &format!("{t_ms:.2}"),
            ]),
        ),
        NetCommand::Place {
            id,
            kind,
            x,
            y,
            facing,
            request,
        } => {
            if *request && !is_host {
                (
                    false,
                    encode(&[
                        "PREQ",
                        &local_id.to_string(),
                        &kind.as_u8().to_string(),
                        &format!("{x:.3}"),
                        &format!("{y:.3}"),
                        &facing.as_u8().to_string(),
                    ]),
                )
            } else {
                (
                    false,
                    encode(&[
                        "PLACE",
                        &local_id.to_string(),
                        &id.to_string(),
                        &kind.as_u8().to_string(),
                        &format!("{x:.3}"),
                        &format!("{y:.3}"),
                        &facing.as_u8().to_string(),
                    ]),
                )
            }
        }
        NetCommand::Remove { id, request } => {
            if *request && !is_host {
                (
                    false,
                    encode(&["RREQ", &local_id.to_string(), &id.to_string()]),
                )
            } else {
                (
                    false,
                    encode(&["REM", &local_id.to_string(), &id.to_string()]),
                )
            }
        }
        NetCommand::Move {
            id,
            x,
            y,
            request,
        } => {
            if *request && !is_host {
                (
                    false,
                    encode(&[
                        "MREQ",
                        &local_id.to_string(),
                        &id.to_string(),
                        &format!("{x:.3}"),
                        &format!("{y:.3}"),
                    ]),
                )
            } else {
                (
                    false,
                    encode(&[
                        "MOVE",
                        &local_id.to_string(),
                        &id.to_string(),
                        &format!("{x:.3}"),
                        &format!("{y:.3}"),
                    ]),
                )
            }
        }
        NetCommand::Rotate {
            id,
            facing,
            request,
        } => {
            if *request && !is_host {
                (
                    false,
                    encode(&[
                        "TREQ",
                        &local_id.to_string(),
                        &id.to_string(),
                        &facing.as_u8().to_string(),
                    ]),
                )
            } else {
                (
                    false,
                    encode(&[
                        "ROT",
                        &local_id.to_string(),
                        &id.to_string(),
                        &facing.as_u8().to_string(),
                    ]),
                )
            }
        }
        NetCommand::Link {
            power,
            from_node,
            from_port,
            to_node,
            to_port,
            request,
        } => {
            if *request && !is_host {
                (
                    false,
                    encode(&[
                        "LREQ",
                        &local_id.to_string(),
                        if *power { "P" } else { "B" },
                        &from_node.to_string(),
                        &from_port.to_string(),
                        &to_node.to_string(),
                        &to_port.to_string(),
                    ]),
                )
            } else {
                (
                    false,
                    encode(&[
                        "LINK",
                        &local_id.to_string(),
                        if *power { "P" } else { "B" },
                        &from_node.to_string(),
                        &from_port.to_string(),
                        &to_node.to_string(),
                        &to_port.to_string(),
                    ]),
                )
            }
        }
    })
}

fn publish_retry(client: &Client, topic: &str, qos: QoS, msg: &str, tries: u32) -> bool {
    for _ in 0..tries {
        if client.publish(topic, qos, false, msg.as_bytes()).is_ok() {
            return true;
        }
        thread::sleep(Duration::from_millis(3));
    }
    false
}

fn run_mqtt(
    is_host: bool,
    code: String,
    local_id: u8,
    ev_tx: Sender<NetEvent>,
    cmd_rx: Receiver<NetCommand>,
) {
    let (host, port) = mqtt_endpoint();
    let client_id = format!(
        "fp{}{}{}",
        code,
        local_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() % 100_000)
            .unwrap_or(0)
    );
    let mut opts = MqttOptions::new(client_id, &host, port);
    opts.set_keep_alive(Duration::from_secs(15));
    opts.set_clean_session(true);

    let (client, mut connection) = Client::new(opts, 8192);
    let tw = topic_world(&code);
    let tc = topic_cursor(&code);
    if client.subscribe(&tw, QoS::AtLeastOnce).is_err()
        || client.subscribe(&tc, QoS::AtMostOnce).is_err()
    {
        let _ = ev_tx.send(NetEvent::JoinFailed {
            reason: "subscribe failed".into(),
        });
        return;
    }

    if is_host {
        let _ = ev_tx.send(NetEvent::HostReady {
            code: code.clone(),
            addr: format!("online via {host}"),
        });
        let _ = ev_tx.send(NetEvent::Joined { player_id: 0 });
        let _ = ev_tx.send(NetEvent::Info(
            "Session online — share your code worldwide.".into(),
        ));
    } else {
        let _ = ev_tx.send(NetEvent::Joined {
            player_id: local_id,
        });
        let _ = ev_tx.send(NetEvent::Info("Joined — waiting for host sync…".into()));
    }

    let hello = encode(&[
        "HELLO",
        if is_host { "HOST" } else { "CLIENT" },
        &local_id.to_string(),
    ]);
    let _ = publish_retry(&client, &tw, QoS::AtLeastOnce, &hello, 5);

    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let tw_pub = tw.clone();
    let tc_pub = tc.clone();
    let ev_pub = ev_tx.clone();
    thread::spawn(move || {
        let mut last_cursor_send = Instant::now() - Duration::from_secs(1);
        loop {
            let first = match cmd_rx.recv() {
                Ok(c) => c,
                Err(_) => break,
            };
            let mut batch = vec![first];
            loop {
                match cmd_rx.try_recv() {
                    Ok(c) => batch.push(c),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        stop2.store(true, Ordering::SeqCst);
                        return;
                    }
                }
            }

            let mut latest_cursor: Option<NetCommand> = None;
            let mut world_ops: Vec<NetCommand> = Vec::new();
            let mut announce = false;
            for cmd in batch {
                match cmd {
                    NetCommand::Stop => {
                        let bye = encode(&["BYE", &local_id.to_string()]);
                        let _ = publish_retry(&client, &tw_pub, QoS::AtLeastOnce, &bye, 3);
                        stop2.store(true, Ordering::SeqCst);
                        return;
                    }
                    NetCommand::Announce => announce = true,
                    NetCommand::SetCursor { .. } => latest_cursor = Some(cmd),
                    other => world_ops.push(other),
                }
            }

            if announce {
                let hello = encode(&[
                    "HELLO",
                    if is_host { "HOST" } else { "CLIENT" },
                    &local_id.to_string(),
                ]);
                let _ = publish_retry(&client, &tw_pub, QoS::AtLeastOnce, &hello, 5);
                if !is_host {
                    let want = encode(&["WANT", &local_id.to_string()]);
                    let _ = publish_retry(&client, &tw_pub, QoS::AtLeastOnce, &want, 5);
                }
            }

            for cmd in world_ops {
                if let Some((_, msg)) = encode_cmd(local_id, is_host, &cmd) {
                    if !publish_retry(&client, &tw_pub, QoS::AtLeastOnce, &msg, 8) {
                        let _ = ev_pub.send(NetEvent::Info(
                            "Warning: world sync publish failed".into(),
                        ));
                    }
                    // Small gap so broker/event-loop can breathe between snap lines.
                    thread::sleep(Duration::from_millis(1));
                }
            }

            // Cap cursor publishes (~30 Hz) — receiver extrapolates between samples.
            if let Some(cmd) = latest_cursor {
                if last_cursor_send.elapsed() >= Duration::from_millis(33) {
                    if let Some((_, msg)) = encode_cmd(local_id, is_host, &cmd) {
                        let _ = client.publish(&tc_pub, QoS::AtMostOnce, false, msg.as_bytes());
                        last_cursor_send = Instant::now();
                    }
                }
            }
        }
    });

    for notification in connection.iter() {
        if stop.load(Ordering::SeqCst) {
            break;
        }
        match notification {
            Ok(Event::Incoming(Packet::Publish(p))) => {
                if let Ok(text) = std::str::from_utf8(&p.payload) {
                    parse_peer(text, local_id, is_host, &ev_tx);
                }
            }
            Err(e) => {
                let _ = ev_tx.send(NetEvent::JoinFailed {
                    reason: format!("connection lost: {e}"),
                });
                break;
            }
            _ => {}
        }
    }
}

pub fn start_host() -> NetHandle {
    let code = gen_code();
    let (ev_tx, ev_rx) = mpsc::channel();
    let (cmd_tx, cmd_rx) = mpsc::channel::<NetCommand>();
    let code_ret = code.clone();
    let (host, _) = mqtt_endpoint();

    thread::spawn(move || {
        run_mqtt(true, code, 0, ev_tx, cmd_rx);
    });

    NetHandle {
        rx: ev_rx,
        tx: cmd_tx,
        is_host: true,
        code: code_ret,
        join_addr: host,
    }
}

pub fn start_client(_ignored: &str, code: &str) -> NetHandle {
    let code = code.trim().to_uppercase();
    let (ev_tx, ev_rx) = mpsc::channel();
    let (cmd_tx, cmd_rx) = mpsc::channel::<NetCommand>();
    let code_ret = code.clone();
    let (host, _) = mqtt_endpoint();

    let local_id: u8 = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(1);
        (1 + (t % 200)) as u8
    };

    thread::spawn(move || {
        if code.len() < 4 {
            let _ = ev_tx.send(NetEvent::JoinFailed {
                reason: "enter the host's 6-digit code".into(),
            });
            return;
        }
        run_mqtt(false, code, local_id, ev_tx, cmd_rx);
    });

    NetHandle {
        rx: ev_rx,
        tx: cmd_tx,
        is_host: false,
        code: code_ret,
        join_addr: host,
    }
}
