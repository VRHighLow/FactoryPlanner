mod sim;
mod net;

use macroquad::prelude::*;
use net::{NetCommand, NetEvent, NetHandle};
use sim::*;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

const MIN_ZOOM: f32 = 0.35;
const MAX_ZOOM: f32 = 2.5;
const GRID_MINOR: f32 = 40.0;
const GRID_MAJOR_EVERY: i32 = 10;
const PORT_HIT: f32 = 14.0;
const HOTBAR_SLOTS: usize = 9;
const BELT_HALF_WIDTH: f32 = 7.0;
const TARGET_FPS: f64 = 120.0;

const BG: Color = Color::from_rgba(22, 26, 32, 255);
const GRID_MINOR_C: Color = Color::from_rgba(48, 56, 68, 90);
const GRID_MAJOR_C: Color = Color::from_rgba(70, 82, 100, 120);
const NODE_BG: Color = Color::from_rgba(28, 32, 40, 245);
const NODE_BORDER: Color = Color::from_rgba(120, 140, 160, 180);
const CYAN: Color = Color::from_rgba(64, 220, 210, 255);
const CYAN_DIM: Color = Color::from_rgba(64, 220, 210, 100);
const BELT_YELLOW: Color = Color::from_rgba(210, 170, 55, 255);
const BELT_DARK: Color = Color::from_rgba(50, 44, 28, 255);
const POWER_C: Color = Color::from_rgba(255, 190, 70, 255);
const POWER_DIM: Color = Color::from_rgba(255, 190, 70, 90);
const TEXT: Color = Color::from_rgba(220, 230, 240, 255);
const TEXT_DIM: Color = Color::from_rgba(150, 160, 175, 255);
const PANEL: Color = Color::from_rgba(16, 18, 24, 240);
const ACCENT: Color = Color::from_rgba(255, 160, 60, 255);
const BAD: Color = Color::from_rgba(220, 80, 80, 120);
const ORE_C: Color = Color::from_rgba(140, 140, 150, 255);
const INGOT_C: Color = Color::from_rgba(190, 200, 220, 255);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Main,
    Play,
    Multiplayer,
    HostLobby,
    JoinLobby,
    Game,
}

struct CursorSample {
    x: f32,
    y: f32,
    t_ms: f32,
    selected: Option<BuildingKind>,
    facing: Facing,
}

struct PeerPresence {
    id: u8,
    x: f32,
    y: f32,
    selected: Option<BuildingKind>,
    facing: Facing,
    samples: VecDeque<CursorSample>,
}

struct Cam {
    x: f32,
    y: f32,
    zoom: f32,
}

impl Cam {
    fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        (
            (wx - self.x) * self.zoom + screen_width() * 0.5,
            (wy - self.y) * self.zoom + screen_height() * 0.5,
        )
    }

    fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        (
            (sx - screen_width() * 0.5) / self.zoom + self.x,
            (sy - screen_height() * 0.5) / self.zoom + self.y,
        )
    }
}

struct Ui {
    build_open: bool,
    build_category: BuildCategory,
    selected: Option<BuildingKind>,
    hotbar: [Option<BuildingKind>; HOTBAR_SLOTS],
    hotbar_index: usize,
    place_facing: Facing,
    wire_from: Option<(u32, usize)>,
    drag_node: Option<u32>,
    drag_off: (f32, f32),
    panning: bool,
    pan_last: (f32, f32),
}

impl Ui {
    fn new() -> Self {
        Self {
            build_open: false,
            build_category: BuildCategory::Energy,
            selected: None,
            hotbar: [None; HOTBAR_SLOTS],
            hotbar_index: 0,
            place_facing: Facing::E,
            wire_from: None,
            drag_node: None,
            drag_off: (0.0, 0.0),
            panning: false,
            pan_last: (0.0, 0.0),
        }
    }
}

struct App {
    screen: Screen,
    world: World,
    cam: Cam,
    ui: Ui,
    net: Option<NetHandle>,
    peers: HashMap<u8, PeerPresence>,
    host_code: String,
    host_addr: String,
    join_code: String,
    join_focus: bool,
    join_status: String,
    last_cursor_send: Instant,
    last_cursor_x: f32,
    last_cursor_y: f32,
    cursor_clock: Instant,
    local_player_id: u8,
}

impl App {
    fn new() -> Self {
        Self {
            screen: Screen::Main,
            world: World::new(),
            cam: Cam {
                x: 0.0,
                y: 0.0,
                zoom: 1.0,
            },
            ui: Ui::new(),
            net: None,
            peers: HashMap::new(),
            host_code: String::new(),
            host_addr: String::new(),
            join_code: String::new(),
            join_focus: false,
            join_status: String::new(),
            last_cursor_send: Instant::now(),
            last_cursor_x: f32::NAN,
            last_cursor_y: f32::NAN,
            cursor_clock: Instant::now(),
            local_player_id: 0,
        }
    }

    fn enter_game(&mut self) {
        self.screen = Screen::Game;
        self.ui = Ui::new();
        self.cam = Cam {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        };
        if self.net.is_none() {
            self.world.clear();
        }
        self.peers.clear();
        // Ask host for a fresh world snapshot (and tell peers we're in-world).
        if let Some(net) = self.net.as_ref() {
            let _ = net.tx.send(NetCommand::Announce);
        }
    }

