#![cfg_attr(windows, windows_subsystem = "windows")]

mod sim;
mod net;

use macroquad::prelude::*;
use net::{NetCommand, NetEvent, NetHandle};
use sim::*;
use std::collections::HashMap;
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum CornerTool {
    Build,
    TechTree,
    Map,
    NodeChart,
}

impl CornerTool {
    const ALL: [CornerTool; 4] = [
        Self::Build,
        Self::TechTree,
        Self::Map,
        Self::NodeChart,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Build => "Build",
            Self::TechTree => "Tech",
            Self::Map => "Map",
            Self::NodeChart => "Nodes",
        }
    }
}

struct Icons {
    hammer: Option<Texture2D>,
}

impl Icons {
    async fn load() -> Self {
        let hammer = match load_texture("assets/icons/hammer.png").await {
            Ok(t) => {
                t.set_filter(FilterMode::Linear);
                Some(t)
            }
            Err(_) => match load_texture(
                "src/Assets/Icons/hammer-icon-on-black-background-black-flat-style-vector-illustration.png",
            )
            .await
            {
                Ok(t) => {
                    t.set_filter(FilterMode::Linear);
                    Some(t)
                }
                Err(e) => {
                    eprintln!("hammer icon missing: {e}");
                    None
                }
            },
        };
        Self { hammer }
    }
}

struct PeerPresence {
    id: u8,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    selected: Option<BuildingKind>,
    facing: Facing,
    last_sample_t: f32,
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

#[derive(Clone, Copy)]
enum ContextTarget {
    Empty,
    Building(u32),
}

#[derive(Clone, Copy)]
struct ContextMenu {
    sx: f32,
    sy: f32,
    target: ContextTarget,
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
    /// Dragging a building from the build menu onto the hotbar (Factorio-style).
    palette_drag: Option<BuildingKind>,
    palette_drag_origin: (f32, f32),
    /// Rearranging / clearing a hotbar slot by drag.
    hotbar_drag_from: Option<usize>,
    hotbar_drag_origin: (f32, f32),
    context_menu: Option<ContextMenu>,
    /// Non-build corner-wheel panels (tech / map / node chart).
    overlay: Option<CornerTool>,
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
            palette_drag: None,
            palette_drag_origin: (0.0, 0.0),
            hotbar_drag_from: None,
            hotbar_drag_origin: (0.0, 0.0),
            context_menu: None,
            overlay: None,
        }
    }

    fn clear_tool(&mut self) {
        self.selected = None;
        self.wire_from = None;
        self.palette_drag = None;
        self.hotbar_drag_from = None;
    }

    fn open_build(&mut self) {
        self.build_open = true;
        self.overlay = None;
        self.wire_from = None;
        self.context_menu = None;
        self.drag_node = None;
    }

    fn toggle_build(&mut self) {
        if self.build_open {
            self.build_open = false;
            self.palette_drag = None;
        } else {
            self.open_build();
        }
    }

    fn activate_corner(&mut self, tool: CornerTool) {
        match tool {
            CornerTool::Build => self.toggle_build(),
            other => {
                self.build_open = false;
                self.palette_drag = None;
                if self.overlay == Some(other) {
                    self.overlay = None;
                } else {
                    self.overlay = Some(other);
                }
                self.context_menu = None;
            }
        }
    }
}

struct App {
    screen: Screen,
    world: World,
    cam: Cam,
    ui: Ui,
    icons: Icons,
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
    last_snap_send: Instant,
    applying_snap: bool,
}

impl App {
    fn new(icons: Icons) -> Self {
        Self {
            screen: Screen::Main,
            world: World::new(),
            cam: Cam {
                x: 0.0,
                y: 0.0,
                zoom: 1.0,
            },
            ui: Ui::new(),
            icons,
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
            last_snap_send: Instant::now(),
            applying_snap: false,
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
            if !net.is_host {
                let _ = net.tx.send(NetCommand::WantSnap);
            }
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
        high_dpi: false,
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
    let icons = Icons::load().await;
    let mut app = App::new(icons);
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
                maybe_host_snapshot(&mut app);
                let (wx, wy) = app.cam.screen_to_world(mouse.0, mouse.1);
                handle_hotkeys(&mut app, wx, wy);
                handle_pan_zoom(&mut app, mouse);
                handle_hud_input(&mut app, mouse, wx, wy);
                if !app.ui.build_open
                    && app.ui.context_menu.is_none()
                    && app.ui.overlay.is_none()
                {
                    handle_world_input(&mut app, mouse, wx, wy);
                }
                send_cursor_if_due(&mut app, wx, wy);
                advance_peer_cursors(&mut app, dt);
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
        "P2P (iroh) — play across the world with a code",
    );
    let bx = 80.0;
    let mut by = 220.0;
    let bw = 320.0;
    let bh = 48.0;
    if button("Host Game", bx, by, bw, bh, mouse) {
        app.stop_net();
        app.world.clear();
        app.join_status = "Code reserved — finishing online setup…".into();
        let handle = net::start_host();
        // Show code immediately; P2P/MQTT finish in the background.
        app.host_code = handle.code.clone();
        app.host_addr = "starting…".into();
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
    draw_menu_backdrop("Host", "Share your code — setup continues while you play");
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
        "Friends: Join with this code (same version). Best once status says joinable.",
        84.0,
        320.0,
        18.0,
        TEXT_DIM,
    );
    if !app.host_addr.is_empty() {
        draw_text(&format!("Transport: {}", app.host_addr), 84.0, 348.0, 16.0, TEXT_DIM);
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
    draw_menu_backdrop("Join Game", "Code only — P2P works worldwide");

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
    let _ = net.tx.send(NetCommand::SnapBegin);
    push_world_ops(app, net);
    let _ = net.tx.send(NetCommand::SnapEnd);
}

fn push_world_ops(app: &App, net: &NetHandle) {
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
                request: false,
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
            request: false,
        });
    }
    for l in &app.world.belts {
        let _ = net.tx.send(NetCommand::Link {
            power: false,
            from_node: l.from_node,
            from_port: l.from_port,
            to_node: l.to_node,
            to_port: l.to_port,
            request: false,
        });
    }
}

