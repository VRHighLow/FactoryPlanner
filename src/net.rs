//! Online multiplayer via iroh-gossip (P2P + n0 public relays).
//!
//! Short 6-digit codes stay for UX: MQTT only publishes/fetches an iroh ticket.
//! All gameplay (cursors, place/remove, snapshots) rides iroh-gossip — not MQTT.

use std::fmt;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use bytes::Bytes;
use iroh::address_lookup::memory::MemoryLookup;
use iroh::endpoint::presets;
use iroh::{Endpoint, EndpointAddr};
use iroh_gossip::api::Event;
use iroh_gossip::net::{Gossip, GOSSIP_ALPN};
use iroh_gossip::proto::TopicId;
use n0_future::StreamExt;
use rumqttc::{Client, Event as MqttEvent, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};

use crate::sim::{BuildingKind, Facing};

const DEFAULT_MQTT_HOST: &str = "broker.emqx.io";
const DEFAULT_MQTT_PORT: u16 = 1883;

#[derive(Clone, Debug)]
pub enum NetEvent {
    HostReady { code: String, addr: String },
    Joined { player_id: u8 },
    JoinFailed { reason: String },
    PeerHello { id: u8 },
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
    Place {
        id: u32,
        kind: BuildingKind,
        x: f32,
        y: f32,
        facing: Facing,
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

#[derive(Debug, Serialize, Deserialize)]
struct Ticket {
    topic: TopicId,
    peers: Vec<EndpointAddr>,
}

impl Ticket {
    fn to_bytes(&self) -> Vec<u8> {
        postcard::to_stdvec(self).expect("ticket encode")
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        postcard::from_bytes(bytes).map_err(|e| format!("bad ticket: {e}"))
    }
}

impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = data_encoding::BASE32_NOPAD.encode(&self.to_bytes());
        text.make_ascii_lowercase();
        write!(f, "{text}")
    }
}

impl FromStr for Ticket {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = data_encoding::BASE32_NOPAD
            .decode(s.trim().to_ascii_uppercase().as_bytes())
            .map_err(|e| format!("bad ticket encoding: {e}"))?;
        Self::from_bytes(&bytes)
    }
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

fn topic_ticket(code: &str) -> String {
    format!("factoryplanner/v5/{code}/t")
}

fn looks_like_ticket(s: &str) -> bool {
    let s = s.trim();
    s.len() >= 40
        && s.chars()
            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '2'..='7'))
}

fn mqtt_publish_ticket(code: &str, ticket: &str) -> Result<(), String> {
    let (host, port) = mqtt_endpoint();
    let client_id = format!(
        "fpht{}{}",
        code,
        Instant::now().elapsed().as_nanos() % 100_000
    );
    let mut opts = MqttOptions::new(client_id, host, port);
    opts.set_keep_alive(Duration::from_secs(15));
    opts.set_clean_session(true);
    let (client, mut connection) = Client::new(opts, 64);
    let topic = topic_ticket(code);
    client
        .publish(&topic, QoS::AtLeastOnce, true, ticket.as_bytes())
        .map_err(|e| format!("mqtt publish: {e}"))?;
    let deadline = Instant::now() + Duration::from_secs(10);
    for notification in connection.iter() {
        if Instant::now() > deadline {
            break;
        }
        match notification {
            Ok(MqttEvent::Incoming(Packet::PubAck(_))) => return Ok(()),
            Ok(MqttEvent::Outgoing(_)) => {}
            Ok(MqttEvent::Incoming(Packet::ConnAck(_))) => {}
            Err(e) => return Err(format!("mqtt: {e}")),
            _ => {}
        }
    }
    // Best-effort: broker may have accepted even if we missed PubAck.
    Ok(())
}

