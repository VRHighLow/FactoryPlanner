//! Online multiplayer via WebRTC (peer-to-peer after public signaling).
//!
//! Signaling uses a public Matchbox server; game traffic (cursors + world)
//! flows directly between peers — not through a lossy public MQTT broker.
//! That removes the jitter that caused choppy cursors and place desync.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use futures_util::FutureExt;
use matchbox_socket::{
    ChannelConfig, PeerId, PeerState, WebRtcSocket, WebRtcSocketBuilder,
};

use crate::sim::{BuildingKind, Facing};

const DEFAULT_SIGNALING: &str = "wss://match-0-9.helsing.studio";

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

fn signaling_base() -> String {
    std::env::var("FACTORY_SIGNALING").unwrap_or_else(|_| DEFAULT_SIGNALING.into())
}

fn gen_code() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:06}", (t % 1_000_000) as u32)
}

fn room_url(code: &str) -> String {
    let base = signaling_base().trim_end_matches('/').to_string();
    format!("{base}/fp{code}")
}

fn peer_u8(peer: PeerId) -> u8 {
    let mut h = DefaultHasher::new();
    peer.hash(&mut h);
    (1 + (h.finish() % 200)) as u8
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

fn encode_cmd(local_id: u8, is_host: bool, cmd: &NetCommand) -> Option<(bool, String)> {
    Some(match cmd {
        NetCommand::Stop | NetCommand::Announce => return None,
        NetCommand::WantSnap => (false, encode(&["WANT", &local_id.to_string()])),
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
            let tag = if *power { "P" } else { "B" };
            if *request && !is_host {
                (
                    false,
                    encode(&[
                        "LREQ",
                        &local_id.to_string(),
                        tag,
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
                        tag,
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

fn broadcast(socket: &mut WebRtcSocket<matchbox_socket::MultipleChannels>, channel: usize, msg: &str) {
    let packet: Box<[u8]> = msg.as_bytes().to_vec().into_boxed_slice();
    let peers: Vec<PeerId> = socket.connected_peers().collect();
    for peer in peers {
        let _ = socket.channel_mut(channel).try_send(packet.clone(), peer);
    }
}

fn run_webrtc(is_host: bool, code: String, local_id: u8, ev_tx: Sender<NetEvent>, cmd_rx: Receiver<NetCommand>) {
    let url = room_url(&code);
    let signaling = signaling_base();

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            let _ = ev_tx.send(NetEvent::JoinFailed {
                reason: format!("runtime error: {e}"),
            });
            return;
        }
    };

    rt.block_on(async move {
        let (mut socket, fut) = WebRtcSocketBuilder::new(url.clone())
            .add_channel(ChannelConfig::reliable())
            .add_channel(ChannelConfig::unreliable())
            .build();

        let mut loop_fut = tokio::spawn(fut).fuse();

        if is_host {
            let _ = ev_tx.send(NetEvent::HostReady {
                code: code.clone(),
                addr: format!("p2p via {signaling}"),
            });
            let _ = ev_tx.send(NetEvent::Joined { player_id: 0 });
            let _ = ev_tx.send(NetEvent::Info(
                "Session online — share your code. Waiting for peer…".into(),
            ));
        } else {
            let _ = ev_tx.send(NetEvent::Joined {
                player_id: local_id,
            });
            let _ = ev_tx.send(NetEvent::Info(
                "Connecting peer-to-peer… (may take a few seconds)".into(),
            ));
        }

        let hello = encode(&[
            "HELLO",
            if is_host { "HOST" } else { "CLIENT" },
            &local_id.to_string(),
        ]);

        let mut announced = false;
        let mut ticks: u32 = 0;

        loop {
            futures_util::select! {
                res = loop_fut => {
                    let reason = match res {
                        Ok(Ok(())) => "signaling closed".into(),
                        Ok(Err(e)) => format!("signaling error: {e}"),
                        Err(e) => format!("net task join: {e}"),
                    };
                    let _ = ev_tx.send(NetEvent::JoinFailed { reason });
                    break;
                }
                default => {}
            }

            for (peer, state) in socket.update_peers() {
                match state {
                    PeerState::Connected => {
                        let _ = ev_tx.send(NetEvent::Info(format!(
                            "Peer online ({})",
                            peer_u8(peer)
                        )));
                        let _ = ev_tx.send(NetEvent::PeerHello { id: peer_u8(peer) });
                        // Introduce ourselves on the reliable channel.
                        let packet: Box<[u8]> = hello.as_bytes().to_vec().into_boxed_slice();
                        let _ = socket.channel_mut(0).try_send(packet, peer);
                        if !is_host {
                            let want = encode(&["WANT", &local_id.to_string()]);
                            let packet: Box<[u8]> = want.as_bytes().to_vec().into_boxed_slice();
                            let _ = socket.channel_mut(0).try_send(packet, peer);
                        }
                        announced = true;
                    }
                    PeerState::Disconnected => {
                        let _ = ev_tx.send(NetEvent::PeerGone { id: peer_u8(peer) });
                    }
                }
            }

            // Reliable world channel
            for (_peer, packet) in socket.channel_mut(0).receive() {
                if let Ok(text) = std::str::from_utf8(&packet) {
                    parse_peer(text, local_id, is_host, &ev_tx);
                }
            }
            // Unreliable cursor channel
            for (_peer, packet) in socket.channel_mut(1).receive() {
                if let Ok(text) = std::str::from_utf8(&packet) {
                    parse_peer(text, local_id, is_host, &ev_tx);
                }
            }

            // Outbound commands
            let mut batch = Vec::new();
            match cmd_rx.try_recv() {
                Ok(c) => {
                    batch.push(c);
                    loop {
                        match cmd_rx.try_recv() {
                            Ok(c) => batch.push(c),
                            Err(TryRecvError::Empty) => break,
                            Err(TryRecvError::Disconnected) => return,
                        }
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => return,
            }

            let mut latest_cursor: Option<NetCommand> = None;
            let mut announce = false;
            for cmd in batch {
                match cmd {
                    NetCommand::Stop => {
                        let bye = encode(&["BYE", &local_id.to_string()]);
                        broadcast(&mut socket, 0, &bye);
                        return;
                    }
                    NetCommand::Announce => announce = true,
                    NetCommand::SetCursor { .. } => latest_cursor = Some(cmd),
                    other => {
                        if let Some((is_cur, msg)) = encode_cmd(local_id, is_host, &other) {
                            broadcast(&mut socket, if is_cur { 1 } else { 0 }, &msg);
                        }
                    }
                }
            }

            if announce || (!announced && ticks == 30) {
                broadcast(&mut socket, 0, &hello);
                if !is_host {
                    let want = encode(&["WANT", &local_id.to_string()]);
                    broadcast(&mut socket, 0, &want);
                }
                announced = true;
            }

            if let Some(cmd) = latest_cursor {
                if let Some((_, msg)) = encode_cmd(local_id, is_host, &cmd) {
                    broadcast(&mut socket, 1, &msg);
                }
            }

            ticks = ticks.saturating_add(1);
            tokio::time::sleep(Duration::from_millis(4)).await;
        }
    });
}

pub fn start_host() -> NetHandle {
    let code = gen_code();
    let (ev_tx, ev_rx) = mpsc::channel();
    let (cmd_tx, cmd_rx) = mpsc::channel::<NetCommand>();
    let code_ret = code.clone();
    let addr = signaling_base();

    thread::spawn(move || {
        run_webrtc(true, code, 0, ev_tx, cmd_rx);
    });

    NetHandle {
        rx: ev_rx,
        tx: cmd_tx,
        is_host: true,
        code: code_ret,
        join_addr: addr,
    }
}

pub fn start_client(_ignored: &str, code: &str) -> NetHandle {
    let code = code.trim().to_uppercase();
    let (ev_tx, ev_rx) = mpsc::channel();
    let (cmd_tx, cmd_rx) = mpsc::channel::<NetCommand>();
    let code_ret = code.clone();
    let addr = signaling_base();

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
        run_webrtc(false, code, local_id, ev_tx, cmd_rx);
    });

    NetHandle {
        rx: ev_rx,
        tx: cmd_tx,
        is_host: false,
        code: code_ret,
        join_addr: addr,
    }
}