    fn stop_net(&mut self) {
        if let Some(net) = self.net.take() {
            let _ = net.tx.send(NetCommand::Stop);
        }
        self.peers.clear();
        self.host_code.clear();
        self.host_addr.clear();
        self.join_status.clear();
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "FactoryPlanner".to_owned(),
        window_width: 1400,
        window_height: 900,
        high_dpi: true,
        platform: miniquad::conf::Platform {
            // Uncap vsync so we can run above 60Hz; we pace to TARGET_FPS ourselves.
            swap_interval: Some(0),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::new();
    let frame_budget = std::time::Duration::from_secs_f64(1.0 / TARGET_FPS);

    loop {
        let frame_start = Instant::now();
        let dt = get_frame_time().clamp(0.0, 0.05);
        let mouse = mouse_position();

        match app.screen {
            Screen::Main => screen_main(&mut app, mouse),
            Screen::Play => screen_play(&mut app, mouse),
            Screen::Multiplayer => screen_multiplayer(&mut app, mouse),
            Screen::HostLobby => screen_host_lobby(&mut app, mouse),
            Screen::JoinLobby => screen_join_lobby(&mut app, mouse),
            Screen::Game => {
                drain_net(&mut app);
                let (wx, wy) = app.cam.screen_to_world(mouse.0, mouse.1);
                handle_hotkeys(&mut app, wx, wy);
                handle_pan_zoom(&mut app, mouse);
                handle_world_input(&mut app, mouse, wx, wy);
                send_cursor_if_due(&mut app, wx, wy);
                playback_peer_cursors(&mut app);
                app.world.tick(dt);
                draw_game(&mut app, mouse, wx, wy);
            }
        }

        next_frame().await;
        let spent = frame_start.elapsed();
        if spent < frame_budget {
            std::thread::sleep(frame_budget - spent);
        }
    }
}

fn peer_color(id: u8) -> Color {
    const PALETTE: [Color; 8] = [
        Color::from_rgba(255, 120, 100, 255),
        Color::from_rgba(100, 200, 255, 255),
        Color::from_rgba(180, 255, 120, 255),
        Color::from_rgba(255, 180, 80, 255),
        Color::from_rgba(200, 140, 255, 255),
        Color::from_rgba(80, 220, 180, 255),
        Color::from_rgba(255, 100, 180, 255),
        Color::from_rgba(160, 180, 255, 255),
    ];
    PALETTE[id as usize % PALETTE.len()]
}

fn button(label: &str, x: f32, y: f32, w: f32, h: f32, mouse: (f32, f32)) -> bool {
    let hovered = mouse.0 >= x && mouse.0 <= x + w && mouse.1 >= y && mouse.1 <= y + h;
    draw_rectangle(
        x,
        y,
        w,
        h,
        if hovered {
            Color::from_rgba(40, 48, 60, 255)
        } else {
            Color::from_rgba(28, 34, 44, 255)
        },
    );
    draw_rectangle_lines(
        x,
        y,
        w,
        h,
        if hovered { 2.0 } else { 1.2 },
        if hovered { CYAN } else { NODE_BORDER },
    );
    let tw = measure_text(label, None, 22, 1.0).width;
    draw_text(
        label,
        x + (w - tw) * 0.5,
        y + h * 0.5 + 7.0,
        22.0,
        TEXT,
    );
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn draw_menu_backdrop(title: &str, subtitle: &str) {
    clear_background(BG);
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(18, 22, 28, 255),
    );
    for i in 0..40 {
        let t = i as f32 / 40.0;
        draw_line(
            0.0,
            screen_height() * t,
            screen_width(),
            screen_height() * t,
            1.0,
            Color::from_rgba(40, 50, 62, 40),
        );
    }
    draw_text(title, 80.0, 120.0, 56.0, CYAN);
    draw_text(subtitle, 84.0, 160.0, 20.0, TEXT_DIM);
}

fn screen_main(app: &mut App, mouse: (f32, f32)) {
    draw_menu_backdrop("FactoryPlanner", "Plan. Place. Power.");
    let bx = 80.0;
    let mut by = 220.0;
    let bw = 280.0;
    let bh = 48.0;
    if button("Play", bx, by, bw, bh, mouse) {
        app.screen = Screen::Play;
    }
    by += 64.0;
    if button("Exit Game", bx, by, bw, bh, mouse) {
        std::process::exit(0);
    }
}

fn screen_play(app: &mut App, mouse: (f32, f32)) {
    draw_menu_backdrop("Play", "Choose a mode");
    let bx = 80.0;
    let mut by = 220.0;
    let bw = 280.0;
    let bh = 48.0;
    if button("Single Player", bx, by, bw, bh, mouse) {
        app.stop_net();
        app.world.clear();
        app.enter_game();
    }
    by += 64.0;
    if button("Multiplayer", bx, by, bw, bh, mouse) {
        app.screen = Screen::Multiplayer;
    }
    by += 64.0;
    if button("Back", bx, by, bw, bh, mouse) {
        app.screen = Screen::Main;
    }
}

fn screen_multiplayer(app: &mut App, mouse: (f32, f32)) {
    draw_menu_backdrop(
        "Multiplayer",
        "Online relay — play across the world with a code",
    );
    let bx = 80.0;
    let mut by = 220.0;
    let bw = 320.0;
    let bh = 48.0;
    if button("Host Game", bx, by, bw, bh, mouse) {
        app.stop_net();
        app.world.clear();
        app.join_status = "Connecting online…".into();
        let handle = net::start_host();
        app.host_code = handle.code.clone();
        app.host_addr = handle.join_addr.clone();
        app.net = Some(handle);
        app.screen = Screen::HostLobby;
    }
    by += 64.0;
    if button("Join Game", bx, by, bw, bh, mouse) {
        app.stop_net();
        app.join_status.clear();
        app.join_code.clear();
        app.join_focus = true;
        app.screen = Screen::JoinLobby;
    }
    by += 64.0;
    if button("Back", bx, by, bw, bh, mouse) {
        app.screen = Screen::Play;
    }
}

fn screen_host_lobby(app: &mut App, mouse: (f32, f32)) {
    drain_net(app);
    draw_menu_backdrop("Host", "Share this code — UK, USA, anywhere");
    draw_text("Your session code", 84.0, 200.0, 20.0, TEXT_DIM);
    draw_text(
        &if app.host_code.is_empty() {
            "……".into()
        } else {
            app.host_code.clone()
        },
        84.0,
        250.0,
        64.0,
        CYAN,
    );
    draw_text(
        "Friends: Multiplayer → Join Game → type this code (no IP needed)",
        84.0,
        320.0,
        18.0,
        TEXT_DIM,
    );
    if !app.host_addr.is_empty() {
        draw_text(&format!("Relay: {}", app.host_addr), 84.0, 348.0, 16.0, TEXT_DIM);
    }
    if !app.join_status.is_empty() {
        draw_text(&app.join_status, 84.0, 380.0, 18.0, ACCENT);
    }
    if button("Enter World", 80.0, 430.0, 280.0, 48.0, mouse) {
        if app.net.is_some() {
            app.enter_game();
        }
    }
    if button("Back", 80.0, 494.0, 280.0, 48.0, mouse) {
        app.stop_net();
        app.screen = Screen::Multiplayer;
    }
}

fn text_field(
    label: &str,
    value: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    focused: bool,
    mouse: (f32, f32),
) -> bool {
    let hovered = mouse.0 >= x && mouse.0 <= x + w && mouse.1 >= y && mouse.1 <= y + h;
    draw_text(label, x, y - 8.0, 16.0, TEXT_DIM);
    draw_rectangle(x, y, w, h, Color::from_rgba(24, 28, 36, 255));
    draw_rectangle_lines(
        x,
        y,
        w,
        h,
        if focused { 2.0 } else { 1.2 },
        if focused {
            ACCENT
        } else if hovered {
            CYAN
        } else {
            NODE_BORDER
        },
    );
    let display = if focused {
        format!("{value}|")
    } else {
        value.to_string()
    };
    draw_text(&display, x + 12.0, y + h * 0.5 + 6.0, 28.0, TEXT);
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn handle_text_input(target: &mut String) {
    while let Some(c) = get_char_pressed() {
        if c.is_ascii_alphanumeric() && target.len() < 12 {
            target.push(c.to_ascii_uppercase());
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        target.pop();
    }
}

fn screen_join_lobby(app: &mut App, mouse: (f32, f32)) {
    drain_net(app);
    draw_menu_backdrop("Join Game", "Code only — works worldwide");

    if text_field(
        "Session code",
        &app.join_code,
        80.0,
        240.0,
        360.0,
        56.0,
        app.join_focus,
        mouse,
    ) {
        app.join_focus = true;
    }
    if app.join_focus {
        handle_text_input(&mut app.join_code);
    }

    if !app.join_status.is_empty() {
        draw_text(&app.join_status, 80.0, 330.0, 18.0, ACCENT);
    }

    if button("Connect", 80.0, 380.0, 280.0, 48.0, mouse) {
        app.stop_net();
        app.world.clear();
        app.join_status = "Connecting online…".into();
        let handle = net::start_client("", &app.join_code);
        app.net = Some(handle);
    }
    if button("Back", 80.0, 444.0, 280.0, 48.0, mouse) {
        app.stop_net();
        app.join_focus = false;
        app.screen = Screen::Multiplayer;
    }
}

fn send_world_snapshot(app: &App) {
    let Some(net) = app.net.as_ref() else {
        return;
    };
    let mut ids: Vec<u32> = app.world.nodes.keys().copied().collect();
    ids.sort_unstable();
    for id in ids {
        if let Some(n) = app.world.nodes.get(&id) {
            let _ = net.tx.send(NetCommand::Place {
                id,
                kind: n.kind,
                x: n.x,
                y: n.y,
                facing: n.facing,
            });
        }
    }
    for l in &app.world.links {
        let _ = net.tx.send(NetCommand::Link {
            power: true,
            from_node: l.from_node,
            from_port: l.from_port,
            to_node: l.to_node,
            to_port: l.to_port,
        });
    }
    for l in &app.world.belts {
        let _ = net.tx.send(NetCommand::Link {
            power: false,
            from_node: l.from_node,
            from_port: l.from_port,
            to_node: l.to_node,
            to_port: l.to_port,
        });
    }
}

fn playback_peer_cursors(app: &mut App) {
    for peer in app.peers.values_mut() {
        // Zero buffer: always show the newest sample immediately.
        while peer.samples.len() > 1 {
            peer.samples.pop_front();
        }
        if let Some(s) = peer.samples.back() {
            peer.x = s.x;
            peer.y = s.y;
            peer.selected = s.selected;
            peer.facing = s.facing;
        }
    }
}

fn drain_net(app: &mut App) {
    let events: Vec<NetEvent> = match app.net.as_ref() {
        Some(net) => {
            let mut evs = Vec::new();
            while let Ok(ev) = net.rx.try_recv() {
                evs.push(ev);
            }
            evs
        }
        None => return,
    };

    for ev in events {
        match ev {
            NetEvent::HostReady { code, addr } => {
                app.host_code = code;
                app.host_addr = addr;
            }
            NetEvent::Joined { player_id } => {
                app.local_player_id = player_id;
                app.world.set_id_namespace(player_id);
                app.join_status = if player_id == 0 {
                    "Online — share your code".into()
                } else {
                    format!("Joined as player {player_id}")
                };
                if app.screen == Screen::JoinLobby {
                    app.enter_game();
                }
            }
            NetEvent::JoinFailed { reason } => {
                app.join_status = format!("Failed: {reason}");
                app.net = None;
            }
            NetEvent::PeerHello { .. } => {
                // Anyone who hears a HELLO while hosting dumps their world so the
                // joiner catches up (buildings + wires).
                if app.net.as_ref().map(|n| n.is_host).unwrap_or(false) {
                    send_world_snapshot(app);
                    app.join_status = "Synced world to joiner".into();
                }
            }
            NetEvent::PeerCursor {
                id,
                x,
                y,
                selected,
                facing,
                t_ms,
            } => {
                if id != app.local_player_id {
                    let sample = CursorSample {
                        x,
                        y,
                        t_ms,
                        selected,
                        facing,
                    };
                    if let Some(peer) = app.peers.get_mut(&id) {
                        // Ignore out-of-order / duplicate timestamps.
                        if peer
                            .samples
                            .back()
                            .map(|s| t_ms + 0.01 < s.t_ms)
                            .unwrap_or(false)
                        {
                            continue;
                        }
                        peer.samples.push_back(sample);
                    } else {
                        let mut samples = VecDeque::new();
                        samples.push_back(sample);
                        app.peers.insert(
                            id,
                            PeerPresence {
                                id,
                                x,
                                y,
                                selected,
                                facing,
                                samples,
                            },
                        );
                    }
                }
            }
            NetEvent::PeerPlace {
                id,
                kind,
                x,
                y,
                facing,
            } => {
                let _ = app.world.place_node_with_id(id, kind, x, y, facing);
                app.join_status = format!("Peer placed {}", kind.short());
            }
            NetEvent::PeerRemove { id } => {
                app.world.remove_node(id);
            }
            NetEvent::PeerMove { id, x, y } => {
                app.world.force_move_node(id, x, y);
            }
            NetEvent::PeerRotate { id, facing } => {
                app.world.force_set_facing(id, facing);
            }
            NetEvent::PeerLink {
                power,
                from_node,
                from_port,
                to_node,
                to_port,
            } => {
                if power {
                    let _ = app
                        .world
                        .connect_power((from_node, from_port), (to_node, to_port));
                } else {
                    let _ = app
                        .world
                        .connect_belt((from_node, from_port), (to_node, to_port));
                }
            }
            NetEvent::PeerGone { id } => {
                app.peers.remove(&id);
            }
            NetEvent::Info(msg) => {
                app.join_status = msg;
            }
        }
    }
}

fn send_cursor_if_due(app: &mut App, wx: f32, wy: f32) {
    let Some(net) = app.net.as_ref() else {
        return;
    };
    // Every frame at 120Hz while in multiplayer — zero smoothing delay on receive.
    app.last_cursor_send = Instant::now();
    app.last_cursor_x = wx;
    app.last_cursor_y = wy;
    let t_ms = app.cursor_clock.elapsed().as_secs_f32() * 1000.0;
    let _ = net.tx.send(NetCommand::SetCursor {
        x: wx,
        y: wy,
        selected: app.ui.selected,
        facing: app.ui.place_facing,
        t_ms,
    });
}

fn handle_hotkeys(app: &mut App, wx: f32, wy: f32) {
    if is_key_pressed(KeyCode::B) {
        app.ui.build_open = !app.ui.build_open;
        if app.ui.build_open {
            app.ui.wire_from = None;
        }
    }
    if is_key_pressed(KeyCode::Escape) {
        if app.ui.build_open {
            app.ui.build_open = false;
        } else {
            app.ui.selected = None;
            app.ui.wire_from = None;
        }
    }
    if is_key_pressed(KeyCode::R) {
        app.ui.place_facing = app.ui.place_facing.rotate_cw();
        let rotated_id = if let Some(id) = app.ui.drag_node {
            if app.world.try_rotate_node(id) {
                Some(id)
            } else {
                None
            }
        } else if let Some(id) = app.world.hit_node(wx, wy) {
            if app.world.try_rotate_node(id) {
                Some(id)
            } else {
                None
            }
        } else {
            None
        };
        if let Some(id) = rotated_id {
            if let Some(n) = app.world.nodes.get(&id) {
                if let Some(net) = app.net.as_ref() {
                    let _ = net.tx.send(NetCommand::Rotate {
                        id,
                        facing: n.facing,
                    });
                }
            }
        }
    }

    for (i, key) in [
        KeyCode::Key1,
        KeyCode::Key2,
        KeyCode::Key3,
        KeyCode::Key4,
        KeyCode::Key5,
        KeyCode::Key6,
        KeyCode::Key7,
        KeyCode::Key8,
        KeyCode::Key9,
    ]
    .iter()
    .enumerate()
    {
        if is_key_pressed(*key) {
            if app.ui.build_open {
                if let Some(kind) = app.ui.selected {
                    app.ui.hotbar[i] = Some(kind);
                }
            } else {
                app.ui.hotbar_index = i;
                app.ui.selected = app.ui.hotbar[i];
                app.ui.wire_from = None;
            }
        }
    }
}

fn handle_pan_zoom(app: &mut App, mouse: (f32, f32)) {
    let cam = &mut app.cam;
    let ui = &mut app.ui;

    if is_mouse_button_pressed(MouseButton::Middle) {
        ui.panning = true;
        ui.pan_last = mouse;
    }
    if is_mouse_button_released(MouseButton::Middle) {
        ui.panning = false;
    }
    if ui.panning && is_mouse_button_down(MouseButton::Middle) {
        let dx = mouse.0 - ui.pan_last.0;
        let dy = mouse.1 - ui.pan_last.1;
        cam.x -= dx / cam.zoom;
        cam.y -= dy / cam.zoom;
        ui.pan_last = mouse;
    }

    let wheel = mouse_wheel().1;
    if wheel != 0.0 {
        let old = cam.zoom;
        cam.zoom = (cam.zoom * (1.0 + wheel * 0.1)).clamp(MIN_ZOOM, MAX_ZOOM);
        let before_x = (mouse.0 - screen_width() * 0.5) / old + cam.x;
        let before_y = (mouse.1 - screen_height() * 0.5) / old + cam.y;
        let after_x = (mouse.0 - screen_width() * 0.5) / cam.zoom + cam.x;
        let after_y = (mouse.1 - screen_height() * 0.5) / cam.zoom + cam.y;
        cam.x += before_x - after_x;
        cam.y += before_y - after_y;
    }
}

fn hotbar_geom() -> (f32, f32, f32, f32) {
    let slot = 56.0;
    let gap = 8.0;
    let width = HOTBAR_SLOTS as f32 * slot + (HOTBAR_SLOTS - 1) as f32 * gap;
    let bar_x = (screen_width() - width) * 0.5;
    let bar_y = screen_height() - slot - 18.0;
    (bar_x, bar_y, slot, gap)
}

fn point_in_hotbar(mx: f32, my: f32) -> Option<usize> {
    let (bar_x, bar_y, slot, gap) = hotbar_geom();
    if my < bar_y || my > bar_y + slot {
        return None;
    }
    for i in 0..HOTBAR_SLOTS {
        let x = bar_x + i as f32 * (slot + gap);
        if mx >= x && mx <= x + slot {
            return Some(i);
        }
    }
    None
}

fn build_menu_rect() -> Rect {
    let w = 520.0;
    let h = 460.0;
    Rect {
        x: (screen_width() - w) * 0.5,
        y: (screen_height() - h) * 0.5 - 30.0,
        w,
        h,
    }
}

fn place_building(app: &mut App, kind: BuildingKind, x: f32, y: f32, facing: Facing) {
    if let Some(id) = app.world.place_node(kind, x, y, facing) {
        if let Some(net) = app.net.as_ref() {
            let _ = net.tx.send(NetCommand::Place {
                id,
                kind,
                x,
                y,
                facing,
            });
            app.join_status = format!("Placed #{id}");
        }
    }
}

fn remove_building(app: &mut App, id: u32) {
    app.world.remove_node(id);
    if let Some(net) = app.net.as_ref() {
        let _ = net.tx.send(NetCommand::Remove { id });
    }
}

fn connect_ports_net(app: &mut App, from: (u32, usize), to: (u32, usize)) {
    if let Some((power, a, b)) = app.world.connect_ports(from, to) {
        if let Some(net) = app.net.as_ref() {
            let _ = net.tx.send(NetCommand::Link {
                power,
                from_node: a.0,
                from_port: a.1,
                to_node: b.0,
                to_port: b.1,
            });
        }
    }
}

fn handle_world_input(app: &mut App, mouse: (f32, f32), wx: f32, wy: f32) {
    if app.ui.build_open || app.ui.panning {
        return;
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(i) = point_in_hotbar(mouse.0, mouse.1) {
            app.ui.hotbar_index = i;
            app.ui.selected = app.ui.hotbar[i];
            app.ui.wire_from = None;
            return;
        }
    }
    if point_in_hotbar(mouse.0, mouse.1).is_some() {
        return;
    }

    let port_r = PORT_HIT / app.cam.zoom;

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(port) = app.world.hit_port(wx, wy, port_r) {
            if let Some(from) = app.ui.wire_from {
                if from != port {
                    connect_ports_net(app, from, port);
                }
                app.ui.wire_from = None;
            } else {
                app.ui.wire_from = Some(port);
                app.ui.selected = None;
            }
            return;
        }
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(kind) = app.ui.selected {
            if app.world.hit_node(wx, wy).is_none() {
                let probe = Node::new(kind, 0.0, 0.0, app.ui.place_facing);
                let x = wx - probe.w() * 0.5;
                let y = wy - probe.h() * 0.5;
                place_building(app, kind, x, y, app.ui.place_facing);
                return;
            }
        }
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(id) = app.world.hit_node(wx, wy) {
            if let Some(n) = app.world.nodes.get(&id) {
                app.ui.drag_node = Some(id);
                app.ui.drag_off = (wx - n.x, wy - n.y);
                app.ui.wire_from = None;
            }
        }
    }
    if is_mouse_button_released(MouseButton::Left) {
        if let Some(id) = app.ui.drag_node.take() {
            if let Some(n) = app.world.nodes.get(&id) {
                if let Some(net) = app.net.as_ref() {
                    let _ = net.tx.send(NetCommand::Move {
                        id,
                        x: n.x,
                        y: n.y,
                    });
                }
            }
        }
    }
    if let Some(id) = app.ui.drag_node {
        if is_mouse_button_down(MouseButton::Left) {
            let _ = app
                .world
                .try_move_node(id, wx - app.ui.drag_off.0, wy - app.ui.drag_off.1);
        }
    }

    if is_mouse_button_pressed(MouseButton::Right) {
        if app.ui.wire_from.take().is_some() {
            return;
        }
        if let Some(id) = app.world.hit_node(wx, wy) {
            remove_building(app, id);
        }
    }
}

fn draw_game(app: &mut App, mouse: (f32, f32), wx: f32, wy: f32) {
    clear_background(BG);
    draw_infinite_grid(&app.cam);
    draw_power_fields(&app.world, &app.cam);
    draw_belt_links(&app.world, &app.cam, &app.ui, wx, wy);
    draw_power_links(&app.world, &app.cam, &app.ui, wx, wy);
    draw_nodes(&app.world, &app.cam);
    draw_placement_ghost(&app.world, &app.ui, &app.cam, wx, wy);
    draw_peer_cursors(app);
    draw_hotbar(&app.ui);
    draw_text(
        "Belts: click item ports to wire  ·  Power: click energy ports",
        16.0,
        screen_height() - 12.0,
        16.0,
        TEXT_DIM,
    );
    if let Some(net) = app.net.as_ref() {
        if net.is_host {
            draw_text(
                &format!("Host  code {}  ·  {}", app.host_code, app.host_addr),
                16.0,
                28.0,
                18.0,
                TEXT_DIM,
            );
        }
        if !app.join_status.is_empty() {
            draw_text(&app.join_status, 16.0, 50.0, 16.0, ACCENT);
        }
    }
    if app.ui.build_open {
        draw_and_handle_build_menu(app, mouse);
    }
}

fn draw_infinite_grid(cam: &Cam) {
    let (x0, y0) = cam.screen_to_world(0.0, 0.0);
    let (x1, y1) = cam.screen_to_world(screen_width(), screen_height());
    let start_x = ((x0 / GRID_MINOR).floor() as i32) - 1;
    let end_x = ((x1 / GRID_MINOR).ceil() as i32) + 1;
    let start_y = ((y0 / GRID_MINOR).floor() as i32) - 1;
    let end_y = ((y1 / GRID_MINOR).ceil() as i32) + 1;

    for gx in start_x..=end_x {
        let wx = gx as f32 * GRID_MINOR;
        let (sx0, sy0) = cam.world_to_screen(wx, y0);
        let (_, sy1) = cam.world_to_screen(wx, y1);
        let major = gx.rem_euclid(GRID_MAJOR_EVERY) == 0;
        draw_line(
            sx0,
            sy0,
            sx0,
            sy1,
            if major { 1.25 } else { 1.0 },
            if major { GRID_MAJOR_C } else { GRID_MINOR_C },
        );
    }
    for gy in start_y..=end_y {
        let wy = gy as f32 * GRID_MINOR;
        let (sx0, sy0) = cam.world_to_screen(x0, wy);
        let (sx1, _) = cam.world_to_screen(x1, wy);
        let major = gy.rem_euclid(GRID_MAJOR_EVERY) == 0;
        draw_line(
            sx0,
            sy0,
            sx1,
            sy0,
            if major { 1.25 } else { 1.0 },
            if major { GRID_MAJOR_C } else { GRID_MINOR_C },
        );
    }
}

fn draw_power_fields(world: &World, cam: &Cam) {
    for n in world.nodes.values() {
        if n.kind != BuildingKind::PowerPole {
            continue;
        }
        let (cx, cy) = n.center();
        let (sx, sy) = cam.world_to_screen(cx, cy);
        let r = POLE_RADIUS * cam.zoom;
        draw_circle(
            sx,
            sy,
            r,
            if n.working {
                Color::from_rgba(255, 190, 70, 28)
            } else {
                Color::from_rgba(120, 120, 130, 18)
            },
        );
        draw_circle_lines(
            sx,
            sy,
            r,
            1.0,
            if n.working {
                POWER_DIM
            } else {
                Color::from_rgba(120, 120, 130, 50)
            },
        );
    }
}

fn draw_power_manhattan(cam: &Cam, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
    let (sx0, sy0) = cam.world_to_screen(x0, y0);
    let (sx1, sy1) = cam.world_to_screen(x1, y1);
    let mx = (sx0 + sx1) * 0.5;
    let t = (2.0 * cam.zoom).clamp(1.5, 3.0);
    draw_line(sx0, sy0, mx, sy0, t, color);
    draw_line(mx, sy0, mx, sy1, t, color);
    draw_line(mx, sy1, sx1, sy1, t, color);
}

fn draw_power_links(world: &World, cam: &Cam, ui: &Ui, wx: f32, wy: f32) {
    for l in &world.links {
        let Some(a) = world.nodes.get(&l.from_node) else {
            continue;
        };
        let Some(b) = world.nodes.get(&l.to_node) else {
            continue;
        };
        let Some((ax, ay)) = a.port_world(l.from_port) else {
            continue;
        };
        let Some((bx, by)) = b.port_world(l.to_port) else {
            continue;
        };
        draw_power_manhattan(cam, ax, ay, bx, by, POWER_C);
    }

    if let Some((nid, pid)) = ui.wire_from {
        if let Some(n) = world.nodes.get(&nid) {
            if let Some(p) = n.ports.get(pid) {
                if p.kind.is_energy() {
                    if let Some((ax, ay)) = n.port_world(pid) {
                        draw_power_manhattan(
                            cam,
                            ax,
                            ay,
                            wx,
                            wy,
                            Color::from_rgba(255, 190, 70, 150),
                        );
                    }
                }
            }
        }
    }
}

fn draw_belt_links(world: &World, cam: &Cam, ui: &Ui, wx: f32, wy: f32) {
    for belt in &world.belts {
        let Some(a) = world.nodes.get(&belt.from_node) else {
            continue;
        };
        let Some(b) = world.nodes.get(&belt.to_node) else {
            continue;
        };
        let Some((ax, ay)) = a.port_world(belt.from_port) else {
            continue;
        };
        let Some((bx, by)) = b.port_world(belt.to_port) else {
            continue;
        };
        let (sx0, sy0) = cam.world_to_screen(ax, ay);
        let (sx1, sy1) = cam.world_to_screen(bx, by);
        let mx = (sx0 + sx1) * 0.5;
        let hw = (BELT_HALF_WIDTH * cam.zoom).clamp(3.0, 12.0);
        draw_line(sx0, sy0, mx, sy0, hw * 2.0, BELT_DARK);
        draw_line(mx, sy0, mx, sy1, hw * 2.0, BELT_DARK);
        draw_line(mx, sy1, sx1, sy1, hw * 2.0, BELT_DARK);
        draw_line(sx0, sy0, mx, sy0, hw * 2.0 - 2.0, BELT_YELLOW);
        draw_line(mx, sy0, mx, sy1, hw * 2.0 - 2.0, BELT_YELLOW);
        draw_line(mx, sy1, sx1, sy1, hw * 2.0 - 2.0, BELT_YELLOW);

        for lane in 0..2 {
            for it in &belt.lanes[lane].items {
                let (iwx, iwy) = belt_item_world(world, belt, lane, it.dist);
                let (sx, sy) = cam.world_to_screen(iwx, iwy);
                let c = match it.item {
                    Item::IronOre => ORE_C,
                    Item::IronIngot => INGOT_C,
                };
                draw_circle(sx, sy, (4.0 * cam.zoom).clamp(2.5, 6.0), c);
            }
        }
    }

    if let Some((nid, pid)) = ui.wire_from {
        if let Some(n) = world.nodes.get(&nid) {
            if let Some(p) = n.ports.get(pid) {
                if !p.kind.is_energy() {
                    if let Some((ax, ay)) = n.port_world(pid) {
                        draw_power_manhattan(
                            cam,
                            ax,
                            ay,
                            wx,
                            wy,
                            Color::from_rgba(210, 170, 55, 160),
                        );
                    }
                }
            }
        }
    }
}

fn draw_nodes(world: &World, cam: &Cam) {
    let mut ids: Vec<u32> = world.nodes.keys().copied().collect();
    ids.sort_unstable();
    for id in ids {
        if let Some(n) = world.nodes.get(&id) {
            draw_node(cam, n);
        }
    }
}

fn draw_node(cam: &Cam, n: &Node) {
    let (sx, sy) = cam.world_to_screen(n.x, n.y);
    let w = n.w() * cam.zoom;
    let h = n.h() * cam.zoom;

    draw_rectangle(sx, sy, w, h, NODE_BG);
    let border = if n.kind.needs_power() && !n.powered {
        Color::from_rgba(180, 70, 70, 220)
    } else {
        NODE_BORDER
    };
    draw_rectangle_lines(sx, sy, w, h, 1.5 * cam.zoom.max(0.7), border);

    draw_rectangle(
        sx,
        sy,
        w,
        26.0 * cam.zoom,
        Color::from_rgba(36, 42, 52, 255),
    );
    draw_text(
        n.kind.label(),
        sx + 8.0 * cam.zoom,
        sy + 18.0 * cam.zoom,
        (15.0 * cam.zoom).clamp(10.0, 18.0),
        TEXT,
    );

    let body = match n.kind {
        BuildingKind::Solar => format!("{:+.0} e/s", SOLAR_POWER),
        BuildingKind::PowerPole => {
            if n.working {
                "Field ON".into()
            } else {
                "No network".into()
            }
        }
        BuildingKind::OreNode => format!(
            "out {:.1}\n{}",
            n.out_ore,
            if n.powered { "Powered" } else { "No power" }
        ),
        BuildingKind::Smelter => format!(
            "in {:.1}\nout {:.1}\n{}",
            n.in_ore,
            n.out_ingot,
            if n.powered { "Powered" } else { "No power" }
        ),
        BuildingKind::Box => format!("ore {:.0}\ningot {:.0}", n.store_ore, n.store_ingot),
        BuildingKind::Splitter => String::new(),
    };

    let line_h = 15.0 * cam.zoom;
    for (i, line) in body.lines().enumerate() {
        draw_text(
            line,
            sx + 10.0 * cam.zoom,
            sy + 44.0 * cam.zoom + i as f32 * line_h,
            (13.0 * cam.zoom).clamp(9.0, 16.0),
            TEXT_DIM,
        );
    }

    for p in &n.ports {
        let (px, py) = cam.world_to_screen(n.x + p.ox, n.y + p.oy);
        let r = (6.0 * cam.zoom).clamp(4.0, 9.0);
        let energy = p.kind.is_energy();
        let is_out = p.kind.is_item_out() || matches!(p.kind, PortKind::EnergyOut);
        let fill = if energy { POWER_C } else { CYAN };
        if is_out || matches!(p.kind, PortKind::EnergyAny) {
            draw_circle(px, py, r, fill);
        } else {
            draw_circle(px, py, r, Color::from_rgba(30, 36, 44, 255));
            draw_circle_lines(px, py, r, 2.0, fill);
        }
    }
}

fn draw_placement_ghost(world: &World, ui: &Ui, cam: &Cam, wx: f32, wy: f32) {
    if ui.build_open || ui.wire_from.is_some() || ui.drag_node.is_some() {
        return;
    }
    let Some(kind) = ui.selected else {
        return;
    };
    let probe = Node::new(kind, 0.0, 0.0, ui.place_facing);
    let x = wx - probe.w() * 0.5;
    let y = wy - probe.h() * 0.5;
    let blocked = world.collides(x, y, probe.w(), probe.h(), None);
    let (sx, sy) = cam.world_to_screen(x, y);
    let w = probe.w() * cam.zoom;
    let h = probe.h() * cam.zoom;
    draw_rectangle(
        sx,
        sy,
        w,
        h,
        if blocked {
            BAD
        } else {
            Color::from_rgba(64, 220, 210, 35)
        },
    );
    draw_rectangle_lines(
        sx,
        sy,
        w,
        h,
        1.5,
        if blocked {
            Color::from_rgba(220, 80, 80, 200)
        } else {
            CYAN_DIM
        },
    );
    draw_text(
        kind.label(),
        sx + 10.0,
        sy + 22.0,
        (15.0 * cam.zoom).clamp(10.0, 18.0),
        if blocked {
            Color::from_rgba(255, 160, 160, 255)
        } else {
            CYAN
        },
    );
    if kind == BuildingKind::PowerPole {
        let (cx, cy) = cam.world_to_screen(x + probe.w() * 0.5, y + probe.h() * 0.5);
        draw_circle_lines(cx, cy, POLE_RADIUS * cam.zoom, 1.0, POWER_DIM);
    }
}

fn draw_peer_ghost(cam: &Cam, kind: BuildingKind, wx: f32, wy: f32, facing: Facing, color: Color) {
    let probe = Node::new(kind, 0.0, 0.0, facing);
    let x = wx - probe.w() * 0.5;
    let y = wy - probe.h() * 0.5;
    let (sx, sy) = cam.world_to_screen(x, y);
    let w = probe.w() * cam.zoom;
    let h = probe.h() * cam.zoom;
    let mut fill = color;
    fill.a = 0.2;
    let mut outline = color;
    outline.a = 0.7;
    draw_rectangle(sx, sy, w, h, fill);
    draw_rectangle_lines(sx, sy, w, h, 1.5, outline);
}

fn draw_peer_cursors(app: &App) {
    for peer in app.peers.values() {
        let color = peer_color(peer.id);
        let (sx, sy) = app.cam.world_to_screen(peer.x, peer.y);
        let size = 12.0;
        draw_triangle(
            Vec2::new(sx, sy),
            Vec2::new(sx + size, sy + size * 0.7),
            Vec2::new(sx + size * 0.25, sy + size),
            color,
        );
        if let Some(kind) = peer.selected {
            draw_peer_ghost(&app.cam, kind, peer.x, peer.y, peer.facing, color);
        }
    }
}

fn draw_hotbar(ui: &Ui) {
    let (bar_x, bar_y, slot, gap) = hotbar_geom();
    for i in 0..HOTBAR_SLOTS {
        let x = bar_x + i as f32 * (slot + gap);
        let selected = i == ui.hotbar_index;
        draw_rectangle(x, bar_y, slot, slot, PANEL);
        draw_rectangle_lines(
            x,
            bar_y,
            slot,
            slot,
            if selected { 2.5 } else { 1.2 },
            if selected { ACCENT } else { NODE_BORDER },
        );
        draw_text(
            &(i + 1).to_string(),
            x + 6.0,
            bar_y + 14.0,
            14.0,
            TEXT_DIM,
        );
        if let Some(kind) = ui.hotbar[i] {
            draw_text(kind.short(), x + 6.0, bar_y + 34.0, 15.0, TEXT);
        }
    }
}

fn draw_and_handle_build_menu(app: &mut App, mouse: (f32, f32)) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(0, 0, 0, 140),
    );

    let r = build_menu_rect();
    draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, NODE_BORDER);
    draw_text("Build", r.x + 20.0, r.y + 32.0, 28.0, TEXT);
    draw_text(
        "Place buildings from the hotbar. Power: click energy ports. R rotates.",
        r.x + 20.0,
        r.y + 54.0,
        15.0,
        TEXT_DIM,
    );