fn mqtt_clear_ticket(code: &str) {
    let (host, port) = mqtt_endpoint();
    let client_id = format!("fpclr{}{}", code, Instant::now().elapsed().as_nanos() % 99_999);
    let mut opts = MqttOptions::new(client_id, host, port);
    opts.set_keep_alive(Duration::from_secs(10));
    let (client, mut connection) = Client::new(opts, 16);
    let topic = topic_ticket(code);
    let _ = client.publish(&topic, QoS::AtLeastOnce, true, b"");
    let deadline = Instant::now() + Duration::from_secs(3);
    for notification in connection.iter() {
        if Instant::now() > deadline {
            break;
        }
        if matches!(
            notification,
            Ok(MqttEvent::Incoming(Packet::PubAck(_))) | Err(_)
        ) {
            break;
        }
    }
}

fn mqtt_fetch_ticket(code: &str) -> Result<String, String> {
    let (host, port) = mqtt_endpoint();
    let client_id = format!(
        "fpjt{}{}",
        code,
        Instant::now().elapsed().as_nanos() % 100_000
    );
    let mut opts = MqttOptions::new(client_id, host, port);
    opts.set_keep_alive(Duration::from_secs(15));
    opts.set_clean_session(true);
    let (client, mut connection) = Client::new(opts, 64);
    let topic = topic_ticket(code);
    client
        .subscribe(&topic, QoS::AtLeastOnce)
        .map_err(|e| format!("mqtt subscribe: {e}"))?;
    let deadline = Instant::now() + Duration::from_secs(15);
    for notification in connection.iter() {
        if Instant::now() > deadline {
            return Err("no host found for that code (timed out)".into());
        }
        match notification {
            Ok(MqttEvent::Incoming(Packet::Publish(p))) => {
                if p.topic == topic {
                    let s = String::from_utf8_lossy(&p.payload).trim().to_string();
                    if !s.is_empty() {
                        return Ok(s);
                    }
                }
            }
            Err(e) => return Err(format!("mqtt: {e}")),
            _ => {}
        }
    }
    Err("no host found for that code".into())
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

fn encode_cmd(local_id: u8, is_host: bool, cmd: &NetCommand) -> Option<String> {
    Some(match cmd {
        NetCommand::Stop | NetCommand::Announce => return None,
        NetCommand::WantSnap => encode(&["WANT", &local_id.to_string()]),
        NetCommand::SnapBegin => encode(&["SNAP0", &local_id.to_string()]),
        NetCommand::SnapEnd => encode(&["SNAP1", &local_id.to_string()]),
        NetCommand::SetCursor {
            x,
            y,
            selected,
            facing,
            t_ms,
        } => encode(&[
            "CUR",
            &local_id.to_string(),
            &format!("{x:.2}"),
            &format!("{y:.2}"),
            &kind_opt(*selected),
            &facing.as_u8().to_string(),
            &format!("{t_ms:.2}"),
        ]),
        NetCommand::Place {
            id,
            kind,
            x,
            y,
            facing,
            request,
        } => {
            if *request && !is_host {
                encode(&[
                    "PREQ",
                    &local_id.to_string(),
                    &kind.as_u8().to_string(),
                    &format!("{x:.3}"),
                    &format!("{y:.3}"),
                    &facing.as_u8().to_string(),
                ])
            } else {
                encode(&[
                    "PLACE",
                    &local_id.to_string(),
                    &id.to_string(),
                    &kind.as_u8().to_string(),
                    &format!("{x:.3}"),
                    &format!("{y:.3}"),
                    &facing.as_u8().to_string(),
                ])
            }
        }
        NetCommand::Remove { id, request } => {
            if *request && !is_host {
                encode(&["RREQ", &local_id.to_string(), &id.to_string()])
            } else {
                encode(&["REM", &local_id.to_string(), &id.to_string()])
            }
        }
        NetCommand::Move {
            id,
            x,
            y,
            request,
        } => {
            if *request && !is_host {
                encode(&[
                    "MREQ",
                    &local_id.to_string(),
                    &id.to_string(),
                    &format!("{x:.3}"),
                    &format!("{y:.3}"),
                ])
            } else {
                encode(&[
                    "MOVE",
                    &local_id.to_string(),
                    &id.to_string(),
                    &format!("{x:.3}"),
                    &format!("{y:.3}"),
                ])
            }
        }
        NetCommand::Rotate {
            id,
            facing,
            request,
        } => {
            if *request && !is_host {
                encode(&[
                    "TREQ",
                    &local_id.to_string(),
                    &id.to_string(),
                    &facing.as_u8().to_string(),
                ])
            } else {
                encode(&[
                    "ROT",
                    &local_id.to_string(),
                    &id.to_string(),
                    &facing.as_u8().to_string(),
                ])
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
                encode(&[
                    "LREQ",
                    &local_id.to_string(),
                    if *power { "P" } else { "B" },
                    &from_node.to_string(),
                    &from_port.to_string(),
                    &to_node.to_string(),
                    &to_port.to_string(),
                ])
            } else {
                encode(&[
                    "LINK",
                    &local_id.to_string(),
                    if *power { "P" } else { "B" },
                    &from_node.to_string(),
                    &from_port.to_string(),
                    &to_node.to_string(),
                    &to_port.to_string(),
                ])
            }
        }
    })
}

async fn run_session(
    is_host: bool,
    code: String,
    local_id: u8,
    ticket: Ticket,
    bootstrap: Vec<EndpointAddr>,
    ev_tx: Sender<NetEvent>,
    cmd_rx: Receiver<NetCommand>,
) {
    let memory_lookup = MemoryLookup::new();
    let endpoint = match Endpoint::builder(presets::N0)
        .address_lookup(memory_lookup.clone())
        .bind()
        .await
    {
        Ok(ep) => ep,
        Err(e) => {
            let _ = ev_tx.send(NetEvent::JoinFailed {
                reason: format!("iroh bind failed: {e}"),
            });
            return;
        }
    };

    let _ = ev_tx.send(NetEvent::Info("Connecting to iroh relay…".into()));
    endpoint.online().await;

    let gossip = Gossip::builder().spawn(endpoint.clone());
    let router = iroh::protocol::Router::builder(endpoint.clone())
        .accept(GOSSIP_ALPN, gossip.clone())
        .spawn();

    // Host publishes a ticket that includes the live endpoint address after online().
    let live_ticket = if is_host {
        let t = Ticket {
            topic: ticket.topic,
            peers: vec![endpoint.addr()],
        };
        let text = t.to_string();
        if let Err(e) = mqtt_publish_ticket(&code, &text) {
            let _ = ev_tx.send(NetEvent::JoinFailed {
                reason: format!("code publish failed: {e}"),
            });
            let _ = router.shutdown().await;
            return;
        }
        // Refresh retained ticket periodically (addr can gain more paths).
        let code_bg = code.clone();
        let ep_bg = endpoint.clone();
        let topic_bg = ticket.topic;
        let stop_refresh = Arc::new(AtomicBool::new(false));
        let stop_refresh2 = stop_refresh.clone();
        thread::spawn(move || {
            while !stop_refresh2.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_secs(20));
                if stop_refresh2.load(Ordering::SeqCst) {
                    break;
                }
                let t = Ticket {
                    topic: topic_bg,
                    peers: vec![ep_bg.addr()],
                };
                let _ = mqtt_publish_ticket(&code_bg, &t.to_string());
            }
        });
        Some(stop_refresh)
    } else {
        None
    };

    let peer_ids: Vec<_> = bootstrap.iter().map(|p| p.id).collect();
    for peer in bootstrap {
        memory_lookup.add_endpoint_info(peer);
    }

    let (sender, mut receiver) = match gossip.subscribe_and_join(ticket.topic, peer_ids).await {
        Ok(t) => t.split(),
        Err(e) => {
            let _ = ev_tx.send(NetEvent::JoinFailed {
                reason: format!("gossip join failed: {e}"),
            });
            let _ = router.shutdown().await;
            return;
        }
    };

    if is_host {
        let _ = ev_tx.send(NetEvent::HostReady {
            code: code.clone(),
            addr: "iroh P2P".into(),
        });
        let _ = ev_tx.send(NetEvent::Joined { player_id: 0 });
        let _ = ev_tx.send(NetEvent::Info(
            "Session ready — share your code (P2P via iroh).".into(),
        ));
    } else {
        let _ = ev_tx.send(NetEvent::Joined {
            player_id: local_id,
        });
        let _ = ev_tx.send(NetEvent::Info("Joined via iroh — syncing…".into()));
    }

    let hello = encode(&[
        "HELLO",
        if is_host { "HOST" } else { "CLIENT" },
        &local_id.to_string(),
    ]);
    let _ = sender.broadcast(Bytes::from(hello)).await;
    if !is_host {
        let want = encode(&["WANT", &local_id.to_string()]);
        let _ = sender.broadcast(Bytes::from(want)).await;
    }

    let (async_tx, mut async_rx) = tokio::sync::mpsc::channel::<NetCommand>(512);
    thread::spawn(move || {
        while let Ok(cmd) = cmd_rx.recv() {
            if async_tx.blocking_send(cmd).is_err() {
                break;
            }
        }
    });

    let mut last_cursor_send = Instant::now() - Duration::from_secs(1);
    let mut stopping = false;

    loop {
        tokio::select! {
            cmd = async_rx.recv() => {
                let Some(first) = cmd else { break; };
                let mut batch = vec![first];
                while let Ok(c) = async_rx.try_recv() {
                    batch.push(c);
                }

                let mut latest_cursor: Option<NetCommand> = None;
                let mut world_ops: Vec<NetCommand> = Vec::new();
                let mut announce = false;
                for cmd in batch {
                    match cmd {
                        NetCommand::Stop => {
                            let bye = encode(&["BYE", &local_id.to_string()]);
                            let _ = sender.broadcast(Bytes::from(bye)).await;
                            stopping = true;
                            break;
                        }
                        NetCommand::Announce => announce = true,
                        NetCommand::SetCursor { .. } => latest_cursor = Some(cmd),
                        other => world_ops.push(other),
                    }
                }
                if stopping {
                    break;
                }

                if announce {
                    let hello = encode(&[
                        "HELLO",
                        if is_host { "HOST" } else { "CLIENT" },
                        &local_id.to_string(),
                    ]);
                    let _ = sender.broadcast(Bytes::from(hello)).await;
                    if !is_host {
                        let want = encode(&["WANT", &local_id.to_string()]);
                        let _ = sender.broadcast(Bytes::from(want)).await;
                    }
                }

                for cmd in world_ops {
                    if let Some(msg) = encode_cmd(local_id, is_host, &cmd) {
                        if sender.broadcast(Bytes::from(msg)).await.is_err() {
                            let _ = ev_tx.send(NetEvent::Info(
                                "Warning: gossip send failed".into(),
                            ));
                        }
                    }
                }

                if let Some(cmd) = latest_cursor {
                    if last_cursor_send.elapsed() >= Duration::from_millis(20) {
                        if let Some(msg) = encode_cmd(local_id, is_host, &cmd) {
                            let _ = sender.broadcast(Bytes::from(msg)).await;
                            last_cursor_send = Instant::now();
                        }
                    }
                }
            }
            ev = receiver.next() => {
                match ev {
                    Some(Ok(Event::Received(msg))) => {
                        if let Ok(text) = std::str::from_utf8(&msg.content) {
                            parse_peer(text, local_id, is_host, &ev_tx);
                        }
                    }
                    Some(Ok(Event::NeighborUp(_))) => {
                        let _ = ev_tx.send(NetEvent::Info("Peer link up".into()));
                        if is_host {
                            let _ = ev_tx.send(NetEvent::WantSnap);
                        }
                    }
                    Some(Ok(Event::NeighborDown(_))) => {
                        let _ = ev_tx.send(NetEvent::Info("Peer link down".into()));
                    }
                    Some(Ok(Event::Lagged)) => {
                        let _ = ev_tx.send(NetEvent::Info("Gossip lagged — requesting snap".into()));
                        if !is_host {
                            let want = encode(&["WANT", &local_id.to_string()]);
                            let _ = sender.broadcast(Bytes::from(want)).await;
                        }
                    }
                    Some(Err(e)) => {
                        let _ = ev_tx.send(NetEvent::JoinFailed {
                            reason: format!("gossip error: {e}"),
                        });
                        break;
                    }
                    None => break,
                }
            }
        }
    }

    if let Some(stop) = live_ticket {
        stop.store(true, Ordering::SeqCst);
        mqtt_clear_ticket(&code);
    }
    let _ = router.shutdown().await;
}

