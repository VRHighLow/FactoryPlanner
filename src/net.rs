//! Online multiplayer via public MQTT broker (UK↔USA, code only).
//! Both peers dial out — no port forwarding / LAN required.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rumqttc::{Client, Event, MqttOptions, Packet, QoS};

use crate::sim::{BuildingKind, Facing};

const DEFAULT_MQTT_HOST: &str = "broker.emqx.io";
const DEFAULT_MQTT_PORT: u16 = 1883;

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

fn topic(code: &str) -> String {
    format!("factoryplanner/v1/{code}")
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

fn parse_peer(raw: &str, local_id: u8, ev: &Sender<NetEvent>) {
    let p: Vec<&str> = raw.trim().split('|').collect();
    match p.first().copied() {
        Some("CUR") if p.len() >= 6 => {
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
            });
        }
        Some("PLACE") if p.len() >= 6 => {
            // Apply remote places only (local already applied).
            // Host id 0 and clients may both place — sync all PLACE messages;
            // local sender still receives own echo — skip if we want by checking...
            // We can't know if we sent it easily; duplicate place_node_with_id is OK (same id).
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
        Some("HELLO") if p.len() >= 3 => {
            let id = p[2].parse().unwrap_or(255);
            if id != local_id {
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
    opts.set_keep_alive(Duration::from_secs(20));
    opts.set_clean_session(true);

    let (client, mut connection) = Client::new(opts, 64);
    let t = topic(&code);
    if let Err(e) = client.subscribe(&t, QoS::AtMostOnce) {
        let _ = ev_tx.send(NetEvent::JoinFailed {
            reason: format!("subscribe failed: {e}"),
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
        let _ = ev_tx.send(NetEvent::Info("Joined online session.".into()));
    }

    let hello = encode(&[
        "HELLO",
        if is_host { "HOST" } else { "CLIENT" },
        &local_id.to_string(),
    ]);
    let _ = client.publish(&t, QoS::AtMostOnce, false, hello.as_bytes());

    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let topic_pub = t.clone();
    thread::spawn(move || {
        while let Ok(cmd) = cmd_rx.recv() {
            let msg = match cmd {
                NetCommand::Stop => {
                    let bye = encode(&["BYE", &local_id.to_string()]);
                    let _ = client.publish(&topic_pub, QoS::AtMostOnce, false, bye.as_bytes());
                    stop2.store(true, Ordering::SeqCst);
                    break;
                }
                NetCommand::SetCursor {
                    x,
                    y,
                    selected,
                    facing,
                } => encode(&[
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
                } => encode(&[
                    "PLACE",
                    &id.to_string(),
                    &kind.as_u8().to_string(),
                    &format!("{x:.3}"),
                    &format!("{y:.3}"),
                    &facing.as_u8().to_string(),
                ]),
                NetCommand::Remove { id } => encode(&["REM", &id.to_string()]),
            };
            if client
                .publish(&topic_pub, QoS::AtMostOnce, false, msg.as_bytes())
                .is_err()
            {
                stop2.store(true, Ordering::SeqCst);
                break;
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
                    parse_peer(text, local_id, &ev_tx);
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

