//! Public TCP relay: both host and clients dial out (works through home NAT).
//!
//! Protocol (first line):
//!   HOST|<6-digit-code>
//!   JOIN|<6-digit-code>
//! Reply:
//!   OK|HOST
//!   OK|CLIENT|<id>
//!   FAIL|<reason>
//! After that, every line is broadcast to everyone else in the room.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

type Writer = Arc<Mutex<TcpStream>>;

struct Member {
    id: u32,
    writer: Writer,
}

struct Room {
    members: Vec<Member>,
}

type Rooms = Arc<Mutex<HashMap<String, Room>>>;

fn main() {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(7788);
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).expect("bind relay");
    println!("factory_relay listening on {addr}");

    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));
    let next_id = Arc::new(AtomicU32::new(1));

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let _ = stream.set_nodelay(true);
        let rooms = rooms.clone();
        let next_id = next_id.clone();
        thread::spawn(move || handle_client(stream, rooms, next_id));
    }
}

fn handle_client(stream: TcpStream, rooms: Rooms, next_id: Arc<AtomicU32>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let writer: Writer = Arc::new(Mutex::new(stream));

    let mut hello = String::new();
    if reader.read_line(&mut hello).ok().filter(|n| *n > 0).is_none() {
        return;
    }
    let parts: Vec<&str> = hello.trim().split('|').collect();
    if parts.len() < 2 {
        let _ = write_line(&writer, "FAIL|bad_hello");
        return;
    }
    let role = parts[0];
    let code = parts[1].trim();
    if code.len() < 4 || code.len() > 12 || !code.chars().all(|c| c.is_ascii_alphanumeric()) {
        let _ = write_line(&writer, "FAIL|bad_code");
        return;
    }
    let code = code.to_uppercase();

    let my_id = next_id.fetch_add(1, Ordering::SeqCst);

    match role {
        "HOST" => {
            {
                let mut map = rooms.lock().unwrap();
                if let Some(room) = map.get(&code) {
                    if !room.members.is_empty() {
                        let _ = write_line(&writer, "FAIL|code_taken");
                        return;
                    }
                }
                map.insert(
                    code.clone(),
                    Room {
                        members: vec![Member {
                            id: my_id,
                            writer: writer.clone(),
                        }],
                    },
                );
            }
            if write_line(&writer, "OK|HOST").is_err() {
                leave(&rooms, &code, my_id);
                return;
            }
            println!("host room {code} id={my_id}");
        }
        "JOIN" => {
            {
                let mut map = rooms.lock().unwrap();
                let Some(room) = map.get_mut(&code) else {
                    let _ = write_line(&writer, "FAIL|no_room");
                    return;
                };
                room.members.push(Member {
                    id: my_id,
                    writer: writer.clone(),
                });
            }
            if write_line(&writer, &format!("OK|CLIENT|{my_id}")).is_err() {
                leave(&rooms, &code, my_id);
                return;
            }
            // Tell room someone joined (optional info)
            broadcast(&rooms, &code, my_id, &format!("INFO|join|{my_id}"));
            println!("join room {code} id={my_id}");
        }
        _ => {
            let _ = write_line(&writer, "FAIL|bad_role");
            return;
        }
    }

    let mut buf = String::new();
    while reader.read_line(&mut buf).ok().filter(|n| *n > 0).is_some() {
        let msg = buf.trim_end_matches(['\r', '\n']).to_string();
        buf.clear();
        if msg.is_empty() {
            continue;
        }
        broadcast(&rooms, &code, my_id, &msg);
    }
    leave(&rooms, &code, my_id);
    broadcast(&rooms, &code, my_id, &format!("INFO|leave|{my_id}"));
    println!("leave room {code} id={my_id}");
}

fn write_line(writer: &Writer, s: &str) -> std::io::Result<()> {
    let mut w = writer.lock().unwrap();
    writeln!(w, "{s}")?;
    w.flush()
}

fn broadcast(rooms: &Rooms, code: &str, from_id: u32, msg: &str) {
    let writers: Vec<Writer> = {
        let map = rooms.lock().unwrap();
        let Some(room) = map.get(code) else {
            return;
        };
        room.members
            .iter()
            .filter(|m| m.id != from_id)
            .map(|m| m.writer.clone())
            .collect()
    };
    for w in writers {
        let _ = write_line(&w, msg);
    }
}

fn leave(rooms: &Rooms, code: &str, id: u32) {
    let mut map = rooms.lock().unwrap();
    if let Some(room) = map.get_mut(code) {
        room.members.retain(|m| m.id != id);
        if room.members.is_empty() {
            map.remove(code);
        }
    }
}