    let tab_y = r.y + 72.0;
    let tab_w = (r.w - 32.0) / BuildCategory::ALL.len() as f32;
    for (i, cat) in BuildCategory::ALL.iter().enumerate() {
        let x = r.x + 16.0 + i as f32 * tab_w;
        let active = app.ui.build_category == *cat;
        let hovered = mouse.0 >= x
            && mouse.0 <= x + tab_w - 4.0
            && mouse.1 >= tab_y
            && mouse.1 <= tab_y + 32.0;
        draw_rectangle(
            x,
            tab_y,
            tab_w - 4.0,
            32.0,
            if active {
                Color::from_rgba(48, 58, 72, 255)
            } else if hovered {
                Color::from_rgba(36, 42, 52, 255)
            } else {
                Color::from_rgba(24, 28, 34, 255)
            },
        );
        draw_rectangle_lines(
            x,
            tab_y,
            tab_w - 4.0,
            32.0,
            1.0,
            if active { CYAN } else { NODE_BORDER },
        );
        draw_text(cat.label(), x + 8.0, tab_y + 21.0, 15.0, TEXT);
        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            app.ui.build_category = *cat;
        }
    }

    let items = BuildingKind::in_category(app.ui.build_category);
    let start_y = tab_y + 48.0;
    let row_h = 52.0;
    for (i, kind) in items.iter().enumerate() {
        let y = start_y + i as f32 * (row_h + 8.0);
        let row = Rect {
            x: r.x + 16.0,
            y,
            w: r.w - 32.0,
            h: row_h,
        };
        let hovered = mouse.0 >= row.x
            && mouse.0 <= row.x + row.w
            && mouse.1 >= row.y
            && mouse.1 <= row.y + row.h;
        let selected = app.ui.selected == Some(*kind);
        draw_rectangle(
            row.x,
            row.y,
            row.w,
            row.h,
            if hovered || selected {
                Color::from_rgba(40, 48, 60, 255)
            } else {
                Color::from_rgba(26, 30, 38, 255)
            },
        );
        draw_rectangle_lines(
            row.x,
            row.y,
            row.w,
            row.h,
            1.2,
            if selected { CYAN } else { NODE_BORDER },
        );
        draw_text(kind.label(), row.x + 16.0, row.y + 32.0, 20.0, TEXT);
        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            app.ui.selected = Some(*kind);
            app.ui.hotbar[app.ui.hotbar_index] = Some(*kind);
            app.ui.build_open = false;
        }
    }
}