fn spawn_runtime(
    is_host: bool,
    code: String,
    local_id: u8,
    ticket: Ticket,
    bootstrap: Vec<EndpointAddr>,
    ev_tx: Sender<NetEvent>,
    cmd_rx: Receiver<NetCommand>,
) {
    thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                let _ = ev_tx.send(NetEvent::JoinFailed {
                    reason: format!("runtime: {e}"),
                });
                return;
            }
        };
        rt.block_on(run_session(
            is_host, code, local_id, ticket, bootstrap, ev_tx, cmd_rx,
        ));
    });
}

pub fn start_host() -> NetHandle {
    let code = gen_code();
    let (ev_tx, ev_rx) = mpsc::channel();
    let (cmd_tx, cmd_rx) = mpsc::channel::<NetCommand>();
    let code_ret = code.clone();
    let topic = TopicId::from_bytes(rand::random());
    // Placeholder ticket; live addr published after endpoint.online().
    let ticket = Ticket {
        topic,
        peers: vec![],
    };
    spawn_runtime(true, code, 0, ticket, vec![], ev_tx, cmd_rx);

    NetHandle {
        rx: ev_rx,
        tx: cmd_tx,
        is_host: true,
        code: code_ret,
        join_addr: "iroh P2P".into(),
    }
}