fn maybe_host_snapshot(app: &mut App) {
    let Some(net) = app.net.as_ref() else {
        return;
    };
    if !net.is_host {
        return;
    }
    // Soft reconcile (no wipe) so lossy brokers heal without flickering.
    if app.last_snap_send.elapsed().as_millis() < 500 {
        return;
    }
    app.last_snap_send = Instant::now();
    push_world_ops(app, net);
}

fn advance_peer_cursors(app: &mut App, dt: f32) {
    for peer in app.peers.values_mut() {
        // Extrapolate with damping so motion stays smooth between sparse samples.
        peer.x += peer.vx * dt;
        peer.y += peer.vy * dt;
        peer.vx *= 0.92;
        peer.vy *= 0.92;
        if peer.vx.abs() < 1.0 {
            peer.vx = 0.0;
        }
        if peer.vy.abs() < 1.0 {
            peer.vy = 0.0;
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

    let is_host = app.net.as_ref().map(|n| n.is_host).unwrap_or(false);

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
                    "Host online — share your code".into()
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
            NetEvent::PeerHello | NetEvent::WantSnap => {
                if is_host {
                    send_world_snapshot(app);
                    app.last_snap_send = Instant::now();
                    app.join_status = "Synced world to joiner".into();
                }
            }
            NetEvent::PlaceRequest {
                kind,
                x,
                y,
                facing,
            } => {
                if is_host {
                    if let Some(id) = app.world.place_node(kind, x, y, facing) {
                        if let Some(net) = app.net.as_ref() {
                            let _ = net.tx.send(NetCommand::Place {
                                id,
                                kind,
                                x,
                                y,
                                facing,
                                request: false,
                            });
                        }
                        app.join_status = format!("Host placed {}", kind.short());
                    }
                }
            }
            NetEvent::RemoveRequest { id } => {
                if is_host {
                    app.world.remove_node(id);
                    if let Some(net) = app.net.as_ref() {
                        let _ = net.tx.send(NetCommand::Remove {
                            id,
                            request: false,
                        });
                    }
                }
            }
            NetEvent::MoveRequest { id, x, y } => {
                if is_host {
                    app.world.force_move_node(id, x, y);
                    if let Some(net) = app.net.as_ref() {
                        let _ = net.tx.send(NetCommand::Move {
                            id,
                            x,
                            y,
                            request: false,
                        });
                    }
                }
            }
            NetEvent::RotateRequest { id, facing } => {
                if is_host {
                    app.world.force_set_facing(id, facing);
                    if let Some(net) = app.net.as_ref() {
                        let _ = net.tx.send(NetCommand::Rotate {
                            id,
                            facing,
                            request: false,
                        });
                    }
                }
            }
            NetEvent::LinkRequest {
                power,
                from_node,
                from_port,
                to_node,
                to_port,
            } => {
                if is_host {
                    let ok = if power {
                        app.world
                            .connect_power((from_node, from_port), (to_node, to_port))
                    } else {
                        app.world
                            .connect_belt((from_node, from_port), (to_node, to_port))
                    };
                    if ok {
                        if let Some(net) = app.net.as_ref() {
                            let _ = net.tx.send(NetCommand::Link {
                                power,
                                from_node,
                                from_port,
                                to_node,
                                to_port,
                                request: false,
                            });
                        }
                    }
                }
            }
            NetEvent::SnapBegin => {
                if !is_host {
                    app.world.nodes.clear();
                    app.world.links.clear();
                    app.world.belts.clear();
                    app.applying_snap = true;
                    app.join_status = "Receiving world…".into();
                }
            }
            NetEvent::SnapEnd => {
                if !is_host {
                    app.applying_snap = false;
                    app.join_status = "World synced".into();
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
                if id == app.local_player_id {
                    continue;
                }
                if let Some(peer) = app.peers.get_mut(&id) {
                    if t_ms + 0.5 < peer.last_sample_t {
                        continue;
                    }
                    let dt = ((t_ms - peer.last_sample_t) / 1000.0).max(0.001);
                    peer.vx = (x - peer.x) / dt;
                    peer.vy = (y - peer.y) / dt;
                    // Soft-correct toward sample (keeps path smooth).
                    peer.x = peer.x * 0.25 + x * 0.75;
                    peer.y = peer.y * 0.25 + y * 0.75;
                    peer.selected = selected;
                    peer.facing = facing;
                    peer.last_sample_t = t_ms;
                } else {
                    app.peers.insert(
                        id,
                        PeerPresence {
                            id,
                            x,
                            y,
                            vx: 0.0,
                            vy: 0.0,
                            selected,
                            facing,
                            last_sample_t: t_ms,
                        },
                    );
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
                if !app.applying_snap {
                    app.join_status = format!("Synced {}", kind.short());
                }
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
                // Surface peer presence clearly in lobby + HUD.
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
        app.ui.toggle_build();
    }
    if is_key_pressed(KeyCode::Escape) {
        if app.ui.context_menu.take().is_some() {
            // closed pie / context
        } else if app.ui.overlay.take().is_some() {
            // closed corner overlay
        } else if app.ui.build_open {
            app.ui.build_open = false;
            app.ui.palette_drag = None;
        } else if app.ui.wire_from.take().is_some() || app.ui.selected.take().is_some() {
            app.ui.hotbar_drag_from = None;
        }
    }
    if is_key_pressed(KeyCode::Q) {
        app.ui.clear_tool();
        app.ui.context_menu = None;
    }
    if is_key_pressed(KeyCode::R) {
        app.ui.place_facing = app.ui.place_facing.rotate_cw();
        let rotated_id = if let Some(id) = app.ui.drag_node {
            if app.world.try_rotate_node(id) {
                Some(id)
            } else {
                None
            }
        } else if let Some(ContextTarget::Building(id)) =
            app.ui.context_menu.as_ref().map(|m| m.target)
        {
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
                        request: !net.is_host,
                    });
                }
            }
        }
    }
    if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
        if let Some(ContextTarget::Building(id)) =
            app.ui.context_menu.as_ref().map(|m| m.target)
        {
            remove_building(app, id);
            app.ui.context_menu = None;
        } else if app.ui.context_menu.is_none() && !app.ui.build_open {
            if let Some(id) = app.world.hit_node(wx, wy) {
                remove_building(app, id);
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
                let kind = app.ui.palette_drag.or(app.ui.selected);
                if let Some(kind) = kind {
                    app.ui.hotbar[i] = Some(kind);
                    app.ui.hotbar_index = i;
                }
            } else {
                app.ui.hotbar_index = i;
                app.ui.selected = app.ui.hotbar[i];
                app.ui.wire_from = None;
                app.ui.context_menu = None;
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

fn ui_scale() -> f32 {
    let by_h = screen_height() / 900.0;
    let by_w = screen_width() / 1400.0;
    by_h.max(by_w).clamp(1.0, 1.5)
}

fn s(v: f32) -> f32 {
    v * ui_scale()
}

/// Floating hotbar — only as wide as the slots, sits above the bottom edge.
fn hotbar_geom() -> (f32, f32, f32, f32) {
    let slot = s(56.0);
    let gap = s(6.0);
    let width = HOTBAR_SLOTS as f32 * slot + (HOTBAR_SLOTS - 1) as f32 * gap;
    let x = (screen_width() - width) * 0.5;
    let y = screen_height() - slot - s(22.0);
    (x, y, slot, gap)
}

/// Vertical tool rail in the bottom-right — icon-only, labels appear on hover.
fn tool_button_rect(index: usize) -> Rect {
    let size = s(48.0);
    let gap = s(10.0);
    let total_h = 4.0 * size + 3.0 * gap;
    let x = screen_width() - size - s(18.0);
    let y0 = screen_height() - total_h - s(22.0);
    Rect {
        x,
        y: y0 + index as f32 * (size + gap),
        w: size,
        h: size,
    }
}

fn point_in_tool_button(mx: f32, my: f32) -> Option<CornerTool> {
    for (i, tool) in CornerTool::ALL.iter().enumerate() {
        let r = tool_button_rect(i);
        if mx >= r.x && mx <= r.x + r.w && my >= r.y && my <= r.y + r.h {
            return Some(*tool);
        }
    }
    None
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

fn point_in_hud_chrome(mx: f32, my: f32) -> bool {
    point_in_hotbar(mx, my).is_some() || point_in_tool_button(mx, my).is_some()
}

fn kind_swatch(kind: BuildingKind) -> Color {
    match kind {
        BuildingKind::Solar => Color::from_rgba(80, 160, 220, 255),
        BuildingKind::PowerPole => POWER_C,
        BuildingKind::OreNode => ORE_C,
        BuildingKind::Smelter => Color::from_rgba(220, 120, 70, 255),
        BuildingKind::Box => Color::from_rgba(160, 170, 190, 255),
        BuildingKind::Splitter => BELT_YELLOW,
    }
}

fn draw_tech_icon(cx: f32, cy: f32, color: Color) {
    let u = s(1.0);
    draw_circle(cx, cy - 5.5 * u, 3.0 * u, color);
    draw_circle(cx - 7.5 * u, cy + 5.0 * u, 3.0 * u, color);
    draw_circle(cx + 7.5 * u, cy + 5.0 * u, 3.0 * u, color);
    draw_line(cx, cy - 5.5 * u, cx - 7.5 * u, cy + 5.0 * u, 1.7 * u, color);
    draw_line(cx, cy - 5.5 * u, cx + 7.5 * u, cy + 5.0 * u, 1.7 * u, color);
}

fn draw_map_icon(cx: f32, cy: f32, color: Color) {
    let u = s(1.0);
    draw_rectangle_lines(cx - 9.0 * u, cy - 7.5 * u, 18.0 * u, 15.0 * u, 1.7 * u, color);
    draw_line(cx - 3.0 * u, cy - 7.5 * u, cx - 3.0 * u, cy + 7.5 * u, 1.4 * u, color);
    draw_line(cx + 3.0 * u, cy - 7.5 * u, cx + 3.0 * u, cy + 7.5 * u, 1.4 * u, color);
    draw_line(cx - 9.0 * u, cy - 1.0 * u, cx + 9.0 * u, cy + 1.5 * u, 1.4 * u, color);
}

fn draw_nodes_icon(cx: f32, cy: f32, color: Color) {
    let u = s(1.0);
    draw_circle(cx - 7.5 * u, cy - 5.0 * u, 3.2 * u, color);
    draw_circle(cx + 7.5 * u, cy - 5.0 * u, 3.2 * u, color);
    draw_circle(cx, cy + 7.5 * u, 3.2 * u, color);
    draw_line(cx - 7.5 * u, cy - 5.0 * u, cx + 7.5 * u, cy - 5.0 * u, 1.6 * u, color);
    draw_line(cx - 7.5 * u, cy - 5.0 * u, cx, cy + 7.5 * u, 1.6 * u, color);
    draw_line(cx + 7.5 * u, cy - 5.0 * u, cx, cy + 7.5 * u, 1.6 * u, color);
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
    let is_host = app.net.as_ref().map(|n| n.is_host).unwrap_or(true);
    if app.net.is_none() || is_host {
        if let Some(id) = app.world.place_node(kind, x, y, facing) {
            if let Some(net) = app.net.as_ref() {
                let _ = net.tx.send(NetCommand::Place {
                    id,
                    kind,
                    x,
                    y,
                    facing,
                    request: false,
                });
            }
            app.join_status = format!("Placed #{id}");
        }
    } else if let Some(net) = app.net.as_ref() {
        let _ = net.tx.send(NetCommand::Place {
            id: 0,
            kind,
            x,
            y,
            facing,
            request: true,
        });
        app.join_status = format!("Placing {}…", kind.short());
    }
}

fn remove_building(app: &mut App, id: u32) {
    let is_host = app.net.as_ref().map(|n| n.is_host).unwrap_or(true);
    if app.net.is_none() || is_host {
        app.world.remove_node(id);
        if let Some(net) = app.net.as_ref() {
            let _ = net.tx.send(NetCommand::Remove {
                id,
                request: false,
            });
        }
    } else if let Some(net) = app.net.as_ref() {
        let _ = net.tx.send(NetCommand::Remove { id, request: true });
        app.join_status = "Removing…".into();
    }
}

fn connect_ports_net(app: &mut App, from: (u32, usize), to: (u32, usize)) {
    let is_host = app.net.as_ref().map(|n| n.is_host).unwrap_or(true);
    if app.net.is_none() || is_host {
        if let Some((power, a, b)) = app.world.connect_ports(from, to) {
            if let Some(net) = app.net.as_ref() {
                let _ = net.tx.send(NetCommand::Link {
                    power,
                    from_node: a.0,
                    from_port: a.1,
                    to_node: b.0,
                    to_port: b.1,
                    request: false,
                });
            }
        }
    } else if let Some(net) = app.net.as_ref() {
        // Guess power vs belt from port kinds locally for the request.
        let power = app
            .world
            .nodes
            .get(&from.0)
            .and_then(|n| n.ports.get(from.1))
            .map(|p| p.kind.is_energy())
            .unwrap_or(false);
        let _ = net.tx.send(NetCommand::Link {
            power,
            from_node: from.0,
            from_port: from.1,
            to_node: to.0,
            to_port: to.1,
            request: true,
        });
    }
}

fn handle_hud_input(app: &mut App, mouse: (f32, f32), wx: f32, wy: f32) {
    // Finish palette → hotbar drops even while the build menu is open.
    if is_mouse_button_released(MouseButton::Left) {
        if let Some(kind) = app.ui.palette_drag.take() {
            let dx = mouse.0 - app.ui.palette_drag_origin.0;
            let dy = mouse.1 - app.ui.palette_drag_origin.1;
            let dragged = dx * dx + dy * dy > 64.0;
            if let Some(i) = point_in_hotbar(mouse.0, mouse.1) {
                app.ui.hotbar[i] = Some(kind);
                app.ui.hotbar_index = i;
            } else if !dragged {
                // Click (not drag): equip and close menu.
                app.ui.selected = Some(kind);
                app.ui.hotbar[app.ui.hotbar_index] = Some(kind);
                app.ui.build_open = false;
                app.ui.wire_from = None;
            }
            return;
        }
        if let Some(from) = app.ui.hotbar_drag_from.take() {
            let dx = mouse.0 - app.ui.hotbar_drag_origin.0;
            let dy = mouse.1 - app.ui.hotbar_drag_origin.1;
            let dragged = dx * dx + dy * dy > 64.0;
            if let Some(to) = point_in_hotbar(mouse.0, mouse.1) {
                if dragged && to != from {
                    app.ui.hotbar.swap(from, to);
                }
                app.ui.hotbar_index = to;
                app.ui.selected = app.ui.hotbar[to];
            } else if dragged && !point_in_hud_chrome(mouse.0, mouse.1) {
                // Dragged off the bar → clear slot.
                app.ui.hotbar[from] = None;
                if app.ui.hotbar_index == from {
                    app.ui.selected = None;
                }
            } else {
                app.ui.hotbar_index = from;
                app.ui.selected = app.ui.hotbar[from];
            }
            return;
        }
    }

    if handle_context_menu_input(app, mouse) {
        return;
    }

    // Dismiss overlay when clicking outside its panel (wheel still clickable).
    if app.ui.overlay.is_some() && is_mouse_button_pressed(MouseButton::Left) {
        if point_in_tool_button(mouse.0, mouse.1).is_none() {
            let w = 520.0;
            let h = 360.0;
            let x = (screen_width() - w) * 0.5;
            let y = (screen_height() - h) * 0.5 - 40.0;
            let inside =
                mouse.0 >= x && mouse.0 <= x + w && mouse.1 >= y && mouse.1 <= y + h;
            if !inside {
                app.ui.overlay = None;
                return;
            }
        }
    }

    if app.ui.panning {
        return;
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(tool) = point_in_tool_button(mouse.0, mouse.1) {
            app.ui.activate_corner(tool);
            return;
        }
        if let Some(i) = point_in_hotbar(mouse.0, mouse.1) {
            app.ui.context_menu = None;
            if app.ui.build_open {
                // While B is open, clicking a slot just highlights it as drop target.
                app.ui.hotbar_index = i;
            } else if app.ui.hotbar[i].is_some() {
                app.ui.hotbar_drag_from = Some(i);
                app.ui.hotbar_drag_origin = mouse;
                app.ui.hotbar_index = i;
                app.ui.selected = app.ui.hotbar[i];
                app.ui.wire_from = None;
            } else {
                // Empty slot: clear tool / select empty.
                app.ui.hotbar_index = i;
                app.ui.selected = None;
                app.ui.wire_from = None;
            }
            return;
        }
    }

    // Right-click on hotbar slot clears it.
    if is_mouse_button_pressed(MouseButton::Right) {
        if let Some(i) = point_in_hotbar(mouse.0, mouse.1) {
            app.ui.hotbar[i] = None;
            if app.ui.hotbar_index == i {
                app.ui.selected = None;
            }
            return;
        }
    }

    // World right-click → Blender-style context panel (when not on HUD).
    if !app.ui.build_open
        && is_mouse_button_pressed(MouseButton::Right)
        && !point_in_hud_chrome(mouse.0, mouse.1)
    {
        if app.ui.wire_from.take().is_some() {
            return;
        }
        if app.ui.selected.take().is_some() {
            // Right-click cancels active place tool first (clean feel).
            return;
        }
        let target = if let Some(id) = app.world.hit_node(wx, wy) {
            ContextTarget::Building(id)
        } else {
            ContextTarget::Empty
        };
        app.ui.context_menu = Some(ContextMenu {
            sx: mouse.0,
            sy: mouse.1,
            target,
        });
    }
}

fn context_items(target: ContextTarget) -> Vec<(&'static str, ContextAction)> {
    match target {
        ContextTarget::Empty => vec![
            ("New", ContextAction::OpenBuild),
            ("Clear tool", ContextAction::ClearTool),
        ],
        ContextTarget::Building(_) => vec![
            ("Delete", ContextAction::Delete),
            ("Rotate", ContextAction::Rotate),
            ("New", ContextAction::OpenBuild),
        ],
    }
}

#[derive(Clone, Copy)]
enum ContextAction {
    OpenBuild,
    ClearTool,
    Delete,
    Rotate,
}

fn context_menu_rect(menu: &ContextMenu) -> Rect {
    let n = context_items(menu.target).len() as f32;
    let w = 168.0;
    let h = 10.0 + n * 34.0;
    let mut x = menu.sx;
    let mut y = menu.sy;
    if x + w > screen_width() - 8.0 {
        x = screen_width() - w - 8.0;
    }
    if y + h > screen_height() - 8.0 {
        y = screen_height() - h - 8.0;
    }
    Rect { x, y, w, h }
}

fn handle_context_menu_input(app: &mut App, mouse: (f32, f32)) -> bool {
    let Some(menu) = app.ui.context_menu.clone() else {
        return false;
    };
    let r = context_menu_rect(&menu);
    let items = context_items(menu.target);

    if is_mouse_button_pressed(MouseButton::Left) {
        let inside = mouse.0 >= r.x
            && mouse.0 <= r.x + r.w
            && mouse.1 >= r.y
            && mouse.1 <= r.y + r.h;
        if !inside {
            app.ui.context_menu = None;
            return true;
        }
        for (i, (_, action)) in items.iter().enumerate() {
            let y = r.y + 6.0 + i as f32 * 34.0;
            if mouse.1 >= y && mouse.1 <= y + 30.0 {
                apply_context_action(app, menu.target, *action);
                app.ui.context_menu = None;
                return true;
            }
        }
        return true;
    }
    if is_mouse_button_pressed(MouseButton::Right) {
        app.ui.context_menu = None;
        return true;
    }
    true // swallow world input while open
}

fn apply_context_action(app: &mut App, target: ContextTarget, action: ContextAction) {
    match action {
        ContextAction::OpenBuild => app.ui.open_build(),
        ContextAction::ClearTool => app.ui.clear_tool(),
        ContextAction::Delete => {
            if let ContextTarget::Building(id) = target {
                remove_building(app, id);
            }
        }
        ContextAction::Rotate => {
            if let ContextTarget::Building(id) = target {
                if app.world.try_rotate_node(id) {
                    if let Some(n) = app.world.nodes.get(&id) {
                        if let Some(net) = app.net.as_ref() {
                            let _ = net.tx.send(NetCommand::Rotate {
                                id,
                                facing: n.facing,
                                request: !net.is_host,
                            });
                        }
                    }
                }
            }
        }
    }
}

fn handle_world_input(app: &mut App, mouse: (f32, f32), wx: f32, wy: f32) {
    if app.ui.panning || point_in_hud_chrome(mouse.0, mouse.1) {
        return;
    }

    // Click selected hotbar slot again to unequip (handled when press selects;
    // toggle here on press over nothing with same selection — skip).

    let port_r = PORT_HIT / app.cam.zoom;

    if is_mouse_button_pressed(MouseButton::Left) {
        // Finish equipping a hotbar drag as a click-select if released on same slot
        // is handled in hud; here: port wiring / place / move.
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
        } else {
            // Empty ground click unequips place tool.
            // (Keep selection if actively placing — Factorio keeps it; we toggle off with Q / RMB.)
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
                        request: !net.is_host,
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
    // Build menu under the hotbar so slots stay visible as drop targets.
    if app.ui.build_open {
        draw_and_handle_build_menu(app, mouse);
    }
    if let Some(overlay) = app.ui.overlay {
        draw_corner_overlay(overlay, mouse);
    }
    draw_hotbar(&app.ui, mouse);
    draw_tool_dock(app, mouse);
    if app.ui.context_menu.is_some() {
        draw_context_menu(&app.ui, mouse);
    }
    draw_drag_ghost(&app.ui, mouse);
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

fn draw_hotbar(ui: &Ui, mouse: (f32, f32)) {
    let (bar_x, bar_y, slot, gap) = hotbar_geom();
    let width = HOTBAR_SLOTS as f32 * slot + (HOTBAR_SLOTS - 1) as f32 * gap;
    let pad = s(10.0);

    // Soft floating capsule plate (no full-width bar).
    draw_rectangle(
        bar_x - pad,
        bar_y - pad,
        width + pad * 2.0,
        slot + pad * 2.0,
        Color::from_rgba(12, 14, 18, 170),
    );
    draw_rectangle_lines(
        bar_x - pad,
        bar_y - pad,
        width + pad * 2.0,
        slot + pad * 2.0,
        1.0,
        Color::from_rgba(80, 100, 120, 90),
    );

    for i in 0..HOTBAR_SLOTS {
        let x = bar_x + i as f32 * (slot + gap);
        let selected = i == ui.hotbar_index && ui.selected.is_some() && ui.hotbar[i] == ui.selected;
        let indexed = i == ui.hotbar_index;
        let hovered = mouse.0 >= x
            && mouse.0 <= x + slot
            && mouse.1 >= bar_y
            && mouse.1 <= bar_y + slot;
        let drop_target = ui.palette_drag.is_some() && hovered;

        draw_rectangle(
            x,
            bar_y,
            slot,
            slot,
            if drop_target {
                Color::from_rgba(40, 70, 60, 220)
            } else if hovered {
                Color::from_rgba(32, 40, 52, 220)
            } else {
                Color::from_rgba(20, 24, 30, 200)
            },
        );
        draw_rectangle_lines(
            x,
            bar_y,
            slot,
            slot,
            if selected || drop_target {
                2.2
            } else if indexed {
                1.6
            } else {
                1.0
            },
            if drop_target {
                CYAN
            } else if selected {
                ACCENT
            } else if indexed {
                Color::from_rgba(180, 150, 90, 180)
            } else {
                Color::from_rgba(70, 85, 100, 140)
            },
        );

        // Quiet key hint
        draw_text(
            &(i + 1).to_string(),
            x + s(5.0),
            bar_y + s(14.0),
            s(12.0),
            Color::from_rgba(140, 155, 170, 160),
        );

        if let Some(kind) = ui.hotbar[i] {
            let dim = ui.hotbar_drag_from == Some(i);
            let mut swatch = kind_swatch(kind);
            if dim {
                swatch.a = 0.35;
            }
            // Color chip centered
            let chip_w = slot - s(18.0);
            let chip_h = s(10.0);
            draw_rectangle(
                x + (slot - chip_w) * 0.5,
                bar_y + s(20.0),
                chip_w,
                chip_h,
                swatch,
            );
            let fs = s(13.0);
            let label = kind.short();
            let tw = measure_text(label, None, fs as u16, 1.0).width;
            draw_text(
                label,
                x + (slot - tw) * 0.5,
                bar_y + slot - s(8.0),
                fs,
                if dim {
                    TEXT_DIM
                } else {
                    TEXT
                },
            );
        }
    }
}

fn draw_tool_dock(app: &App, mouse: (f32, f32)) {
    // Slim floating rail behind the icons.
    let top = tool_button_rect(0);
    let bot = tool_button_rect(3);
    let rail_pad = s(8.0);
    draw_rectangle(
        top.x - rail_pad,
        top.y - rail_pad,
        top.w + rail_pad * 2.0,
        (bot.y + bot.h) - top.y + rail_pad * 2.0,
        Color::from_rgba(12, 14, 18, 160),
    );
    draw_rectangle_lines(
        top.x - rail_pad,
        top.y - rail_pad,
        top.w + rail_pad * 2.0,
        (bot.y + bot.h) - top.y + rail_pad * 2.0,
        1.0,
        Color::from_rgba(80, 100, 120, 80),
    );

    for (i, tool) in CornerTool::ALL.iter().enumerate() {
        let r = tool_button_rect(i);
        let active = match *tool {
            CornerTool::Build => app.ui.build_open,
            other => app.ui.overlay == Some(other),
        };
        let hovered = mouse.0 >= r.x
            && mouse.0 <= r.x + r.w
            && mouse.1 >= r.y
            && mouse.1 <= r.y + r.h;

        let cx = r.x + r.w * 0.5;
        let cy = r.y + r.h * 0.5;
        let radius = r.w * 0.46;

        draw_circle(
            cx,
            cy,
            radius,
            if active {
                Color::from_rgba(36, 58, 52, 240)
            } else if hovered {
                Color::from_rgba(34, 44, 58, 240)
            } else {
                Color::from_rgba(22, 26, 34, 220)
            },
        );
        draw_circle_lines(
            cx,
            cy,
            radius,
            if active || hovered { 2.0 } else { 1.2 },
            if active {
                CYAN
            } else if hovered {
                ACCENT
            } else {
                Color::from_rgba(90, 110, 130, 160)
            },
        );

        let accent = if active || hovered { CYAN } else { TEXT };
        match *tool {
            CornerTool::Build => {
                if let Some(tex) = app.icons.hammer.as_ref() {
                    let size = s(22.0);
                    draw_texture_ex(
                        tex,
                        cx - size * 0.5,
                        cy - size * 0.5,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(size, size)),
                            ..Default::default()
                        },
                    );
                } else {
                    draw_circle(cx, cy, s(4.0), accent);
                }
            }
            CornerTool::TechTree => draw_tech_icon(cx, cy, accent),
            CornerTool::Map => draw_map_icon(cx, cy, accent),
            CornerTool::NodeChart => draw_nodes_icon(cx, cy, accent),
        }

        // Hover / active label floats to the left — keeps chrome quiet.
        if hovered || active {
            let label = tool.label();
            let fs = s(14.0);
            let tw = measure_text(label, None, fs as u16, 1.0).width;
            let lx = r.x - s(14.0) - tw;
            let ly = cy + fs * 0.35;
            draw_rectangle(
                lx - s(8.0),
                cy - s(12.0),
                tw + s(16.0),
                s(24.0),
                Color::from_rgba(12, 14, 18, 200),
            );
            draw_text(label, lx, ly, fs, if active { CYAN } else { TEXT });
        }
    }
}

fn draw_corner_overlay(tool: CornerTool, _mouse: (f32, f32)) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(0, 0, 0, 120),
    );
    let w = 520.0;
    let h = 360.0;
    let x = (screen_width() - w) * 0.5;
    let y = (screen_height() - h) * 0.5 - 40.0;
    draw_rectangle(x, y, w, h, PANEL);
    draw_rectangle_lines(x, y, w, h, 1.5, NODE_BORDER);

    let title = match tool {
        CornerTool::TechTree => "Tech Tree",
        CornerTool::Map => "Map",
        CornerTool::NodeChart => "Node Chart",
        CornerTool::Build => "Build",
    };
    draw_text(title, x + 24.0, y + 40.0, 30.0, TEXT);
    draw_text(
        "Coming soon — placeholder panel",
        x + 24.0,
        y + 72.0,
        18.0,
        TEXT_DIM,
    );

    match tool {
        CornerTool::TechTree => {
            draw_tech_icon(x + w * 0.5, y + h * 0.55, CYAN);
            draw_text(
                "Unlock machines and logistics upgrades here.",
                x + 24.0,
                y + h - 36.0,
                16.0,
                TEXT_DIM,
            );
        }
        CornerTool::Map => {
            draw_map_icon(x + w * 0.5, y + h * 0.55, CYAN);
            draw_text(
                "World overview and remote navigation.",
                x + 24.0,
                y + h - 36.0,
                16.0,
                TEXT_DIM,
            );
        }
        CornerTool::NodeChart => {
            draw_nodes_icon(x + w * 0.5, y + h * 0.55, CYAN);
            draw_text(
                "Factory graph — belts, power, and throughput.",
                x + 24.0,
                y + h - 36.0,
                16.0,
                TEXT_DIM,
            );
        }
        CornerTool::Build => {}
    }

    // Click outside closes via handle_hud_input.
}

fn draw_drag_ghost(ui: &Ui, mouse: (f32, f32)) {
    let kind = if let Some(kind) = ui.palette_drag {
        let dx = mouse.0 - ui.palette_drag_origin.0;
        let dy = mouse.1 - ui.palette_drag_origin.1;
        if dx * dx + dy * dy > 36.0 {
            Some(kind)
        } else {
            None
        }
    } else if let Some(i) = ui.hotbar_drag_from {
        let dx = mouse.0 - ui.hotbar_drag_origin.0;
        let dy = mouse.1 - ui.hotbar_drag_origin.1;
        if dx * dx + dy * dy > 36.0 {
            ui.hotbar[i]
        } else {
            None
        }
    } else {
        None
    };
    let Some(kind) = kind else {
        return;
    };
    let size = 48.0;
    let x = mouse.0 - size * 0.5;
    let y = mouse.1 - size * 0.5;
    draw_rectangle(x, y, size, size, Color::from_rgba(20, 24, 30, 220));
    draw_rectangle_lines(x, y, size, size, 2.0, CYAN);
    draw_rectangle(x + 10.0, y + 10.0, size - 20.0, 12.0, kind_swatch(kind));
    draw_text(kind.short(), x + 6.0, y + 38.0, 14.0, TEXT);
}

fn draw_context_menu(ui: &Ui, mouse: (f32, f32)) {
    let Some(menu) = ui.context_menu.as_ref() else {
        return;
    };
    let r = context_menu_rect(menu);
    let items = context_items(menu.target);
    draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(18, 20, 26, 250));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.4, NODE_BORDER);
    for (i, (label, _)) in items.iter().enumerate() {
        let y = r.y + 6.0 + i as f32 * 34.0;
        let hovered = mouse.0 >= r.x
            && mouse.0 <= r.x + r.w
            && mouse.1 >= y
            && mouse.1 <= y + 30.0;
        if hovered {
            draw_rectangle(
                r.x + 4.0,
                y,
                r.w - 8.0,
                30.0,
                Color::from_rgba(48, 58, 72, 255),
            );
        }
        draw_text(label, r.x + 14.0, y + 21.0, 18.0, if hovered { CYAN } else { TEXT });
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
        "Drag onto the hotbar · click to equip · 1–9 pins to a slot",
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
        if hovered && is_mouse_button_pressed(MouseButton::Left) && app.ui.palette_drag.is_none() {
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
        let selected = app.ui.selected == Some(*kind) || app.ui.palette_drag == Some(*kind);
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
        draw_rectangle(row.x + 14.0, row.y + 16.0, 28.0, 20.0, kind_swatch(*kind));
        draw_text(kind.label(), row.x + 54.0, row.y + 32.0, 20.0, TEXT);
        if hovered
            && is_mouse_button_pressed(MouseButton::Left)
            && app.ui.palette_drag.is_none()
        {
            app.ui.palette_drag = Some(*kind);
            app.ui.palette_drag_origin = mouse;
            app.ui.context_menu = None;
        }
    }

    // Click dimmer outside panel closes menu (unless dragging).
    if app.ui.palette_drag.is_none()
        && is_mouse_button_pressed(MouseButton::Left)
        && !(mouse.0 >= r.x
            && mouse.0 <= r.x + r.w
            && mouse.1 >= r.y
            && mouse.1 <= r.y + r.h)
        && !point_in_hud_chrome(mouse.0, mouse.1)
    {
        app.ui.build_open = false;
    }
}