pub fn start_client(_ignored: &str, code_or_ticket: &str) -> NetHandle {
    let raw = code_or_ticket.trim().to_string();
    let (ev_tx, ev_rx) = mpsc::channel();
    let (cmd_tx, cmd_rx) = mpsc::channel::<NetCommand>();

    let local_id: u8 = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(1);
        (1 + (t % 200)) as u8
    };

    let code_display = if looks_like_ticket(&raw) {
        "TICKET".into()
    } else {
        raw.to_uppercase()
    };

    thread::spawn(move || {
        let ticket = if looks_like_ticket(&raw) {
            match Ticket::from_str(&raw) {
                Ok(t) => t,
                Err(e) => {
                    let _ = ev_tx.send(NetEvent::JoinFailed { reason: e });
                    return;
                }
            }
        } else {
            let code = raw.to_uppercase();
            if code.len() < 4 {
                let _ = ev_tx.send(NetEvent::JoinFailed {
                    reason: "enter the host's 6-digit code".into(),
                });
                return;
            }
            let _ = ev_tx.send(NetEvent::Info("Looking up session code…".into()));
            let text = match mqtt_fetch_ticket(&code) {
                Ok(t) => t,
                Err(e) => {
                    let _ = ev_tx.send(NetEvent::JoinFailed { reason: e });
                    return;
                }
            };
            match Ticket::from_str(&text) {
                Ok(t) => t,
                Err(e) => {
                    let _ = ev_tx.send(NetEvent::JoinFailed { reason: e });
                    return;
                }
            }
        };

        let bootstrap = ticket.peers.clone();
        let code = if looks_like_ticket(&raw) {
            "TICKET".into()
        } else {
            raw.to_uppercase()
        };
        spawn_runtime(false, code, local_id, ticket, bootstrap, ev_tx, cmd_rx);
    });

    NetHandle {
        rx: ev_rx,
        tx: cmd_tx,
        is_host: false,
        code: code_display,
        join_addr: "iroh P2P".into(),
    }
}
