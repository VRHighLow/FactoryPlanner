#![cfg_attr(windows, windows_subsystem = "windows")]

mod belts;
mod player;
mod sim;
mod net;
mod save;

use macroquad::prelude::*;
use macroquad::window::miniquad::*;
use net::{NetCommand, NetEvent, NetHandle};
use player::{CamMode, Player};
use save::{
    apply_save, capture_save, format_saved_at, list_saves, most_recent_save, read_save,
    write_autosave, write_manual_save, Settings, AUTOSAVE_INTERVAL_SECS,
};
use sim::*;
use std::collections::HashMap;
use std::time::Instant;

const MIN_ZOOM: f32 = 0.35;
const MAX_ZOOM: f32 = 2.5;
/// Must match `belts::TILE_SIZE` / `sim::TILE_SIZE`.
const GRID_MINOR: f32 = 40.0;
const GRID_MAJOR_EVERY: i32 = 10;
const PORT_HIT: f32 = 14.0;
const HOTBAR_SLOTS: usize = 9;
const TARGET_FPS: f64 = 120.0;
/// Fixed simulation rate (Factorio-style UPS). Render stays at TARGET_FPS.
const TARGET_UPS: f64 = 60.0;
const FIXED_DT: f32 = 1.0 / TARGET_UPS as f32;
/// Cap catch-up steps so a hitch doesn't spiral the sim.
const MAX_SIM_STEPS: u32 = 5;

const BG: Color = Color::from_rgba(22, 26, 32, 255);
const GRID_MINOR_C: Color = Color::from_rgba(48, 56, 68, 90);
const NODE_BORDER: Color = Color::from_rgba(120, 140, 160, 180);
const CYAN: Color = Color::from_rgba(64, 220, 210, 255);
const BELT_YELLOW: Color = Color::from_rgba(210, 170, 55, 255);
const POWER_C: Color = Color::from_rgba(255, 190, 70, 255);
const POWER_DIM: Color = Color::from_rgba(255, 190, 70, 90);
const TEXT: Color = Color::from_rgba(220, 230, 240, 255);
const TEXT_DIM: Color = Color::from_rgba(150, 160, 175, 255);
const PANEL: Color = Color::from_rgba(16, 18, 24, 240);
const ACCENT: Color = Color::from_rgba(255, 160, 60, 255);
const ORE_C: Color = Color::from_rgba(140, 140, 150, 255);
const INGOT_C: Color = Color::from_rgba(190, 200, 220, 255);

/// Starting clear pocket radius (world units). Totems expand further.
const STORM_SAFE_RADIUS: f32 = 2160.0;
const STORM_MAX_TOTEMS: usize = 8;
const STORM_MAX_FLASHES: usize = 4;
/// Hard build border sits inside the visual fog coast (~0.78 in the shader).
const STORM_HARD_CLEAR_SCALE: f32 = 0.72;
/// World-space radius of cloud illumination around a strike.
const STORM_FLASH_RADIUS: f32 = 520.0;

const STORM_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying vec2 uv;
varying vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0;
    uv = texcoord;
}
"#;

const STORM_FRAGMENT: &str = r#"#version 100
precision highp float;

varying vec2 uv;
varying vec4 color;

uniform vec2 ScreenSize;
uniform vec2 CamPos;
uniform float CamZoom;
uniform float Time;
uniform vec4 Totems[8];
uniform vec4 Flashes[4];

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < 5; i++) {
        v += a * noise(p);
        p *= 2.03;
        a *= 0.5;
    }
    return v;
}

float clear_zone(vec2 world, vec2 center, float radius, float mist, float fine) {
    if (radius < 1.0) {
        return 0.0;
    }
    float rr = length(world - center) / radius;
    float coast = 0.78 + (mist - 0.5) * 0.85 + (fine - 0.5) * 0.25;
    return 1.0 - smoothstep(coast - 0.32, coast + 0.48, rr);
}

void main() {
    vec2 screen = vec2(gl_FragCoord.x, ScreenSize.y - gl_FragCoord.y);
    vec2 world = (screen - 0.5 * ScreenSize) / max(CamZoom, 0.001) + CamPos;

    float t = Time * 0.028;
    vec2 q = world * 0.00155;
    vec2 warp = vec2(
        fbm(q + vec2(t, 0.0)),
        fbm(q + vec2(5.2, 1.3) - t)
    );
    vec2 p = q + (warp - 0.5) * 1.25;

    float lumps = fbm(p * 1.25);
    float detail = fbm(p * 3.4 + t * 0.45);
    float fine = fbm(p * 7.0 - t * 0.2);
    float mist = lumps * 0.55 + detail * 0.30 + fine * 0.15;

    float clear_amt = 0.0;
    for (int i = 0; i < 8; i++) {
        if (Totems[i].w > 0.5) {
            clear_amt = max(
                clear_amt,
                clear_zone(world, Totems[i].xy, Totems[i].z, mist, fine)
            );
        }
    }
    float outside = 1.0 - clear_amt;

    // Denser billows, fewer voids.
    float body = smoothstep(0.16, 0.52, mist);
    float holes = 1.0 - smoothstep(0.10, 0.32, detail) * 0.38;
    float density = clamp(outside * body * holes, 0.0, 1.0);

    vec3 col = mix(vec3(0.36, 0.34, 0.46), vec3(0.78, 0.80, 0.88), mist);
    col = mix(col, vec3(0.45, 0.52, 0.72), detail * 0.35);

    float rim = smoothstep(0.02, 0.20, density) * (1.0 - smoothstep(0.20, 0.50, density));
    col += vec3(0.40, 0.62, 0.88) * rim * 0.5;

    // Localized cloud flash — only lights fog near each strike (aerial storm view).
    float lit = 0.0;
    for (int i = 0; i < 4; i++) {
        float inten = Flashes[i].z;
        if (inten > 0.01) {
            vec2 d = world - Flashes[i].xy;
            float dist2 = dot(d, d);
            float rad = max(Flashes[i].w, 80.0);
            float fall = exp(-dist2 / (rad * rad));
            lit = max(lit, inten * fall);
        }
    }
    lit = clamp(lit, 0.0, 1.5) * density;
    float core_lit = lit * (0.55 + mist * 0.55);
    col = mix(col, vec3(0.78, 0.82, 0.95), core_lit * 0.55);
    col += vec3(0.65, 0.72, 1.0) * core_lit * 0.55;
    col += vec3(0.95, 0.97, 1.0) * lit * lit * 0.35;

    float alpha = density * (0.42 + 0.48 * mist) + outside * (0.10 + 0.10 * lumps);
    alpha = clamp(alpha + lit * 0.18, 0.0, 0.92);
    if (alpha < 0.015) {
        discard;
    }
    gl_FragColor = vec4(col, alpha);
}
"#;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Main,
    Play,
    Multiplayer,
    HostLobby,
    JoinLobby,
    Game,
    Settings,
    LoadGame,
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
    /// Mouse cursor (placement / aim).
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    /// Smoothed drone + local thruster FX.
    drone: player::RemoteDrone,
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
            ((wx - self.x) * self.zoom + screen_width() * 0.5).round(),
            ((wy - self.y) * self.zoom + screen_height() * 0.5).round(),
        )
    }

    fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        (
            (sx - screen_width() * 0.5) / self.zoom + self.x,
            (sy - screen_height() * 0.5) / self.zoom + self.y,
        )
    }

    /// Lock cam to the zoom lattice so grid lines / labels don't shimmer while following.
    fn quantize(&mut self) {
        let z = self.zoom.max(1e-4);
        self.x = (self.x * z).round() / z;
        self.y = (self.y * z).round() / z;
    }
}

/// Visual storm border — clear pocket at origin; fog outside. Gameplay later.
struct Storm {
    cx: f32,
    cy: f32,
    radius: f32,
    time: f32,
    /// Local cloud flashes: (x, y, intensity, radius).
    flashes: [(f32, f32, f32, f32); STORM_MAX_FLASHES],
    material: Option<Material>,
}

impl Storm {
    fn new(material: Option<Material>) -> Self {
        Self {
            cx: 0.0,
            cy: 0.0,
            radius: STORM_SAFE_RADIUS,
            time: 0.0,
            flashes: [(0.0, 0.0, 0.0, 0.0); STORM_MAX_FLASHES],
            material,
        }
    }

    fn tick(&mut self, dt: f32) {
        self.time += dt;
        for slot in &mut self.flashes {
            if slot.2 <= 0.0 {
                continue;
            }
            slot.2 = (slot.2 - dt * 4.5).max(0.0);
            if slot.2 > 0.15 && storm_hash01(self.time * 40.0 + slot.0 * 0.01) > 0.82 {
                slot.2 = (slot.2 * 0.55).max(0.08);
            }
            if slot.2 <= 0.01 {
                *slot = (0.0, 0.0, 0.0, 0.0);
            }
        }
    }

    fn trigger_flash(&mut self, x: f32, y: f32, intensity: f32) {
        let intensity = intensity.clamp(0.2, 1.5);
        // Reinforce a nearby existing flash, else take weakest/empty slot.
        let mut best_near = None;
        let mut best_near_d2 = (STORM_FLASH_RADIUS * 0.45).powi(2);
        let mut weakest = 0usize;
        let mut weakest_i = f32::MAX;
        for (i, slot) in self.flashes.iter().enumerate() {
            if slot.2 < weakest_i {
                weakest_i = slot.2;
                weakest = i;
            }
            if slot.2 > 0.01 {
                let dx = slot.0 - x;
                let dy = slot.1 - y;
                let d2 = dx * dx + dy * dy;
                if d2 < best_near_d2 {
                    best_near_d2 = d2;
                    best_near = Some(i);
                }
            }
        }
        let idx = best_near.unwrap_or(weakest);
        let cur = self.flashes[idx].2;
        self.flashes[idx] = (x, y, cur.max(intensity), STORM_FLASH_RADIUS);
    }

    /// Clear zones: base pocket + powered totems. Each is (cx, cy, radius).
    fn clear_zones(&self, world: &World) -> Vec<(f32, f32, f32)> {
        let mut zones = vec![(self.cx, self.cy, self.radius)];
        for n in world.nodes.values() {
            if n.kind == BuildingKind::Totem && n.powered {
                let (cx, cy) = n.center();
                zones.push((cx, cy, TOTEM_CLEAR_RADIUS));
            }
        }
        zones
    }

    /// Hard gameplay clear check — stable circles, ignores animated fog.
    fn in_clear(&self, wx: f32, wy: f32, zones: &[(f32, f32, f32)]) -> bool {
        zones.iter().any(|&(cx, cy, radius)| {
            let r = radius * STORM_HARD_CLEAR_SCALE;
            if r < 1.0 {
                return false;
            }
            let dx = wx - cx;
            let dy = wy - cy;
            dx * dx + dy * dy <= r * r
        })
    }

    fn point_in_storm(&self, wx: f32, wy: f32, world: &World) -> bool {
        let zones = self.clear_zones(world);
        !self.in_clear(wx, wy, &zones)
    }

    /// 0 = clear, 1 = deep storm (visual / CPU fog only — not for build rules).
    fn coverage_at(&self, wx: f32, wy: f32, zones: &[(f32, f32, f32)]) -> f32 {
        let t = self.time * 0.028;
        let q = (wx * 0.00155, wy * 0.00155);
        let warp_x = storm_fbm(q.0 + t, q.1) - 0.5;
        let warp_y = storm_fbm(q.0 + 5.2, q.1 - t) - 0.5;
        let px = q.0 + warp_x * 1.25;
        let py = q.1 + warp_y * 1.25;
        let lumps = storm_fbm(px * 1.25, py * 1.25);
        let detail = storm_fbm(px * 3.4 + t * 0.45, py * 3.4);
        let fine = storm_fbm(px * 7.0 - t * 0.2, py * 7.0);
        let mist = lumps * 0.55 + detail * 0.30 + fine * 0.15;

        let mut clear_amt = 0.0_f32;
        for &(cx, cy, radius) in zones {
            if radius < 1.0 {
                continue;
            }
            let dx = (wx - cx) / radius;
            let dy = (wy - cy) / radius;
            let rr = (dx * dx + dy * dy).sqrt();
            let coast = 0.78 + (mist - 0.5) * 0.85 + (fine - 0.5) * 0.25;
            let inside = 1.0 - smoothstep(coast - 0.32, coast + 0.48, rr);
            clear_amt = clear_amt.max(inside);
        }
        let outside = 1.0 - clear_amt;
        let body = smoothstep(0.16, 0.52, mist);
        let holes = 1.0 - smoothstep(0.10, 0.32, detail) * 0.38;
        (outside * body * holes).clamp(0.0, 1.0)
    }
}

fn storm_hash01(seed: f32) -> f32 {
    let x = (seed * 12.9898).sin() * 43758.5453;
    x.fract().abs()
}

fn storm_hash(x: i32, y: i32) -> f32 {
    let mut n = x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    (n & 0x7fff_ffff) as f32 / 0x7fff_ffff as f32
}

fn storm_noise(x: f32, y: f32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let fx = x - x0 as f32;
    let fy = y - y0 as f32;
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let a = storm_hash(x0, y0);
    let b = storm_hash(x0 + 1, y0);
    let c = storm_hash(x0, y0 + 1);
    let d = storm_hash(x0 + 1, y0 + 1);
    a * (1.0 - ux) * (1.0 - uy)
        + b * ux * (1.0 - uy)
        + c * (1.0 - ux) * uy
        + d * ux * uy
}

fn storm_fbm(mut x: f32, mut y: f32) -> f32 {
    let mut v = 0.0;
    let mut a = 0.5;
    for _ in 0..5 {
        v += a * storm_noise(x, y);
        x *= 2.03;
        y *= 2.03;
        a *= 0.5;
    }
    v
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn create_storm_material() -> Option<Material> {
    let pipeline_params = PipelineParams {
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };
    load_material(
        ShaderSource::Glsl {
            vertex: STORM_VERTEX,
            fragment: STORM_FRAGMENT,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("ScreenSize", UniformType::Float2),
                UniformDesc::new("CamPos", UniformType::Float2),
                UniformDesc::new("CamZoom", UniformType::Float1),
                UniformDesc::new("Time", UniformType::Float1),
                UniformDesc::array(
                    UniformDesc::new("Totems", UniformType::Float4),
                    STORM_MAX_TOTEMS,
                ),
                UniformDesc::array(
                    UniformDesc::new("Flashes", UniformType::Float4),
                    STORM_MAX_FLASHES,
                ),
            ],
            pipeline_params,
            ..Default::default()
        },
    )
    .map_err(|e| {
        eprintln!("storm nebula shader unavailable, using CPU fog: {e:?}");
        e
    })
    .ok()
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
    /// `None` = All categories.
    build_category: Option<BuildCategory>,
    build_search: String,
    build_search_focus: bool,
    build_scroll: f32,
    /// Drain the open-key char so it doesn't land in search.
    suppress_search_chars: bool,
    selected: Option<BuildingKind>,
    hotbar: [Option<BuildingKind>; HOTBAR_SLOTS],
    hotbar_index: usize,
    place_facing: Facing,
    wire_from: Option<(u32, usize)>,
    /// Last tile painted while drag-placing belts (avoids re-paint spam).
    belt_paint_last: Option<(i32, i32)>,
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
            build_category: None,
            build_search: String::new(),
            build_search_focus: true,
            build_scroll: 0.0,
            suppress_search_chars: false,
            selected: None,
            hotbar: [None; HOTBAR_SLOTS],
            hotbar_index: 0,
            place_facing: Facing::E,
            wire_from: None,
            belt_paint_last: None,
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
        self.belt_paint_last = None;
        self.palette_drag = None;
        self.hotbar_drag_from = None;
    }

    fn open_build(&mut self) {
        self.build_open = true;
        self.overlay = None;
        self.wire_from = None;
        self.context_menu = None;
        self.drag_node = None;
        self.build_search_focus = true;
        self.build_scroll = 0.0;
        self.suppress_search_chars = true;
    }

    fn close_build(&mut self) {
        self.build_open = false;
        self.palette_drag = None;
        self.build_search_focus = false;
    }

    fn toggle_build(&mut self) {
        if self.build_open {
            self.close_build();
        } else {
            self.open_build();
        }
    }

    fn activate_corner(&mut self, tool: CornerTool) {
        match tool {
            CornerTool::Build => self.toggle_build(),
            other => {
                self.close_build();
                if self.overlay == Some(other) {
                    self.overlay = None;
                } else {
                    self.overlay = Some(other);
                }
                self.context_menu = None;
            }
        }
    }

    fn filtered_buildings(&self) -> Vec<BuildingKind> {
        let base: Vec<BuildingKind> = match self.build_category {
            Some(cat) => BuildingKind::in_category(cat),
            None => BuildingKind::all().to_vec(),
        };
        base.into_iter()
            .filter(|k| k.matches_query(&self.build_search))
            .collect()
    }
}

struct LightningFx {
    /// Main bolt polyline in world space.
    points: Vec<(f32, f32)>,
    /// Optional side branches.
    branches: Vec<Vec<(f32, f32)>>,
    life: f32,
    max_life: f32,
    width: f32,
}

struct App {
    screen: Screen,
    /// Where Settings returns to when closed.
    settings_return: Screen,
    world: World,
    cam: Cam,
    ui: Ui,
    icons: Icons,
    storm: Storm,
    settings: Settings,
    pause_open: bool,
    autosave_timer: f32,
    /// Accumulator for fixed 60 UPS world steps.
    sim_accum: f32,
    lightning_cd: f32,
    lightning_fx: Vec<LightningFx>,
    status_toast: String,
    load_scroll: f32,
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
    player: Player,
}

impl App {
    fn new(icons: Icons) -> Self {
        let settings = Settings::load();
        Self {
            screen: Screen::Main,
            settings_return: Screen::Main,
            world: World::new(),
            cam: Cam {
                x: 0.0,
                y: 0.0,
                zoom: 1.0,
            },
            ui: Ui::new(),
            icons,
            storm: Storm::new(None),
            settings,
            pause_open: false,
            autosave_timer: 0.0,
            sim_accum: 0.0,
            lightning_cd: 1.5,
            lightning_fx: Vec::new(),
            status_toast: String::new(),
            load_scroll: 0.0,
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
            player: Player::new(0.0, 0.0),
        }
    }

    fn is_single_player(&self) -> bool {
        self.net.is_none()
    }

    fn open_settings(&mut self, from: Screen) {
        self.settings_return = from;
        self.screen = Screen::Settings;
    }

    fn enter_game_common(&mut self) {
        self.screen = Screen::Game;
        self.pause_open = false;
        self.ui = Ui::new();
        self.peers.clear();
        self.autosave_timer = 0.0;
        self.sim_accum = 0.0;
        if let Some(net) = self.net.as_ref() {
            let _ = net.tx.send(NetCommand::Announce);
            if !net.is_host {
                let _ = net.tx.send(NetCommand::WantSnap);
            }
        }
    }

    fn enter_new_singleplayer(&mut self) {
        self.stop_net();
        self.world.clear();
        self.world
            .seed_nests(self.storm.cx, self.storm.cy, self.storm.radius);
        self.cam = Cam {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        };
        self.player = Player::new(0.0, 0.0);
        self.enter_game_common();
    }

    fn enter_from_save(&mut self, save: &save::GameSave) -> Result<(), String> {
        self.stop_net();
        apply_save(&mut self.world, save)?;
        if self.world.nests.is_empty() {
            self.world
                .seed_nests(self.storm.cx, self.storm.cy, self.storm.radius);
        }
        self.cam = Cam {
            x: save.cam_x,
            y: save.cam_y,
            zoom: save.cam_zoom.clamp(MIN_ZOOM, MAX_ZOOM),
        };
        self.player = Player::new(save.cam_x, save.cam_y);
        self.enter_game_common();
        for (i, k) in save.hotbar.iter().enumerate() {
            self.ui.hotbar[i] = k.and_then(|v| BuildingKind::from_u8(v));
        }
        self.ui.hotbar_index = save.hotbar_index.min(HOTBAR_SLOTS - 1);
        self.ui.selected = self.ui.hotbar[self.ui.hotbar_index];
        Ok(())
    }

    fn capture_current_save(&self, label: &str) -> save::GameSave {
        capture_save(
            &self.world,
            self.cam.x,
            self.cam.y,
            self.cam.zoom,
            &self.ui.hotbar,
            self.ui.hotbar_index,
            label,
        )
    }

    fn do_manual_save(&mut self) {
        let save = self.capture_current_save("Manual Save");
        match write_manual_save(&save) {
            Ok(_) => self.status_toast = "Game saved".into(),
            Err(e) => self.status_toast = format!("Save failed: {e}"),
        }
    }

    fn do_autosave(&mut self) {
        if !self.is_single_player() {
            return;
        }
        let mut save = self.capture_current_save("Autosave");
        match write_autosave(&mut self.settings, &mut save) {
            Ok(_) => self.status_toast = format!("Autosaved ({})", save.label),
            Err(e) => self.status_toast = format!("Autosave failed: {e}"),
        }
    }

    fn return_to_main_menu(&mut self) {
        self.stop_net();
        self.pause_open = false;
        self.ui = Ui::new();
        self.world.clear();
        self.lightning_fx.clear();
        self.lightning_cd = 1.5;
        self.screen = Screen::Main;
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
    let settings = Settings::load();
    Conf {
        window_title: "FactoryPlanner".to_owned(),
        window_width: settings.window_w,
        window_height: settings.window_h,
        fullscreen: !settings.display_mode.is_windowed(),
        high_dpi: false,
        platform: miniquad::conf::Platform {
            swap_interval: Some(if settings.vsync { 1 } else { 0 }),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let icons = Icons::load().await;
    let mut app = App::new(icons);
    app.storm.material = create_storm_material();
    app.settings.apply_runtime();
    let frame_budget = std::time::Duration::from_secs_f64(1.0 / TARGET_FPS);

    loop {
        let frame_start = Instant::now();
        let frame_dt = get_frame_time().clamp(0.0, 0.05);
        let mouse = mouse_position();

        match app.screen {
            Screen::Main => screen_main(&mut app, mouse, frame_dt),
            Screen::Play => screen_play(&mut app, mouse, frame_dt),
            Screen::Multiplayer => screen_multiplayer(&mut app, mouse, frame_dt),
            Screen::HostLobby => screen_host_lobby(&mut app, mouse, frame_dt),
            Screen::JoinLobby => screen_join_lobby(&mut app, mouse, frame_dt),
            Screen::Settings => screen_settings(&mut app, mouse, frame_dt),
            Screen::LoadGame => screen_load_game(&mut app, mouse, frame_dt),
            Screen::Game => {
                drain_net(&mut app);
                maybe_host_snapshot(&mut app);
                let (wx, wy) = app.cam.screen_to_world(mouse.0, mouse.1);
                handle_hotkeys(&mut app, wx, wy);
                if !app.pause_open {
                    handle_build_search_input(&mut app.ui);
                }
                if !app.pause_open {
                    handle_pan_zoom(&mut app, mouse);
                    handle_hud_input(&mut app, mouse, wx, wy);
                    if !app.ui.build_open
                        && app.ui.context_menu.is_none()
                        && app.ui.overlay.is_none()
                    {
                        handle_world_input(&mut app, mouse, wx, wy);
                    }
                    send_cursor_if_due(&mut app, wx, wy);
                    // Visual interpolation at render rate.
                    advance_peer_cursors(&mut app, frame_dt);
                    app.storm.tick(frame_dt);
                    // Fixed 60 UPS simulation.
                    app.sim_accum += frame_dt;
                    let mut steps = 0u32;
                    while app.sim_accum >= FIXED_DT && steps < MAX_SIM_STEPS {
                        app.sim_accum -= FIXED_DT;
                        let wish = if app.ui.build_open
                            || app.ui.overlay.is_some()
                            || app.ui.context_menu.is_some()
                        {
                            // Don't walk the mech while typing / in menus.
                            Vec2::ZERO
                        } else {
                            player::movement_wish()
                        };
                        app.player.tick(FIXED_DT, wish);
                        app.world.tick(FIXED_DT);
                        let zones = app.storm.clear_zones(&app.world);
                        let report = app.world.combat_step(FIXED_DT, &zones);
                        if report.nests_reawakened > 0 {
                            app.status_toast = "Hateful nest reawakened — bigger swarm!".into();
                        } else if report.nests_revealed > 0 {
                            app.status_toast = if report.nests_revealed == 1 {
                                "Nest revealed! Swarm incoming — fortify!".into()
                            } else {
                                format!(
                                    "{} nests revealed! Swarms incoming!",
                                    report.nests_revealed
                                )
                            };
                        } else if report.waves_launched > 0 {
                            app.status_toast = "Enemy wave launched!".into();
                        } else if report.destroyed > 0 {
                            app.status_toast = if report.destroyed == 1 {
                                "Raiders destroyed a building!".into()
                            } else {
                                format!("Raiders destroyed {} buildings!", report.destroyed)
                            };
                        }
                        tick_storm_lightning(&mut app, FIXED_DT);
                        steps += 1;
                    }
                    // Camera follow / leash after sim (and after MMB pan).
                    match app.player.cam_mode {
                        CamMode::Follow => {
                            player::snap_cam_to_player(
                                &mut app.cam.x,
                                &mut app.cam.y,
                                app.player.x,
                                app.player.y,
                            );
                        }
                        CamMode::Free => {
                            player::clamp_cam_to_player(
                                &mut app.cam.x,
                                &mut app.cam.y,
                                app.cam.zoom,
                                app.player.x,
                                app.player.y,
                            );
                        }
                    }
                    app.cam.quantize();
                    if steps >= MAX_SIM_STEPS {
                        app.sim_accum = 0.0;
                    }
                    if app.is_single_player() {
                        app.autosave_timer += frame_dt;
                        if app.autosave_timer >= AUTOSAVE_INTERVAL_SECS {
                            app.autosave_timer = 0.0;
                            app.do_autosave();
                        }
                    }
                } else {
                    app.storm.tick(frame_dt);
                    app.cam.quantize();
                }
                // Refresh mouse world after cam lock/quantize so HUD text doesn't shimmer.
                let (wx, wy) = app.cam.screen_to_world(mouse.0, mouse.1);
                // Lightning bolts animate at render rate.
                tick_lightning_fx(&mut app, frame_dt);
                draw_game(&mut app, mouse, wx, wy);
                if app.pause_open {
                    draw_and_handle_pause_menu(&mut app, mouse);
                }
                if !app.status_toast.is_empty() {
                    draw_text(&app.status_toast, 16.0, screen_height() - 28.0, 18.0, ACCENT);
                }
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

fn menu_panel_geom(btn_count: usize) -> (f32, f32, f32, f32, f32) {
    let bw = 320.0;
    let bh = 48.0;
    let gap = 14.0;
    let title_block = 110.0;
    let total_h = title_block + btn_count as f32 * (bh + gap);
    let bx = (screen_width() - bw) * 0.5;
    let top = ((screen_height() - total_h) * 0.5).max(40.0);
    (bx, top, bw, bh, gap)
}

fn draw_menu_storm_backdrop(app: &mut App, dt: f32, title: &str, subtitle: &str) {
    app.storm.tick(dt);
    let cam = Cam {
        x: 80.0,
        y: -40.0,
        zoom: 0.28,
    };
    clear_background(BG);
    draw_infinite_grid(&cam);
    draw_storm(&app.storm, &World::new(), &cam);
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(8, 10, 16, 110),
    );
    let tw = measure_text(title, None, 56, 1.0).width;
    let sx = (screen_width() - tw) * 0.5;
    let (_, top, _, _, _) = menu_panel_geom(5);
    draw_text(title, sx, top + 48.0, 56.0, CYAN);
    let sw = measure_text(subtitle, None, 20, 1.0).width;
    draw_text(
        subtitle,
        (screen_width() - sw) * 0.5,
        top + 82.0,
        20.0,
        TEXT_DIM,
    );
}

fn screen_main(app: &mut App, mouse: (f32, f32), dt: f32) {
    let has_continue = most_recent_save().is_some();
    let buttons = if has_continue { 5 } else { 4 };
    draw_menu_storm_backdrop(app, dt, "FactoryPlanner", "Plan. Place. Power.");
    let (bx, top, bw, bh, gap) = menu_panel_geom(buttons);
    let mut by = top + 110.0;

    if has_continue {
        if button("Continue", bx, by, bw, bh, mouse) {
            if let Some(info) = most_recent_save() {
                match read_save(&info.path) {
                    Ok(save) => {
                        if let Err(e) = app.enter_from_save(&save) {
                            app.status_toast = e;
                        }
                    }
                    Err(e) => app.status_toast = e,
                }
            }
        }
        by += bh + gap;
    }

    if button("New Game", bx, by, bw, bh, mouse) {
        app.screen = Screen::Play;
    }
    by += bh + gap;
    if button("Load Game", bx, by, bw, bh, mouse) {
        app.load_scroll = 0.0;
        app.screen = Screen::LoadGame;
    }
    by += bh + gap;
    if button("Settings", bx, by, bw, bh, mouse) {
        app.open_settings(Screen::Main);
    }
    by += bh + gap;
    if button("Exit Game", bx, by, bw, bh, mouse) {
        std::process::exit(0);
    }
}

fn screen_play(app: &mut App, mouse: (f32, f32), dt: f32) {
    draw_menu_storm_backdrop(app, dt, "New Game", "Choose a mode");
    let (bx, top, bw, bh, gap) = menu_panel_geom(3);
    let mut by = top + 110.0;
    if button("Single Player", bx, by, bw, bh, mouse) {
        app.enter_new_singleplayer();
    }
    by += bh + gap;
    if button("Multiplayer", bx, by, bw, bh, mouse) {
        app.screen = Screen::Multiplayer;
    }
    by += bh + gap;
    if button("Back", bx, by, bw, bh, mouse) {
        app.screen = Screen::Main;
    }
}

fn screen_multiplayer(app: &mut App, mouse: (f32, f32), dt: f32) {
    draw_menu_storm_backdrop(
        app,
        dt,
        "Multiplayer",
        "P2P (iroh) — play across the world with a code",
    );
    let (bx, top, bw, bh, gap) = menu_panel_geom(3);
    let mut by = top + 110.0;
    if button("Host Game", bx, by, bw, bh, mouse) {
        app.stop_net();
        app.world.clear();
        app.world
            .seed_nests(app.storm.cx, app.storm.cy, app.storm.radius);
        app.join_status = "Code reserved — finishing online setup…".into();
        let handle = net::start_host();
        app.host_code = handle.code.clone();
        app.host_addr = "starting…".into();
        app.net = Some(handle);
        app.screen = Screen::HostLobby;
    }
    by += bh + gap;
    if button("Join Game", bx, by, bw, bh, mouse) {
        app.stop_net();
        app.join_status.clear();
        app.join_code.clear();
        app.join_focus = true;
        app.screen = Screen::JoinLobby;
    }
    by += bh + gap;
    if button("Back", bx, by, bw, bh, mouse) {
        app.screen = Screen::Play;
    }
}

fn screen_settings(app: &mut App, mouse: (f32, f32), dt: f32) {
    draw_menu_storm_backdrop(app, dt, "Settings", "Display & performance");
    let (bx, top, bw, bh, gap) = menu_panel_geom(7);
    let mut by = top + 110.0;

    let mode_label = format!("Display: {}", app.settings.display_mode.label());
    if button(&mode_label, bx, by, bw, bh, mouse) {
        app.settings.display_mode = app.settings.display_mode.next();
    }
    by += bh + gap;

    let vsync_label = format!(
        "VSync: {} (restart)",
        if app.settings.vsync { "On" } else { "Off" }
    );
    if button(&vsync_label, bx, by, bw, bh, mouse) {
        app.settings.vsync = !app.settings.vsync;
    }
    by += bh + gap;

    let windowed = app.settings.display_mode.is_windowed();
    let res_label = if windowed {
        format!("Resolution: {}×{}", app.settings.window_w, app.settings.window_h)
    } else {
        "Resolution: (windowed only)".into()
    };
    if windowed && button(&res_label, bx, by, bw, bh, mouse) {
        let presets = [(1280, 720), (1400, 900), (1600, 900), (1920, 1080)];
        let idx = presets
            .iter()
            .position(|&(w, h)| w == app.settings.window_w && h == app.settings.window_h)
            .unwrap_or(1);
        let (w, h) = presets[(idx + 1) % presets.len()];
        app.settings.window_w = w;
        app.settings.window_h = h;
    } else if !windowed {
        // Draw disabled-looking row without handling clicks.
        draw_rectangle(bx, by, bw, bh, Color::from_rgba(22, 26, 32, 255));
        draw_rectangle_lines(bx, by, bw, bh, 1.0, NODE_BORDER);
        let tw = measure_text(&res_label, None, 22, 1.0).width;
        draw_text(
            &res_label,
            bx + (bw - tw) * 0.5,
            by + bh * 0.5 + 7.0,
            22.0,
            TEXT_DIM,
        );
    }
    by += bh + gap;

    let fps_label = format!(
        "Show FPS: {}",
        if app.settings.show_fps { "On" } else { "Off" }
    );
    if button(&fps_label, bx, by, bw, bh, mouse) {
        app.settings.show_fps = !app.settings.show_fps;
    }
    by += bh + gap;

    if button("Apply", bx, by, bw, bh, mouse) {
        if let Err(e) = app.settings.save() {
            app.status_toast = e;
        } else {
            app.settings.apply_runtime();
            app.status_toast = "Settings applied".into();
        }
    }
    by += bh + gap;

    if button("Back", bx, by, bw, bh, mouse) {
        let _ = app.settings.save();
        if app.settings_return == Screen::Game {
            app.screen = Screen::Game;
            app.pause_open = true;
        } else {
            app.screen = app.settings_return;
        }
    }
}

fn screen_load_game(app: &mut App, mouse: (f32, f32), dt: f32) {
    draw_menu_storm_backdrop(app, dt, "Load Game", "Autosaves and manual saves");
    let saves = list_saves();
    let (bx, top, bw, bh, gap) = menu_panel_geom(1.max(saves.len().min(6) + 1));
    let mut by = top + 110.0;

    if saves.is_empty() {
        let msg = "No saves found";
        let tw = measure_text(msg, None, 20, 1.0).width;
        draw_text(
            msg,
            (screen_width() - tw) * 0.5,
            by + 20.0,
            20.0,
            TEXT_DIM,
        );
        by += 64.0;
    } else {
        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            app.load_scroll = (app.load_scroll - wheel).max(0.0);
        }
        let start = app.load_scroll as usize;
        for info in saves.iter().skip(start).take(6) {
            let label = format!("{} — {}", info.label, format_saved_at(info.saved_at));
            if button(&label, bx, by, bw, bh, mouse) {
                match read_save(&info.path) {
                    Ok(save) => {
                        if let Err(e) = app.enter_from_save(&save) {
                            app.status_toast = e;
                        }
                    }
                    Err(e) => app.status_toast = e,
                }
            }
            by += bh + gap;
        }
    }

    if button("Back", bx, by, bw, bh, mouse) {
        if app.settings_return == Screen::Game || app.pause_open {
            app.screen = Screen::Game;
            app.pause_open = true;
        } else {
            app.screen = Screen::Main;
        }
    }
}

fn pause_menu_rect() -> Rect {
    let w = 360.0;
    let h = 360.0;
    Rect {
        x: (screen_width() - w) * 0.5,
        y: (screen_height() - h) * 0.5,
        w,
        h,
    }
}

fn draw_and_handle_pause_menu(app: &mut App, mouse: (f32, f32)) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(0, 0, 0, 160),
    );
    let r = pause_menu_rect();
    draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, NODE_BORDER);
    let title = "Paused";
    let tw = measure_text(title, None, 32, 1.0).width;
    draw_text(title, r.x + (r.w - tw) * 0.5, r.y + 42.0, 32.0, TEXT);

    let bw = r.w - 48.0;
    let bh = 44.0;
    let bx = r.x + 24.0;
    let mut by = r.y + 70.0;
    let gap = 12.0;

    if button("Resume", bx, by, bw, bh, mouse) {
        app.pause_open = false;
        return;
    }
    by += bh + gap;
    if app.is_single_player() {
        if button("Save Game", bx, by, bw, bh, mouse) {
            app.do_manual_save();
        }
        by += bh + gap;
        if button("Load Game", bx, by, bw, bh, mouse) {
            app.load_scroll = 0.0;
            app.settings_return = Screen::Game;
            app.screen = Screen::LoadGame;
            return;
        }
        by += bh + gap;
    }
    if button("Settings", bx, by, bw, bh, mouse) {
        app.open_settings(Screen::Game);
        return;
    }
    by += bh + gap;
    if button("Main Menu", bx, by, bw, bh, mouse) {
        app.return_to_main_menu();
    }
}

fn screen_host_lobby(app: &mut App, mouse: (f32, f32), dt: f32) {
    drain_net(app);
    draw_menu_storm_backdrop(
        app,
        dt,
        "Host",
        "Share your code — setup continues while you play",
    );
    let (bx, top, bw, bh, gap) = menu_panel_geom(4);
    let mut by = top + 110.0;
    draw_text("Your session code", bx, by - 8.0, 18.0, TEXT_DIM);
    draw_text(
        &if app.host_code.is_empty() {
            "……".into()
        } else {
            app.host_code.clone()
        },
        bx,
        by + 36.0,
        48.0,
        CYAN,
    );
    by += 70.0;
    if !app.host_addr.is_empty() {
        draw_text(&format!("Transport: {}", app.host_addr), bx, by, 16.0, TEXT_DIM);
        by += 24.0;
    }
    if !app.join_status.is_empty() {
        draw_text(&app.join_status, bx, by, 18.0, ACCENT);
        by += 28.0;
    }
    by += 12.0;
    if button("Enter World", bx, by, bw, bh, mouse) {
        if app.net.is_some() {
            app.cam = Cam {
                x: 0.0,
                y: 0.0,
                zoom: 1.0,
            };
            app.player = Player::new(0.0, 0.0);
            app.enter_game_common();
        }
    }
    by += bh + gap;
    if button("Back", bx, by, bw, bh, mouse) {
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

fn screen_join_lobby(app: &mut App, mouse: (f32, f32), dt: f32) {
    drain_net(app);
    draw_menu_storm_backdrop(app, dt, "Join Game", "Code only — P2P works worldwide");
    let (bx, top, bw, bh, gap) = menu_panel_geom(3);
    let mut by = top + 110.0;

    if text_field(
        "Session code",
        &app.join_code,
        bx,
        by,
        bw,
        56.0,
        app.join_focus,
        mouse,
    ) {
        app.join_focus = true;
    }
    if app.join_focus {
        handle_text_input(&mut app.join_code);
    }
    by += 72.0;

    if !app.join_status.is_empty() {
        draw_text(&app.join_status, bx, by, 18.0, ACCENT);
        by += 28.0;
    }

    if button("Connect", bx, by, bw, bh, mouse) {
        app.stop_net();
        app.world.clear();
        app.join_status = "Connecting online…".into();
        let handle = net::start_client("", &app.join_code);
        app.net = Some(handle);
    }
    by += bh + gap;
    if button("Back", bx, by, bw, bh, mouse) {
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
    // Belt tiles: multiplayer sync TBD — topology-only power links for now.
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

fn peer_label(id: u8) -> String {
    format!("Player {}", id + 1)
}

fn advance_peer_cursors(app: &mut App, dt: f32) {
    for peer in app.peers.values_mut() {
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
        peer.drone.tick(dt);
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
                    app.cam = Cam {
                        x: 0.0,
                        y: 0.0,
                        zoom: 1.0,
                    };
                    app.player = Player::new(0.0, 0.0);
                    app.enter_game_common();
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
                    if app.world.try_move_node(id, x, y) {
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
                    app.world.belt_tiles.clear();
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
                dx,
                dy,
                dfacing,
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
                    peer.x = peer.x * 0.25 + x * 0.75;
                    peer.y = peer.y * 0.25 + y * 0.75;
                    peer.drone.set_target(dx, dy, dfacing);
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
                            drone: player::RemoteDrone::new(dx, dy, dfacing),
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
    // Cap at ~20 Hz — flooding every frame made remote drones jitter.
    if app.last_cursor_send.elapsed().as_millis() < 50 {
        return;
    }
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
        dx: app.player.x,
        dy: app.player.y,
        dfacing: app.player.facing,
    });
}

fn handle_hotkeys(app: &mut App, wx: f32, wy: f32) {
    if app.pause_open {
        if is_key_pressed(KeyCode::Escape) {
            app.pause_open = false;
        }
        return;
    }

    // While the build menu is open, letter keys feed search — don't toggle/rotate/clear.
    if !app.ui.build_open {
        if is_key_pressed(KeyCode::B) {
            app.ui.toggle_build();
        }
        if is_key_pressed(KeyCode::C) {
            app.player.cam_mode = app.player.cam_mode.toggle();
            if app.player.cam_mode == CamMode::Follow {
                player::snap_cam_to_player(
                    &mut app.cam.x,
                    &mut app.cam.y,
                    app.player.x,
                    app.player.y,
                );
            }
            app.status_toast = format!("{}  (C to toggle)", app.player.cam_mode.label());
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
    } else if is_key_pressed(KeyCode::B) && app.ui.build_search.is_empty() {
        // B still closes when search is empty (no conflict with typing).
        app.ui.close_build();
    }

    if is_key_pressed(KeyCode::Escape) {
        if app.pause_open {
            app.pause_open = false;
        } else if app.ui.context_menu.take().is_some() {
            // closed pie / context
        } else if app.ui.overlay.take().is_some() {
            // closed corner overlay
        } else if app.ui.build_open {
            if !app.ui.build_search.is_empty() {
                app.ui.build_search.clear();
                app.ui.build_scroll = 0.0;
            } else {
                app.ui.close_build();
            }
        } else if app.ui.wire_from.take().is_some() || app.ui.selected.take().is_some() {
            app.ui.hotbar_drag_from = None;
        } else {
            app.pause_open = true;
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
    let follow = app.player.cam_mode == CamMode::Follow;
    let cam = &mut app.cam;
    let ui = &mut app.ui;

    // Middle-mouse free pan — only in Free mode (Follow keeps cam on the mech).
    if !follow {
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
    } else {
        ui.panning = false;
    }

    // Build menu owns the wheel for grid scrolling while open.
    if ui.build_open {
        return;
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
    cam.quantize();
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
        BuildingKind::Totem => Color::from_rgba(140, 100, 220, 255),
        BuildingKind::Turret => Color::from_rgba(200, 90, 90, 255),
        BuildingKind::PowerWire => Color::from_rgba(255, 190, 70, 255),
        BuildingKind::Conveyor => BELT_YELLOW,
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
    let w = s(760.0);
    let h = s(540.0);
    Rect {
        x: (screen_width() - w) * 0.5,
        y: (screen_height() - h) * 0.5 - s(20.0),
        w,
        h,
    }
}

fn handle_build_search_input(ui: &mut Ui) {
    if !ui.build_open {
        return;
    }
    if ui.suppress_search_chars {
        ui.suppress_search_chars = false;
        while get_char_pressed().is_some() {}
        return;
    }
    // Auto-focus search so type-to-filter always works while the menu is open.
    ui.build_search_focus = true;
    while let Some(c) = get_char_pressed() {
        if ui.build_search.len() >= 48 {
            break;
        }
        if c.is_ascii_alphabetic() || c == ' ' || c == '-' || c == '_' {
            ui.build_search.push(c);
            ui.build_scroll = 0.0;
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        ui.build_search.pop();
        ui.build_scroll = 0.0;
    }
}

fn place_building(app: &mut App, kind: BuildingKind, x: f32, y: f32, facing: Facing) {
    if kind.is_cable() {
        return;
    }
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
            let probe = Node::new(kind, x, y, facing);
            let (cx, cy) = (x + probe.w() * 0.5, y + probe.h() * 0.5);
            if app.storm.point_in_storm(cx, cy, &app.world) {
                app.join_status = format!("Placed #{id} · exposed to storm!");
            } else {
                app.join_status = format!("Placed #{id}");
            }
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

fn connect_ports_net(app: &mut App, from: (u32, usize), to: (u32, usize)) -> bool {
    let tool = app.ui.selected;
    let is_host = app.net.as_ref().map(|n| n.is_host).unwrap_or(true);
    if app.net.is_none() || is_host {
        if let Some((power, a, b)) = connect_with_tool(&mut app.world, from, to, tool) {
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
            true
        } else {
            let power = matches!(tool, Some(BuildingKind::PowerWire));
            if let Some(hint) = app.world.connect_fail_hint(from, to, power) {
                app.status_toast = hint.into();
            } else {
                app.status_toast = "Can't connect those ports".into();
            }
            false
        }
    } else if let Some(net) = app.net.as_ref() {
        let power = matches!(tool, Some(BuildingKind::PowerWire));
        let _ = net.tx.send(NetCommand::Link {
            power,
            from_node: from.0,
            from_port: from.1,
            to_node: to.0,
            to_port: to.1,
            request: true,
        });
        true
    } else {
        false
    }
}

fn connect_with_tool(
    world: &mut World,
    from: (u32, usize),
    to: (u32, usize),
    tool: Option<BuildingKind>,
) -> Option<(bool, (u32, usize), (u32, usize))> {
    match tool {
        Some(BuildingKind::PowerWire) => {
            let before = world.links.len();
            if !world.connect_power(from, to) {
                return None;
            }
            let l = world.links.get(before)?;
            Some((true, (l.from_node, l.from_port), (l.to_node, l.to_port)))
        }
        _ => None,
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
                app.ui.close_build();
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

    // World right-click.
    if !app.ui.build_open
        && is_mouse_button_pressed(MouseButton::Right)
        && !point_in_hud_chrome(mouse.0, mouse.1)
    {
        if app.ui.wire_from.take().is_some() {
            return;
        }
        // Belt tool: RMB mines/erases the tile under the cursor (keep tool equipped).
        if app.ui.selected == Some(BuildingKind::Conveyor) {
            let (tx, ty) = world_to_tile(wx, wy);
            if app.world.remove_belt_at(tx, ty) {
                app.status_toast = "Belt removed".into();
            }
            return;
        }
        if app.ui.selected.take().is_some() {
            // Right-click cancels other place tools first.
            return;
        }
        let target = if let Some(id) = app.world.hit_node(wx, wy) {
            ContextTarget::Building(id)
        } else {
            let (tx, ty) = world_to_tile(wx, wy);
            if app.world.belt_at(tx, ty).is_some() {
                let _ = app.world.remove_belt_at(tx, ty);
                return;
            }
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

    // Conveyor tool: drag-paint Factorio-style belt tiles on the grid.
    // (RMB erase is handled in handle_hud_input so it isn't eaten by tool-cancel.)
    if app.ui.selected == Some(BuildingKind::Conveyor) {
        if is_mouse_button_down(MouseButton::Left) {
            let (tx, ty) = world_to_tile(wx, wy);
            if app.ui.belt_paint_last != Some((tx, ty)) {
                if app.world.paint_belt(tx, ty, app.ui.place_facing) {
                    app.ui.belt_paint_last = Some((tx, ty));
                } else if app.ui.belt_paint_last.is_none() {
                    app.status_toast = "Can't place belt on a building".into();
                }
            }
            return;
        }
        if is_mouse_button_released(MouseButton::Left) {
            app.ui.belt_paint_last = None;
        }
        return;
    } else {
        app.ui.belt_paint_last = None;
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        // Port linking only while Wire tool is selected.
        if app.ui.selected.filter(|k| k.is_cable()).is_some() {
            if let Some(port) = app.world.hit_port(wx, wy, port_r) {
                let port_ok = app
                    .world
                    .nodes
                    .get(&port.0)
                    .and_then(|n| n.ports.get(port.1))
                    .map(|p| p.kind.is_energy())
                    .unwrap_or(false);
                if port_ok {
                    if let Some(from) = app.ui.wire_from {
                        if from != port {
                            if connect_ports_net(app, from, port) {
                                app.ui.wire_from = None;
                            }
                        } else {
                            app.ui.wire_from = None;
                        }
                    } else {
                        app.ui.wire_from = Some(port);
                    }
                }
                return;
            }
        }
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(kind) = app.ui.selected {
            if kind.is_cable() || kind.is_belt_tool() {
                return;
            }
            if app.world.hit_node(wx, wy).is_none() {
                let (x, y) = snap_building_xy(kind, app.ui.place_facing, wx, wy);
                place_building(app, kind, x, y, app.ui.place_facing);
                return;
            }
        }
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(id) = app.world.hit_node(wx, wy) {
            if let Some(n) = app.world.nodes.get(&id) {
                if n.kind.is_cable() {
                    app.ui.wire_from = None;
                    app.ui.drag_node = None;
                    return;
                }
                app.ui.drag_node = Some(id);
                app.ui.drag_off = (wx - n.x, wy - n.y);
                app.ui.wire_from = None;
            }
        }
    }
    if is_mouse_button_released(MouseButton::Left) {
        if let Some(id) = app.ui.drag_node.take() {
            if let Some(n) = app.world.nodes.get(&id) {
                // Snap dragged buildings onto the grid.
                let (sx, sy) = snap_building_xy(n.kind, n.facing, n.x + n.w() * 0.5, n.y + n.h() * 0.5);
                let _ = app.world.try_move_node(id, sx, sy);
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
    }
    if let Some(id) = app.ui.drag_node {
        if is_mouse_button_down(MouseButton::Left) {
            let nx = wx - app.ui.drag_off.0;
            let ny = wy - app.ui.drag_off.1;
            let _ = app.world.try_move_node(id, nx, ny);
        }
    }
}

fn draw_game(app: &mut App, mouse: (f32, f32), wx: f32, wy: f32) {
    clear_background(BG);
    draw_infinite_grid(&app.cam);
    draw_power_fields(&app.world, &app.cam);
    draw_belt_tiles(&app.world, &app.cam, &app.ui, wx, wy);
    draw_power_links(&app.world, &app.cam, &app.ui, wx, wy);
    draw_nests_and_raiders(&app.world, &app.cam, &app.storm);
    draw_storm_blots(&app.world, &app.cam);
    let hover_id = if point_in_hud_chrome(mouse.0, mouse.1) || app.ui.build_open {
        None
    } else {
        app.world.hit_node(wx, wy).filter(|&id| {
            app.world
                .nodes
                .get(&id)
                .map(|n| !n.kind.is_cable())
                .unwrap_or(false)
        })
    };
    draw_nodes(&app.world, &app.cam, &app.ui, hover_id);
    draw_combat_shots(&app.world, &app.cam);
    player::draw_player(
        &app.player,
        app.cam.x,
        app.cam.y,
        app.cam.zoom,
        peer_color(app.local_player_id),
        None,
    );
    draw_placement_ghost(&app.world, &app.ui, &app.cam, &app.storm, wx, wy);
    draw_peer_cursors(app);
    draw_storm(&app.storm, &app.world, &app.cam);
    draw_lightning_fx(app);
    // Cam mode chip (top-left under any net banner).
    {
        let label = format!("{} · WASD move · C toggle", app.player.cam_mode.label());
        draw_text(&label, 16.0, 28.0, 16.0, TEXT_DIM);
    }
    if let Some(net) = app.net.as_ref() {
        if net.is_host {
            draw_text(
                &format!("Host  code {}  ·  {}", app.host_code, app.host_addr),
                16.0,
                50.0,
                18.0,
                TEXT_DIM,
            );
        }
        if !app.join_status.is_empty() {
            draw_text(&app.join_status, 16.0, 72.0, 16.0, ACCENT);
        }
    }
    if app.settings.show_fps {
        let fps = get_fps();
        let label = format!("{fps} FPS · {} UPS", TARGET_UPS as i32);
        let tw = measure_text(&label, None, 18, 1.0).width;
        draw_text(
            &label,
            screen_width() - tw - 16.0,
            28.0,
            18.0,
            TEXT_DIM,
        );
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

fn tick_storm_lightning(app: &mut App, dt: f32) {
    app.lightning_cd -= dt;
    if app.lightning_cd > 0.0 {
        return;
    }
    let seed = app.storm.time * 11.17 + app.lightning_fx.len() as f32 * 3.9;
    // Ambient bolts are frequent; damaging hits are a subset.
    let wait = 0.35 + storm_hash01(seed) * 0.85;
    app.lightning_cd = wait;

    let zones = app.storm.clear_zones(&app.world);
    let intensity = 0.55 + storm_hash01(seed + 2.2) * 0.85;

    // Ambient strike somewhere in the storm (often near the camera for visibility).
    let (tx, ty) = ambient_strike_point(app, seed + 5.0, &zones);
    app.storm.trigger_flash(tx, ty, intensity);
    spawn_lightning_bolt(app, tx, ty - 520.0 - storm_hash01(seed + 8.0) * 240.0, tx, ty, seed, 0.28, 2.4);

    // Chance of a second ambient fork nearby.
    if storm_hash01(seed + 12.0) > 0.55 {
        let (tx2, ty2) = ambient_strike_point(app, seed + 15.0, &zones);
        spawn_lightning_bolt(
            app,
            tx2,
            ty2 - 380.0 - storm_hash01(seed + 16.0) * 160.0,
            tx2,
            ty2,
            seed + 20.0,
            0.22,
            1.8,
        );
        if storm_hash01(seed + 21.0) > 0.7 {
            app.storm.trigger_flash(tx2, ty2, intensity * 0.65);
        }
    }

    // Sometimes the storm hits an exposed building (rare accent — nests are the real threat).
    let victims: Vec<(u32, f32, f32)> = app
        .world
        .nodes
        .iter()
        .filter_map(|(&id, n)| {
            if n.kind.is_cable() {
                return None;
            }
            let (cx, cy) = n.center();
            if app.storm.in_clear(cx, cy, &zones) {
                None
            } else {
                Some((id, cx, cy))
            }
        })
        .collect();
    if victims.is_empty() || storm_hash01(seed + 30.0) < 0.88 {
        return;
    }

    let pick = (storm_hash01(seed + 31.0) * victims.len() as f32).floor() as usize;
    let (id, cx, cy) = victims[pick.min(victims.len() - 1)];
    spawn_lightning_bolt(
        app,
        cx + (storm_hash01(seed + 32.0) - 0.5) * 60.0,
        cy - 480.0 - storm_hash01(seed + 33.0) * 200.0,
        cx,
        cy,
        seed + 40.0,
        0.4,
        3.2,
    );
    app.storm.trigger_flash(cx, cy, 1.15);

    let destroyed = if let Some(n) = app.world.nodes.get_mut(&id) {
        n.hp = (n.hp - LIGHTNING_DAMAGE).max(0.0);
        n.hp <= 0.0
    } else {
        false
    };
    if destroyed {
        remove_building(app, id);
        app.status_toast = "Storm lightning destroyed a building!".into();
    }
}

fn ambient_strike_point(app: &App, seed: f32, zones: &[(f32, f32, f32)]) -> (f32, f32) {
    // Bias toward the view so players see the show.
    let base_r = app.storm.radius * STORM_HARD_CLEAR_SCALE;
    for attempt in 0..8 {
        let s = seed + attempt as f32 * 17.3;
        let ang = storm_hash01(s) * std::f32::consts::TAU;
        let dist = base_r * (1.15 + storm_hash01(s + 1.0) * 2.2);
        let mut x = app.storm.cx + ang.cos() * dist + app.cam.x * 0.15;
        let mut y = app.storm.cy + ang.sin() * dist + app.cam.y * 0.15;
        // Pull toward camera a bit.
        x = x * 0.55 + app.cam.x * 0.45;
        y = y * 0.55 + app.cam.y * 0.45;
        if !app.storm.in_clear(x, y, zones) {
            return (x, y);
        }
    }
    let ang = storm_hash01(seed) * std::f32::consts::TAU;
    (
        app.cam.x + ang.cos() * base_r * 1.4,
        app.cam.y + ang.sin() * base_r * 1.4,
    )
}

fn spawn_lightning_bolt(
    app: &mut App,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    seed: f32,
    life: f32,
    width: f32,
) {
    let segs = 10 + (storm_hash01(seed) * 6.0) as usize;
    let points = jagged_polyline(x0, y0, x1, y1, segs, 55.0, seed);
    let mut branches = Vec::new();
    let branch_count = 1 + (storm_hash01(seed + 50.0) * 2.5) as usize;
    for b in 0..branch_count {
        if points.len() < 4 {
            break;
        }
        let idx = 2 + ((storm_hash01(seed + 60.0 + b as f32) * (points.len() - 3) as f32) as usize);
        let idx = idx.min(points.len() - 2);
        let (px, py) = points[idx];
        let (nx, ny) = points[idx + 1];
        let dx = nx - px;
        let dy = ny - py;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let px_dir = -dy / len;
        let py_dir = dx / len;
        let side = if storm_hash01(seed + 70.0 + b as f32) > 0.5 {
            1.0
        } else {
            -1.0
        };
        let blen = 70.0 + storm_hash01(seed + 80.0 + b as f32) * 140.0;
        let ex = px + dx * 0.3 + px_dir * side * blen;
        let ey = py + dy * 0.3 + py_dir * side * blen;
        let bsegs = 4 + (storm_hash01(seed + 90.0 + b as f32) * 3.0) as usize;
        branches.push(jagged_polyline(px, py, ex, ey, bsegs, 28.0, seed + 100.0 + b as f32));
    }
    app.lightning_fx.push(LightningFx {
        points,
        branches,
        life,
        max_life: life,
        width,
    });
}

fn jagged_polyline(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    segs: usize,
    jag: f32,
    seed: f32,
) -> Vec<(f32, f32)> {
    let segs = segs.max(2);
    let mut pts = Vec::with_capacity(segs + 1);
    for i in 0..=segs {
        let t = i as f32 / segs as f32;
        let mut x = x0 + (x1 - x0) * t;
        let mut y = y0 + (y1 - y0) * t;
        if i > 0 && i < segs {
            let s = seed + i as f32 * 19.7;
            let dx = x1 - x0;
            let dy = y1 - y0;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            let px = -dy / len;
            let py = dx / len;
            let amp = jag * (1.0 - (t - 0.5).abs() * 1.2) * (0.45 + storm_hash01(s) * 0.9);
            let off = (storm_hash01(s + 3.0) - 0.5) * 2.0 * amp;
            x += px * off;
            y += py * off;
        }
        pts.push((x, y));
    }
    pts
}

fn tick_lightning_fx(app: &mut App, dt: f32) {
    for fx in &mut app.lightning_fx {
        fx.life -= dt;
    }
    app.lightning_fx.retain(|fx| fx.life > 0.0);
}

fn draw_lightning_polyline(cam: &Cam, pts: &[(f32, f32)], width: f32, a: f32) {
    if pts.len() < 2 {
        return;
    }
    let glow = Color::from_rgba(140, 180, 255, (70.0 * a) as u8);
    let mid = Color::from_rgba(200, 220, 255, (160.0 * a) as u8);
    let core = Color::from_rgba(255, 255, 255, (240.0 * a) as u8);
    for w in [width * 3.2, width * 1.6, width * 0.7] {
        let col = if w > width * 2.5 {
            glow
        } else if w > width {
            mid
        } else {
            core
        };
        for i in 0..pts.len() - 1 {
            let (sx0, sy0) = cam.world_to_screen(pts[i].0, pts[i].1);
            let (sx1, sy1) = cam.world_to_screen(pts[i + 1].0, pts[i + 1].1);
            draw_line(sx0, sy0, sx1, sy1, w * cam.zoom.max(0.5), col);
        }
    }
}

fn draw_lightning_fx(app: &App) {
    for fx in &app.lightning_fx {
        let a = (fx.life / fx.max_life).clamp(0.0, 1.0);
        // Brief bright hold then falloff.
        let a = if a > 0.7 { 1.0 } else { a / 0.7 };
        draw_lightning_polyline(&app.cam, &fx.points, fx.width, a);
        for branch in &fx.branches {
            draw_lightning_polyline(&app.cam, branch, fx.width * 0.55, a * 0.85);
        }
        if let Some(&(x, y)) = fx.points.last() {
            let (sx, sy) = app.cam.world_to_screen(x, y);
            draw_circle(
                sx,
                sy,
                (14.0 + fx.width * 2.0) * a * app.cam.zoom,
                Color::from_rgba(220, 235, 255, (100.0 * a) as u8),
            );
            draw_circle(
                sx,
                sy,
                (6.0 + fx.width) * a * app.cam.zoom,
                Color::from_rgba(255, 255, 255, (180.0 * a) as u8),
            );
        }
    }
    // No full-screen flash — illumination stays inside the storm fog shader only.
}

fn draw_storm(storm: &Storm, world: &World, cam: &Cam) {
    let zones = storm.clear_zones(world);
    if let Some(mat) = storm.material.as_ref() {
        mat.set_uniform("ScreenSize", vec2(screen_width(), screen_height()));
        mat.set_uniform("CamPos", vec2(cam.x, cam.y));
        mat.set_uniform("CamZoom", cam.zoom);
        mat.set_uniform("Time", storm.time);
        let mut totems = [vec4(0.0, 0.0, 0.0, 0.0); STORM_MAX_TOTEMS];
        for (i, &(cx, cy, radius)) in zones.iter().take(STORM_MAX_TOTEMS).enumerate() {
            totems[i] = vec4(cx, cy, radius, 1.0);
        }
        mat.set_uniform_array("Totems", &totems);
        let mut flashes = [vec4(0.0, 0.0, 0.0, 0.0); STORM_MAX_FLASHES];
        for (i, &(x, y, inten, rad)) in storm.flashes.iter().enumerate() {
            flashes[i] = vec4(x, y, inten, rad);
        }
        mat.set_uniform_array("Flashes", &flashes);
        gl_use_material(mat);
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), WHITE);
        gl_use_default_material();
        return;
    }

    // CPU fallback — denser organic fog without vortices.
    let sw = screen_width();
    let sh = screen_height();
    let cell = 22.0_f32;
    let cols = (sw / cell).ceil() as i32 + 1;
    let rows = (sh / cell).ceil() as i32 + 1;

    for gy in 0..rows {
        for gx in 0..cols {
            let sx = gx as f32 * cell + cell * 0.5;
            let sy = gy as f32 * cell + cell * 0.5;
            let (wx, wy) = cam.screen_to_world(sx, sy);
            let c = storm.coverage_at(wx, wy, &zones);
            if c <= 0.03 {
                continue;
            }
            let mut lit = 0.0_f32;
            for &(fx, fy, inten, rad) in &storm.flashes {
                if inten <= 0.01 {
                    continue;
                }
                let dx = wx - fx;
                let dy = wy - fy;
                let fall = (-(dx * dx + dy * dy) / (rad * rad).max(1.0)).exp();
                lit = lit.max(inten * fall);
            }
            lit = (lit * c).clamp(0.0, 1.5);
            let a = ((55.0 + c * 175.0) * (1.0 + lit * 0.35)).min(255.0) as u8;
            let r = ((95.0 + c * 85.0) + lit * 80.0).min(255.0) as u8;
            let g = ((92.0 + c * 90.0) + lit * 85.0).min(255.0) as u8;
            let b = ((118.0 + c * 75.0) + lit * 90.0).min(255.0) as u8;
            draw_rectangle(
                gx as f32 * cell,
                gy as f32 * cell,
                cell + 1.0,
                cell + 1.0,
                Color::from_rgba(r, g, b, a),
            );
        }
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
            if major { 1.35 } else { 1.0 },
            if major {
                Color::from_rgba(70, 82, 100, 130)
            } else {
                GRID_MINOR_C
            },
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
            if major { 1.35 } else { 1.0 },
            if major {
                Color::from_rgba(70, 82, 100, 130)
            } else {
                GRID_MINOR_C
            },
        );
    }
}

fn draw_power_fields(world: &World, cam: &Cam) {
    for n in world.nodes.values() {
        if n.kind == BuildingKind::PowerPole {
            let (cx, cy) = n.center();
            let (sx, sy) = cam.world_to_screen(cx, cy);
            let r = POLE_RADIUS * cam.zoom;
            let on = n.working;
            draw_circle(
                sx,
                sy,
                r,
                if on {
                    Color::from_rgba(255, 190, 70, 22)
                } else {
                    Color::from_rgba(120, 120, 130, 14)
                },
            );
            draw_circle(
                sx,
                sy,
                r * 0.7,
                if on {
                    Color::from_rgba(255, 205, 100, 14)
                } else {
                    Color::from_rgba(120, 120, 130, 8)
                },
            );
            draw_circle_lines(
                sx,
                sy,
                r,
                2.2,
                if on {
                    with_alpha(POWER_C, 0.2)
                } else {
                    Color::from_rgba(120, 120, 130, 35)
                },
            );
            draw_circle_lines(
                sx,
                sy,
                r,
                1.0,
                if on {
                    POWER_DIM
                } else {
                    Color::from_rgba(120, 120, 130, 50)
                },
            );
        } else if n.kind == BuildingKind::Totem {
            let (cx, cy) = n.center();
            let (sx, sy) = cam.world_to_screen(cx, cy);
            let r = TOTEM_CLEAR_RADIUS * STORM_HARD_CLEAR_SCALE * cam.zoom;
            draw_circle(
                sx,
                sy,
                r,
                if n.powered {
                    Color::from_rgba(140, 100, 220, 18)
                } else {
                    Color::from_rgba(90, 80, 110, 10)
                },
            );
            draw_circle(
                sx,
                sy,
                r * 0.82,
                if n.powered {
                    Color::from_rgba(160, 130, 240, 12)
                } else {
                    Color::from_rgba(90, 80, 110, 6)
                },
            );
            draw_circle_lines(
                sx,
                sy,
                r,
                2.4,
                if n.powered {
                    Color::from_rgba(160, 120, 230, 50)
                } else {
                    Color::from_rgba(100, 90, 120, 30)
                },
            );
            draw_circle_lines(
                sx,
                sy,
                r,
                1.1,
                if n.powered {
                    Color::from_rgba(190, 160, 255, 120)
                } else {
                    Color::from_rgba(100, 90, 120, 50)
                },
            );
        } else if n.kind == BuildingKind::Turret {
            let (cx, cy) = n.center();
            let (sx, sy) = cam.world_to_screen(cx, cy);
            let r = TURRET_RANGE * cam.zoom;
            if n.powered {
                draw_circle(sx, sy, r, Color::from_rgba(200, 80, 80, 10));
            }
            draw_circle_lines(
                sx,
                sy,
                r,
                2.0,
                if n.powered {
                    Color::from_rgba(200, 90, 90, 35)
                } else {
                    Color::from_rgba(120, 80, 80, 20)
                },
            );
            draw_circle_lines(
                sx,
                sy,
                r,
                1.0,
                if n.powered {
                    Color::from_rgba(220, 110, 110, 80)
                } else {
                    Color::from_rgba(120, 80, 80, 35)
                },
            );
        }
    }
}

fn draw_power_manhattan(cam: &Cam, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
    let (sx0, sy0) = cam.world_to_screen(x0, y0);
    let (sx1, sy1) = cam.world_to_screen(x1, y1);
    let mx = (sx0 + sx1) * 0.5;
    // Physical copper-style cable: dark sheath + bright core.
    let outer = (4.2 * cam.zoom).clamp(2.5, 6.5);
    let inner = (outer - 1.6).max(1.2);
    let sheath = Color::from_rgba(90, 70, 30, 220);
    // Soft copper glow under sheath.
    draw_line(sx0, sy0, mx, sy0, outer * 1.8, with_alpha(color, 0.12));
    draw_line(mx, sy0, mx, sy1, outer * 1.8, with_alpha(color, 0.12));
    draw_line(mx, sy1, sx1, sy1, outer * 1.8, with_alpha(color, 0.12));
    draw_line(sx0, sy0, mx, sy0, outer, sheath);
    draw_line(mx, sy0, mx, sy1, outer, sheath);
    draw_line(mx, sy1, sx1, sy1, outer, sheath);
    draw_line(sx0, sy0, mx, sy0, inner, color);
    draw_line(mx, sy0, mx, sy1, inner, color);
    draw_line(mx, sy1, sx1, sy1, inner, color);
    // Tick marks show it's a live power run (not a belt).
    draw_manhattan_ticks(sx0, sy0, mx, sy0, sx1, sy1, cam.zoom, color);
}

fn manhattan_segments(
    sx0: f32,
    sy0: f32,
    sx1: f32,
    sy1: f32,
) -> [(f32, f32, f32, f32); 3] {
    let mx = (sx0 + sx1) * 0.5;
    [(sx0, sy0, mx, sy0), (mx, sy0, mx, sy1), (mx, sy1, sx1, sy1)]
}

fn draw_manhattan_ticks(
    sx0: f32,
    sy0: f32,
    mx: f32,
    _my0: f32,
    sx1: f32,
    sy1: f32,
    zoom: f32,
    color: Color,
) {
    let segs = manhattan_segments(sx0, sy0, sx1, sy1);
    let _ = mx;
    let step = (28.0 * zoom).clamp(14.0, 36.0);
    let tick = (3.5 * zoom).clamp(2.0, 5.0);
    for (ax, ay, bx, by) in segs {
        let dx = bx - ax;
        let dy = by - ay;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 8.0 {
            continue;
        }
        let (ux, uy) = (dx / len, dy / len);
        let (nx, ny) = (-uy, ux);
        let mut d = step * 0.5;
        while d < len - 2.0 {
            let px = ax + ux * d;
            let py = ay + uy * d;
            draw_line(
                px - nx * tick,
                py - ny * tick,
                px + nx * tick,
                py + ny * tick,
                1.2,
                with_alpha(color, 0.55),
            );
            d += step;
        }
    }
}

fn draw_chevron(cx: f32, cy: f32, dx: f32, dy: f32, size: f32, color: Color) {
    let len = (dx * dx + dy * dy).sqrt().max(1e-3);
    let (ux, uy) = (dx / len, dy / len);
    let (nx, ny) = (-uy, ux);
    let tip_x = cx + ux * size;
    let tip_y = cy + uy * size;
    let back = size * 0.85;
    let wing = size * 0.7;
    let bx = cx - ux * back * 0.15;
    let by = cy - uy * back * 0.15;
    draw_triangle(
        Vec2::new(tip_x, tip_y),
        Vec2::new(bx - nx * wing, by - ny * wing),
        Vec2::new(bx + nx * wing, by + ny * wing),
        color,
    );
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
        if ui.selected == Some(BuildingKind::PowerWire) {
            if let Some(n) = world.nodes.get(&nid) {
                if let Some(p) = n.ports.get(pid) {
                    if p.kind.is_energy() {
                        if let Some((ax, ay)) = n.port_world(pid) {
                            let dist = (wx - ax).abs() + (wy - ay).abs();
                            let ok = dist <= POWER_WIRE_MAX_REACH;
                            let col = if ok {
                                Color::from_rgba(255, 190, 70, 160)
                            } else {
                                Color::from_rgba(255, 90, 70, 180)
                            };
                            draw_power_manhattan(cam, ax, ay, wx, wy, col);
                            // Reach ring from selected socket.
                            let (sx, sy) = cam.world_to_screen(ax, ay);
                            draw_circle_lines(
                                sx,
                                sy,
                                POWER_WIRE_MAX_REACH * cam.zoom,
                                1.2,
                                Color::from_rgba(255, 190, 70, 70),
                            );
                        }
                    }
                }
            }
        }
    }
}

fn draw_belt_tiles(world: &World, cam: &Cam, ui: &Ui, wx: f32, wy: f32) {
    let ts = TILE_SIZE * cam.zoom;
    for (&(tx, ty), tile) in &world.belt_tiles {
        let (ox, oy) = tile_origin(tx, ty);
        let (sx, sy) = cam.world_to_screen(ox, oy);
        let load = tile.item_count();
        let congested = load >= 8;
        let fill = if congested {
            Color::from_rgba(78, 42, 24, 230)
        } else {
            Color::from_rgba(42, 36, 22, 225)
        };
        let edge = if congested {
            Color::from_rgba(240, 140, 70, 255)
        } else {
            BELT_YELLOW
        };
        let shape = belts::belt_shape(&world.belt_tiles, tx, ty, tile.dir);
        draw_belt_tile_shape(sx, sy, ts, tile.dir, shape, fill, edge);

        for lane in 0..2 {
            for it in &tile.lanes[lane].items {
                let (iwx, iwy) =
                    belts::item_world_pos_shaped(tx, ty, tile.dir, shape, lane, it.progress);
                let (isx, isy) = cam.world_to_screen(iwx, iwy);
                draw_item_chip(isx, isy, cam.zoom, it.item);
            }
        }
    }

    // Ghost belt under cursor while paint tool is equipped.
    if ui.selected == Some(BuildingKind::Conveyor) {
        let (tx, ty) = world_to_tile(wx, wy);
        let (ox, oy) = tile_origin(tx, ty);
        let (sx, sy) = cam.world_to_screen(ox, oy);
        let building_hit = world
            .hit_node(ox + TILE_SIZE * 0.5, oy + TILE_SIZE * 0.5)
            .is_some();
        let col = if building_hit {
            Color::from_rgba(220, 70, 70, 100)
        } else {
            Color::from_rgba(210, 170, 55, 90)
        };
        draw_rectangle(sx, sy, ts, ts, col);
        let (cx, cy) = (sx + ts * 0.5, sy + ts * 0.5);
        let (dx, dy) = match ui.place_facing {
            Facing::E => (1.0, 0.0),
            Facing::W => (-1.0, 0.0),
            Facing::S => (0.0, 1.0),
            Facing::N => (0.0, -1.0),
        };
        draw_chevron(cx, cy, dx, dy, ts * 0.28, BELT_YELLOW);
    }
}

fn facing_unit(f: Facing) -> (f32, f32) {
    match f {
        Facing::E => (1.0, 0.0),
        Facing::W => (-1.0, 0.0),
        Facing::S => (0.0, 1.0),
        Facing::N => (0.0, -1.0),
    }
}

fn draw_belt_tile_shape(
    sx: f32,
    sy: f32,
    ts: f32,
    dir: Facing,
    shape: belts::BeltShape,
    fill: Color,
    edge: Color,
) {
    let cx = sx + ts * 0.5;
    let cy = sy + ts * 0.5;
    let (fx, fy) = facing_unit(dir);
    let inset = (ts * 0.08).clamp(1.5, 4.0);
    let top = mix_rgb(fill, Color::from_rgba(95, 82, 48, 255), 0.45);
    let rail = Color::from_rgba(28, 24, 16, 200);
    match shape {
        belts::BeltShape::Straight => {
            // Soft outer glow
            draw_rectangle(
                sx - 1.5,
                sy - 1.5,
                ts + 3.0,
                ts + 3.0,
                with_alpha(edge, 0.12),
            );
            draw_rectangle(sx, sy, ts, ts, fill);
            // Fake volume: lighter strip along travel "top"
            let strip = ts * 0.22;
            if fx.abs() > fy.abs() {
                draw_rectangle(sx + inset, sy + inset, ts - inset * 2.0, strip, top);
                draw_rectangle(
                    sx + inset,
                    sy + ts - inset - strip * 0.55,
                    ts - inset * 2.0,
                    strip * 0.55,
                    rail,
                );
            } else {
                draw_rectangle(sx + inset, sy + inset, strip, ts - inset * 2.0, top);
                draw_rectangle(
                    sx + ts - inset - strip * 0.55,
                    sy + inset,
                    strip * 0.55,
                    ts - inset * 2.0,
                    rail,
                );
            }
            draw_rectangle_lines(sx, sy, ts, ts, 2.4, with_alpha(edge, 0.35));
            draw_rectangle_lines(sx + 0.5, sy + 0.5, ts - 1.0, ts - 1.0, 1.2, edge);
            draw_chevron(cx, cy, fx, fy, ts * 0.26, with_alpha(edge, 0.55));
            draw_chevron(cx, cy, fx, fy, ts * 0.22, edge);
        }
        belts::BeltShape::CornerLeft | belts::BeltShape::CornerRight => {
            let pts = corner_triangle(sx, sy, ts, dir, shape);
            draw_poly_fill(&pts, with_alpha(edge, 0.12));
            draw_poly_fill(&pts, fill);
            // Inset lighter plate toward centroid.
            let (ax, ay) = triangle_centroid(&pts);
            let mut inset_pts = pts;
            for p in &mut inset_pts {
                p.0 = p.0 + (ax - p.0) * 0.18;
                p.1 = p.1 + (ay - p.1) * 0.18;
            }
            draw_poly_fill(&inset_pts, top);
            draw_poly_outline(&pts, 3.0, with_alpha(edge, 0.3));
            draw_poly_outline(&pts, 1.5, edge);
            let entry = match shape {
                belts::BeltShape::CornerLeft => belts::facing_left(dir),
                belts::BeltShape::CornerRight => belts::facing_right(dir),
                belts::BeltShape::Straight => unreachable!(),
            };
            let (ix, iy) = facing_unit(belts::facing_opposite(entry));
            let mut bx = ix + fx;
            let mut by = iy + fy;
            let len = (bx * bx + by * by).sqrt().max(1e-3);
            bx /= len;
            by /= len;
            draw_chevron(ax, ay, bx, by, ts * 0.24, with_alpha(edge, 0.5));
            draw_chevron(ax, ay, bx, by, ts * 0.2, edge);
        }
    }
}

/// Right-triangle covering the tile except the empty inner corner of the bend.
fn corner_triangle(
    sx: f32,
    sy: f32,
    ts: f32,
    exit: Facing,
    shape: belts::BeltShape,
) -> [(f32, f32); 3] {
    let nw = (sx, sy);
    let ne = (sx + ts, sy);
    let sw = (sx, sy + ts);
    let se = (sx + ts, sy + ts);
    let entry = match shape {
        belts::BeltShape::CornerLeft => belts::facing_left(exit),
        belts::BeltShape::CornerRight => belts::facing_right(exit),
        belts::BeltShape::Straight => unreachable!(),
    };
    // Empty corner = opposite(entry) + opposite(exit) (inner elbow of the turn).
    let (dx, dy) = (
        belts::facing_delta(belts::facing_opposite(entry)).0
            + belts::facing_delta(belts::facing_opposite(exit)).0,
        belts::facing_delta(belts::facing_opposite(entry)).1
            + belts::facing_delta(belts::facing_opposite(exit)).1,
    );
    let empty = if dx < 0 && dy < 0 {
        nw
    } else if dx > 0 && dy < 0 {
        ne
    } else if dx < 0 && dy > 0 {
        sw
    } else {
        se
    };
    let all = [nw, ne, se, sw];
    let mut out = [(0.0, 0.0); 3];
    let mut i = 0;
    for p in all {
        if p != empty {
            out[i] = p;
            i += 1;
        }
    }
    out
}

fn triangle_centroid(pts: &[(f32, f32); 3]) -> (f32, f32) {
    (
        (pts[0].0 + pts[1].0 + pts[2].0) / 3.0,
        (pts[0].1 + pts[1].1 + pts[2].1) / 3.0,
    )
}

fn draw_nodes(world: &World, cam: &Cam, ui: &Ui, hover_id: Option<u32>) {
    let mut ids: Vec<u32> = world.nodes.keys().copied().collect();
    ids.sort_unstable();
    let connect_tool = ui.selected.filter(|k| k.is_cable());
    for id in ids {
        if let Some(n) = world.nodes.get(&id) {
            if n.kind.is_cable() {
                continue;
            }
            draw_node(
                cam,
                n,
                hover_id == Some(id),
                connect_tool,
                ui.wire_from,
                world,
                id,
            );
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NodeLod {
    Far,
    Mid,
    Near,
}

fn node_lod(zoom: f32) -> NodeLod {
    if zoom < 0.55 {
        NodeLod::Far
    } else if zoom < 0.95 {
        NodeLod::Mid
    } else {
        NodeLod::Near
    }
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a,
    }
}

fn mix_rgb(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

fn draw_item_chip(sx: f32, sy: f32, zoom: f32, item: Item) {
    let r = (5.0 * zoom).clamp(3.2, 7.5);
    match item {
        Item::IronOre => {
            // Faceted rock: clustered irregular blobs + mineral sparkle.
            draw_circle(sx + r * 0.05, sy + r * 0.25, r * 1.05, Color::from_rgba(0, 0, 0, 40));
            let rock = Color::from_rgba(95, 98, 108, 255);
            let rock_d = Color::from_rgba(58, 60, 68, 255);
            let rock_h = Color::from_rgba(170, 175, 185, 255);
            let vein = Color::from_rgba(200, 150, 90, 200);
            draw_circle(sx - r * 0.2, sy + r * 0.1, r * 0.72, rock_d);
            draw_circle(sx + r * 0.28, sy - r * 0.05, r * 0.58, rock);
            draw_circle(sx + r * 0.05, sy + r * 0.28, r * 0.48, rock_d);
            draw_circle(sx - r * 0.05, sy - r * 0.22, r * 0.42, rock);
            draw_circle(sx - r * 0.3, sy - r * 0.05, r * 0.22, rock_h);
            draw_circle(sx + r * 0.15, sy + r * 0.05, r * 0.12, vein);
            draw_circle(sx + r * 0.35, sy - r * 0.2, r * 0.08, Color::from_rgba(230, 210, 140, 180));
            draw_circle_lines(sx - r * 0.2, sy + r * 0.1, r * 0.72, 1.0, Color::from_rgba(40, 42, 48, 160));
            draw_circle_lines(sx + r * 0.28, sy - r * 0.05, r * 0.58, 1.0, Color::from_rgba(40, 42, 48, 140));
        }
        Item::IronIngot => {
            let hw = r * 1.05;
            let hh = r * 0.58;
            draw_ellipse(sx, sy + hh * 0.55, hw * 1.05, hh * 0.45, 0.0, Color::from_rgba(0, 0, 0, 40));
            // Beveled bar: dark base, lit top face, edge highlight.
            draw_rectangle(sx - hw, sy - hh * 0.35, hw * 2.0, hh * 1.55, Color::from_rgba(55, 62, 72, 255));
            draw_rectangle(sx - hw * 0.92, sy - hh * 0.85, hw * 1.84, hh * 0.95, INGOT_C);
            draw_rectangle(
                sx - hw * 0.75,
                sy - hh * 0.75,
                hw * 1.5,
                hh * 0.28,
                Color::from_rgba(245, 250, 255, 160),
            );
            draw_line(
                sx - hw * 0.9,
                sy + hh * 0.15,
                sx + hw * 0.9,
                sy + hh * 0.15,
                1.0,
                Color::from_rgba(30, 35, 42, 180),
            );
            draw_rectangle_lines(
                sx - hw,
                sy - hh * 0.35,
                hw * 2.0,
                hh * 1.55,
                1.1,
                Color::from_rgba(210, 225, 240, 200),
            );
        }
    }
}

fn draw_poly_fill(pts: &[(f32, f32)], color: Color) {
    if pts.len() < 3 {
        return;
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    for &(x, y) in pts {
        cx += x;
        cy += y;
    }
    let n = pts.len() as f32;
    cx /= n;
    cy /= n;
    for i in 0..pts.len() {
        let (x0, y0) = pts[i];
        let (x1, y1) = pts[(i + 1) % pts.len()];
        draw_triangle(Vec2::new(cx, cy), Vec2::new(x0, y0), Vec2::new(x1, y1), color);
    }
}

fn draw_poly_outline(pts: &[(f32, f32)], thickness: f32, color: Color) {
    if pts.len() < 2 {
        return;
    }
    for i in 0..pts.len() {
        let (x0, y0) = pts[i];
        let (x1, y1) = pts[(i + 1) % pts.len()];
        draw_line(x0, y0, x1, y1, thickness, color);
    }
}

fn hex_points(cx: f32, cy: f32, rx: f32, ry: f32) -> [(f32, f32); 6] {
    let mut pts = [(0.0, 0.0); 6];
    for i in 0..6 {
        let a = std::f32::consts::FRAC_PI_6 + i as f32 * std::f32::consts::FRAC_PI_3;
        pts[i] = (cx + a.cos() * rx, cy + a.sin() * ry);
    }
    pts
}

/// Unique building silhouette inside the AABB — readable at any zoom, zero art.
fn draw_building_silhouette(
    kind: BuildingKind,
    facing: Facing,
    sx: f32,
    sy: f32,
    w: f32,
    h: f32,
    fill: Color,
    border: Color,
    accent: Color,
    lit: bool,
    detail: bool,
) {
    let cx = sx + w * 0.5;
    let cy = sy + h * 0.5;
    let edge = (w.min(h) * 0.035).clamp(1.2, 3.0);
    let accent_fill = with_alpha(accent, if lit { 0.35 } else { 0.18 });
    let dim = Color::from_rgba(170, 195, 220, if lit { 180 } else { 90 });

    match kind {
        BuildingKind::Solar => {
            // Wide hex panel.
            let pts = hex_points(cx, cy, w * 0.46, h * 0.42);
            draw_poly_fill(&pts, fill);
            draw_poly_fill(&pts, accent_fill);
            draw_poly_outline(&pts, edge, border);
            if detail {
                draw_line(cx - w * 0.28, cy, cx + w * 0.28, cy, edge, dim);
                draw_line(cx, cy - h * 0.28, cx, cy + h * 0.28, edge, dim);
                draw_circle_lines(cx, cy, w.min(h) * 0.12, edge, accent);
            }
        }
        BuildingKind::PowerPole => {
            // Tall mast diamond + cross-arm.
            let mast = [
                (cx, sy + h * 0.06),
                (cx + w * 0.16, cy),
                (cx, sy + h * 0.94),
                (cx - w * 0.16, cy),
            ];
            draw_poly_fill(&mast, fill);
            draw_poly_fill(&mast, accent_fill);
            draw_poly_outline(&mast, edge, border);
            let arm_y = sy + h * 0.28;
            draw_rectangle(sx + w * 0.08, arm_y - edge, w * 0.84, edge * 2.2, border);
            draw_circle(cx, sy + h * 0.12, w.min(h) * 0.08, accent);
            if detail {
                draw_circle_lines(cx, cy, w.min(h) * 0.22, edge, with_alpha(accent, 0.5));
            }
        }
        BuildingKind::OreNode => {
            // Layered ore outcrop: dark base rock, mid plates, mineral veins, sparkle.
            let r0 = w.min(h) * 0.30;
            let rock = Color::from_rgba(52, 56, 64, 255);
            let rock_m = Color::from_rgba(78, 84, 96, 255);
            let rock_h = Color::from_rgba(130, 138, 150, 220);
            let vein = Color::from_rgba(210, 160, 80, 200);
            draw_ellipse(cx, cy + h * 0.28, w * 0.42, h * 0.18, 0.0, Color::from_rgba(0, 0, 0, 50));
            draw_circle(cx - w * 0.14, cy + h * 0.08, r0 * 1.2, rock);
            draw_circle(cx + w * 0.2, cy - h * 0.06, r0 * 0.95, rock);
            draw_circle(cx + w * 0.02, cy + h * 0.2, r0 * 0.8, rock);
            draw_circle(cx - w * 0.14, cy + h * 0.08, r0 * 1.05, rock_m);
            draw_circle(cx + w * 0.18, cy - h * 0.1, r0 * 0.7, rock_m);
            draw_circle(cx - w * 0.02, cy - h * 0.02, r0 * 0.55, rock_h);
            draw_circle(cx + w * 0.08, cy + h * 0.05, r0 * 0.22, vein);
            draw_circle(cx - w * 0.22, cy + h * 0.0, r0 * 0.14, vein);
            draw_circle(cx + w * 0.28, cy - h * 0.02, r0 * 0.1, Color::from_rgba(255, 220, 140, 180));
            draw_circle_lines(cx - w * 0.14, cy + h * 0.08, r0 * 1.2, edge, border);
            draw_circle_lines(cx + w * 0.2, cy - h * 0.06, r0 * 0.95, edge, border);
            draw_circle_lines(cx + w * 0.02, cy + h * 0.2, r0 * 0.8, edge, border);
            if detail {
                draw_circle(cx - w * 0.08, cy - h * 0.12, r0 * 0.18, accent);
                draw_circle_lines(cx, cy, r0 * 1.55, 1.2, with_alpha(accent, 0.35));
            }
        }
        BuildingKind::Smelter => {
            // Trapezoid body + chimney.
            let body = [
                (sx + w * 0.08, sy + h * 0.92),
                (sx + w * 0.92, sy + h * 0.92),
                (sx + w * 0.78, sy + h * 0.38),
                (sx + w * 0.22, sy + h * 0.38),
            ];
            draw_poly_fill(&body, fill);
            draw_poly_fill(&body, accent_fill);
            draw_poly_outline(&body, edge, border);
            let chim = [
                (cx - w * 0.08, sy + h * 0.38),
                (cx + w * 0.08, sy + h * 0.38),
                (cx + w * 0.06, sy + h * 0.08),
                (cx - w * 0.06, sy + h * 0.08),
            ];
            draw_poly_fill(&chim, fill);
            draw_poly_outline(&chim, edge, border);
            if detail {
                draw_line(sx + w * 0.3, sy + h * 0.65, sx + w * 0.7, sy + h * 0.65, edge, dim);
                draw_circle(cx, sy + h * 0.1, edge * 1.5, accent);
            }
        }
        BuildingKind::Box => {
            // Beveled crate (octagon).
            let inset_x = w * 0.12;
            let inset_y = h * 0.12;
            let pts = [
                (sx + inset_x, sy),
                (sx + w - inset_x, sy),
                (sx + w, sy + inset_y),
                (sx + w, sy + h - inset_y),
                (sx + w - inset_x, sy + h),
                (sx + inset_x, sy + h),
                (sx, sy + h - inset_y),
                (sx, sy + inset_y),
            ];
            draw_poly_fill(&pts, fill);
            draw_poly_fill(&pts, accent_fill);
            draw_poly_outline(&pts, edge, border);
            if detail {
                draw_line(sx + w * 0.2, cy, sx + w * 0.8, cy, edge, dim);
                draw_line(cx, sy + h * 0.2, cx, sy + h * 0.8, edge, dim);
            }
        }
        BuildingKind::Splitter => {
            // Looks like three belt tiles along the long axis, flow toward outputs.
            let along_w = w >= h;
            let (cells, cell_w, cell_h) = if along_w {
                (3, w / 3.0, h)
            } else {
                (3, w, h / 3.0)
            };
            let (fx, fy) = match facing {
                Facing::E => (1.0, 0.0),
                Facing::W => (-1.0, 0.0),
                Facing::S => (0.0, 1.0),
                Facing::N => (0.0, -1.0),
            };
            for i in 0..cells {
                let (x0, y0) = if along_w {
                    (sx + cell_w * i as f32, sy)
                } else {
                    (sx, sy + cell_h * i as f32)
                };
                draw_rectangle(x0 + 1.0, y0 + 1.0, cell_w - 2.0, cell_h - 2.0, fill);
                draw_rectangle(x0 + 1.0, y0 + 1.0, cell_w - 2.0, cell_h - 2.0, accent_fill);
                draw_rectangle_lines(
                    x0 + 1.0,
                    y0 + 1.0,
                    cell_w - 2.0,
                    cell_h - 2.0,
                    edge,
                    border,
                );
                draw_chevron(
                    x0 + cell_w * 0.5,
                    y0 + cell_h * 0.5,
                    fx,
                    fy,
                    cell_w.min(cell_h) * 0.28,
                    accent,
                );
            }
            if detail {
                let (ox, oy) = match facing {
                    Facing::E => (sx + w - 3.0, cy),
                    Facing::W => (sx + 3.0, cy),
                    Facing::S => (cx, sy + h - 3.0),
                    Facing::N => (cx, sy + 3.0),
                };
                draw_circle(ox, oy, 2.5, accent);
            }
        }
        BuildingKind::Totem => {
            // Tall spire + pedestal disk.
            let spire = [
                (cx, sy + h * 0.04),
                (cx + w * 0.18, sy + h * 0.55),
                (cx + w * 0.12, sy + h * 0.78),
                (cx - w * 0.12, sy + h * 0.78),
                (cx - w * 0.18, sy + h * 0.55),
            ];
            draw_circle(cx, sy + h * 0.82, w * 0.32, fill);
            draw_circle(cx, sy + h * 0.82, w * 0.32, accent_fill);
            draw_circle_lines(cx, sy + h * 0.82, w * 0.32, edge, border);
            draw_poly_fill(&spire, fill);
            draw_poly_fill(&spire, accent_fill);
            draw_poly_outline(&spire, edge, border);
            draw_circle(cx, sy + h * 0.12, w.min(h) * 0.07, accent);
            if detail {
                draw_circle_lines(cx, sy + h * 0.4, w * 0.2, edge, with_alpha(accent, 0.55));
            }
        }
        BuildingKind::Turret => {
            // Round base + barrel wedge.
            let r = w.min(h) * 0.38;
            draw_circle(cx, cy + h * 0.08, r, fill);
            draw_circle(cx, cy + h * 0.08, r, accent_fill);
            draw_circle_lines(cx, cy + h * 0.08, r, edge, border);
            let barrel = [
                (cx - w * 0.08, cy + h * 0.02),
                (cx + w * 0.42, cy - h * 0.38),
                (cx + w * 0.12, cy + h * 0.14),
            ];
            draw_poly_fill(&barrel, fill);
            draw_poly_fill(&barrel, with_alpha(accent, if lit { 0.55 } else { 0.3 }));
            draw_poly_outline(&barrel, edge, border);
            if detail {
                draw_circle(cx, cy + h * 0.08, r * 0.28, dim);
            }
        }
        BuildingKind::PowerWire | BuildingKind::Conveyor => {
            draw_rectangle(sx, sy, w, h, fill);
            draw_rectangle_lines(sx, sy, w, h, edge, border);
        }
    }
}

fn draw_node(
    cam: &Cam,
    n: &Node,
    hovered: bool,
    connect_tool: Option<BuildingKind>,
    wire_from: Option<(u32, usize)>,
    world: &World,
    node_id: u32,
) {
    let (sx, sy) = cam.world_to_screen(n.x, n.y);
    let w = n.w() * cam.zoom;
    let h = n.h() * cam.zoom;
    let lod = node_lod(cam.zoom);
    let accent = kind_swatch(n.kind);
    let damaged = n.hp < n.max_hp - 0.5;
    let unpowered = n.kind.needs_power() && !n.powered;
    let lit = !unpowered && (n.working || !n.kind.needs_power() || n.powered);

    let fill = if unpowered {
        Color::from_rgba(48, 30, 34, 220)
    } else if lit {
        Color::from_rgba(22, 32, 46, 230)
    } else {
        Color::from_rgba(26, 30, 38, 210)
    };
    let border = if unpowered {
        Color::from_rgba(220, 80, 80, 230)
    } else if damaged {
        Color::from_rgba(230, 170, 60, 230)
    } else if hovered {
        with_alpha(accent, 1.0)
    } else {
        with_alpha(accent, 0.9)
    };

    let detail = lod != NodeLod::Far;
    draw_building_silhouette(n.kind, n.facing, sx, sy, w, h, fill, border, accent, lit, detail);

    if lod == NodeLod::Near || hovered {
        let label = if hovered && lod == NodeLod::Near {
            n.kind.label()
        } else {
            n.kind.short()
        };
        let fs = (12.0 * cam.zoom).clamp(9.0, 15.0);
        // Label sits under the silhouette so it doesn't fight the shape.
        draw_text(
            label,
            sx + w * 0.5 - measure_text(label, None, fs as u16, 1.0).width * 0.5,
            sy + h + fs + 2.0,
            fs,
            with_alpha(TEXT, 0.85),
        );
    }

    if lod == NodeLod::Near && hovered {
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
                "out {:.1}  {}",
                n.out_ore,
                if n.powered { "OK" } else { "OFF" }
            ),
            BuildingKind::Smelter => format!("in {:.0}  out {:.0}", n.in_ore, n.out_ingot),
            BuildingKind::Box => format!("ore {:.0}  ingot {:.0}", n.store_ore, n.store_ingot),
            BuildingKind::Splitter => String::new(),
            BuildingKind::Totem => {
                if n.powered {
                    "Sheltering".into()
                } else {
                    "No power".into()
                }
            }
            BuildingKind::Turret => {
                if n.powered {
                    "Armed".into()
                } else {
                    "No power".into()
                }
            }
            BuildingKind::PowerWire | BuildingKind::Conveyor => String::new(),
        };
        if !body.is_empty() {
            let fs = (11.0 * cam.zoom).clamp(9.0, 13.0);
            draw_text(
                &body,
                sx + w * 0.5 - measure_text(&body, None, fs as u16, 1.0).width * 0.5,
                sy + h + fs * 2.2,
                fs,
                TEXT_DIM,
            );
        }
    }

    if damaged {
        let bar_w = w * 0.7;
        let bar_h = (3.5 * cam.zoom).max(2.5);
        let bar_x = sx + (w - bar_w) * 0.5;
        let bar_y = sy + h + 2.0;
        let pct = (n.hp / n.max_hp).clamp(0.0, 1.0);
        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::from_rgba(20, 24, 30, 180));
        draw_rectangle(
            bar_x,
            bar_y,
            bar_w * pct,
            bar_h,
            if pct > 0.45 {
                Color::from_rgba(220, 170, 60, 230)
            } else {
                Color::from_rgba(220, 70, 70, 230)
            },
        );
    }

    let show_ports =
        connect_tool.is_some() || lod == NodeLod::Near || (lod == NodeLod::Mid && hovered);
    if show_ports {
        for (pi, p) in n.ports.iter().enumerate() {
            let relevant = match connect_tool {
                Some(BuildingKind::PowerWire) => p.kind.is_energy(),
                Some(BuildingKind::Conveyor) => !p.kind.is_energy(),
                _ => true,
            };
            if connect_tool.is_some() && !relevant {
                continue;
            }
            let (px, py) = cam.world_to_screen(n.x + p.ox, n.y + p.oy);
            let r = (7.0 * cam.zoom).clamp(5.0, 11.0);
            let selected = wire_from == Some((node_id, pi));
            let valid_target = if let (Some(from), Some(tool)) = (wire_from, connect_tool) {
                if from == (node_id, pi) {
                    false
                } else {
                    match tool {
                        BuildingKind::PowerWire => world.can_connect_power(from, (node_id, pi)),
                        BuildingKind::Conveyor => world.can_connect_belt(from, (node_id, pi)),
                        _ => false,
                    }
                }
            } else {
                false
            };
            draw_port_icon(
                px,
                py,
                r,
                p,
                n,
                selected,
                valid_target,
                connect_tool.is_some(),
            );
        }
    }
}

/// Outward normal from building center through the port (for chevron direction).
fn port_outward(n: &Node, p: &Port) -> (f32, f32) {
    let dx = p.ox - n.w() * 0.5;
    let dy = p.oy - n.h() * 0.5;
    if dx.abs() >= dy.abs() {
        (if dx >= 0.0 { 1.0 } else { -1.0 }, 0.0)
    } else {
        (0.0, if dy >= 0.0 { 1.0 } else { -1.0 })
    }
}

fn draw_port_icon(
    px: f32,
    py: f32,
    r: f32,
    p: &Port,
    n: &Node,
    selected: bool,
    valid_target: bool,
    connect_mode: bool,
) {
    let (ox, oy) = port_outward(n, p);
    let energy = p.kind.is_energy();
    let base = if energy { POWER_C } else { CYAN };
    let color = if selected {
        Color::from_rgba(255, 255, 200, 255)
    } else if valid_target {
        Color::from_rgba(110, 255, 140, 255)
    } else {
        with_alpha(base, if connect_mode { 0.95 } else { 0.85 })
    };

    if p.kind.is_bidirectional() {
        // Diamond = bidirectional energy socket.
        let d = r * 1.05;
        let pts = [(px, py - d), (px + d, py), (px, py + d), (px - d, py)];
        draw_poly_fill(&pts, Color::from_rgba(20, 26, 34, 230));
        draw_poly_outline(&pts, 2.0, color);
        draw_circle(px, py, r * 0.28, color);
    } else if p.kind.is_output() {
        // Filled chevron pointing OUT of the building.
        draw_circle(px, py, r * 0.55, Color::from_rgba(20, 26, 34, 220));
        draw_chevron(px, py, ox, oy, r * 1.15, color);
        if connect_mode && !energy {
            draw_text("OUT", px - 10.0, py - r - 2.0, 10.0, with_alpha(color, 0.9));
        }
    } else if p.kind.is_input() {
        // Hollow socket = INPUT (items flow in).
        let s = r * 1.05;
        draw_rectangle(
            px - s,
            py - s,
            s * 2.0,
            s * 2.0,
            Color::from_rgba(20, 26, 34, 230),
        );
        draw_rectangle_lines(px - s, py - s, s * 2.0, s * 2.0, 2.2, color);
        draw_chevron(px, py, -ox, -oy, r * 0.7, with_alpha(color, 0.85));
        if connect_mode && !energy {
            draw_text("IN", px - 7.0, py - r - 2.0, 10.0, with_alpha(color, 0.9));
        }
    }

    if selected {
        draw_circle_lines(px, py, r * 1.7, 1.5, Color::from_rgba(255, 255, 180, 200));
    } else if valid_target {
        draw_circle_lines(px, py, r * 1.55, 1.4, Color::from_rgba(110, 255, 140, 180));
    }
}

fn draw_nests_and_raiders(world: &World, cam: &Cam, storm: &Storm) {
    let zones = storm.clear_zones(world);
    for nest in &world.nests {
        // Hidden in the storm until a clear zone reveals them.
        if !storm.in_clear(nest.x, nest.y, &zones) {
            continue;
        }
        let (sx, sy) = cam.world_to_screen(nest.x, nest.y);
        let r = NEST_RADIUS * cam.zoom;
        let fill = if nest.active {
            Color::from_rgba(160, 50, 70, 200)
        } else {
            Color::from_rgba(70, 55, 65, 160)
        };
        draw_circle(sx, sy, r, fill);
        // Wind-up ring: nest is about to launch a swarm.
        if nest.active && nest.wave_cd > 0.0 && nest.wave_cd <= NEST_REVEAL_WINDUP + 0.5 {
            let pulse = 1.0 - (nest.wave_cd / (NEST_REVEAL_WINDUP + 0.5)).clamp(0.0, 1.0);
            draw_circle_lines(
                sx,
                sy,
                r * (1.3 + pulse * 0.8),
                2.0,
                Color::from_rgba(255, 80, 80, (80.0 + pulse * 140.0) as u8),
            );
        }
        draw_circle_lines(
            sx,
            sy,
            r,
            1.5,
            if nest.active {
                Color::from_rgba(255, 90, 110, 220)
            } else {
                Color::from_rgba(120, 100, 110, 140)
            },
        );
        // Simple "nest" lobes.
        draw_circle(sx - r * 0.45, sy - r * 0.2, r * 0.4, fill);
        draw_circle(sx + r * 0.4, sy - r * 0.15, r * 0.35, fill);
        if nest.hp < nest.max_hp - 0.5 {
            let bar_w = r * 1.6;
            let pct = (nest.hp / nest.max_hp).clamp(0.0, 1.0);
            draw_rectangle(
                sx - bar_w * 0.5,
                sy + r + 4.0 * cam.zoom,
                bar_w,
                4.0 * cam.zoom.max(0.7),
                Color::from_rgba(30, 30, 36, 220),
            );
            draw_rectangle(
                sx - bar_w * 0.5,
                sy + r + 4.0 * cam.zoom,
                bar_w * pct,
                4.0 * cam.zoom.max(0.7),
                Color::from_rgba(220, 80, 90, 255),
            );
        }
        if nest.active && cam.zoom > 0.45 {
            draw_text(
                "NEST",
                sx - 16.0 * cam.zoom,
                sy - r - 6.0 * cam.zoom,
                (12.0 * cam.zoom).clamp(9.0, 14.0),
                Color::from_rgba(255, 140, 150, 230),
            );
        }
    }
    for raider in &world.raiders {
        let (sx, sy) = cam.world_to_screen(raider.x, raider.y);
        let (fill, rim) = match raider.role {
            RaiderRole::Assault => (
                Color::from_rgba(210, 60, 80, 230),
                Color::from_rgba(255, 120, 130, 255),
            ),
            RaiderRole::Hunter => (
                Color::from_rgba(230, 120, 50, 230),
                Color::from_rgba(255, 180, 90, 255),
            ),
            RaiderRole::Saboteur => (
                Color::from_rgba(160, 80, 220, 230),
                Color::from_rgba(210, 150, 255, 255),
            ),
            RaiderRole::Fogcaller => (
                Color::from_rgba(70, 90, 120, 240),
                Color::from_rgba(140, 180, 220, 255),
            ),
        };
        let scale = if raider.role == RaiderRole::Fogcaller {
            1.25
        } else {
            1.0
        };
        let r = RAIDER_RADIUS * cam.zoom * scale;
        draw_ellipse(
            sx,
            sy + r * 0.55,
            r * 1.05,
            r * 0.4,
            0.0,
            Color::from_rgba(0, 0, 0, 50),
        );
        draw_circle(sx, sy, r * 1.15, with_alpha(rim, 0.18));
        draw_circle(sx, sy, r, fill);
        draw_circle(sx - r * 0.2, sy - r * 0.25, r * 0.45, with_alpha(rim, 0.35));
        draw_circle_lines(sx, sy, r, 2.0, with_alpha(rim, 0.45));
        draw_circle_lines(sx, sy, r, 1.1, rim);
        draw_circle(sx, sy - r * 0.55, r * 0.35, rim);
        draw_circle(sx - r * 0.08, sy - r * 0.62, r * 0.12, Color::from_rgba(255, 255, 255, 90));
    }
}

fn draw_storm_blots(world: &World, cam: &Cam) {
    for blot in &world.storm_blots {
        let (sx, sy) = cam.world_to_screen(blot.x, blot.y);
        let r = blot.radius * cam.zoom;
        let a = (blot.life / FOG_BLOT_LIFE).clamp(0.15, 1.0);
        draw_circle(
            sx,
            sy,
            r,
            Color::from_rgba(40, 55, 80, (55.0 * a) as u8),
        );
        draw_circle_lines(
            sx,
            sy,
            r,
            2.0,
            Color::from_rgba(120, 160, 220, (160.0 * a) as u8),
        );
        draw_circle(
            sx,
            sy,
            r * 0.35,
            Color::from_rgba(90, 120, 180, (90.0 * a) as u8),
        );
    }
}

fn draw_combat_shots(world: &World, cam: &Cam) {
    for shot in &world.combat_shots {
        let (sx0, sy0) = cam.world_to_screen(shot.x0, shot.y0);
        let (sx1, sy1) = cam.world_to_screen(shot.x1, shot.y1);
        let a = (shot.life / 0.12).clamp(0.0, 1.0);
        let core = (2.0 * cam.zoom).max(1.4);
        draw_line(
            sx0,
            sy0,
            sx1,
            sy1,
            core * 3.0,
            Color::from_rgba(255, 180, 80, (40.0 * a) as u8),
        );
        draw_line(
            sx0,
            sy0,
            sx1,
            sy1,
            core * 1.6,
            Color::from_rgba(255, 210, 120, (120.0 * a) as u8),
        );
        draw_line(
            sx0,
            sy0,
            sx1,
            sy1,
            core,
            Color::from_rgba(255, 240, 200, (220.0 * a) as u8),
        );
        draw_circle(
            sx1,
            sy1,
            (4.5 * cam.zoom).max(2.5),
            Color::from_rgba(255, 200, 100, (90.0 * a) as u8),
        );
        draw_circle(
            sx1,
            sy1,
            (2.6 * cam.zoom).max(1.8),
            Color::from_rgba(255, 240, 200, (200.0 * a) as u8),
        );
    }
}

fn draw_placement_ghost(world: &World, ui: &Ui, cam: &Cam, storm: &Storm, wx: f32, wy: f32) {
    if ui.build_open || ui.drag_node.is_some() {
        return;
    }
    let Some(kind) = ui.selected else {
        return;
    };
    // Wire / conveyor tools: cursor hint instead of a building ghost.
    if kind.is_cable() {
        if ui.wire_from.is_some() {
            return; // rubber-band drawn by link renderers
        }
        let (sx, sy) = cam.world_to_screen(wx, wy);
        draw_text(
            "Wire — OUT ▶ or ◆ socket  →  ◆ socket",
            sx + 14.0,
            sy - 8.0,
            16.0,
            POWER_C,
        );
        return;
    }
    if kind.is_belt_tool() {
        let (sx, sy) = cam.world_to_screen(wx, wy);
        draw_text(
            "Belt — drag to paint · R rotate · RMB erase",
            sx + 14.0,
            sy - 8.0,
            16.0,
            BELT_YELLOW,
        );
        return;
    }
    if ui.wire_from.is_some() {
        return;
    }
    let probe = Node::new(kind, 0.0, 0.0, ui.place_facing);
    let (x, y) = snap_building_xy(kind, ui.place_facing, wx, wy);
    let blocked = world.collides(x, y, probe.w(), probe.h(), None);
    let in_storm = storm.point_in_storm(x + probe.w() * 0.5, y + probe.h() * 0.5, world);
    let (sx, sy) = cam.world_to_screen(x, y);
    let w = probe.w() * cam.zoom;
    let h = probe.h() * cam.zoom;
    let fill = if blocked {
        Color::from_rgba(180, 50, 50, 90)
    } else if in_storm {
        Color::from_rgba(255, 160, 60, 70)
    } else {
        Color::from_rgba(30, 50, 60, 100)
    };
    let outline = if blocked {
        Color::from_rgba(220, 80, 80, 200)
    } else if in_storm {
        Color::from_rgba(255, 170, 70, 220)
    } else {
        CYAN
    };
    let accent = kind_swatch(kind);
    draw_building_silhouette(
        kind,
        ui.place_facing,
        sx,
        sy,
        w,
        h,
        fill,
        outline,
        accent,
        !blocked,
        true,
    );
    let label = if blocked {
        kind.label().to_string()
    } else if in_storm {
        format!("{} · storm!", kind.short())
    } else {
        kind.label().to_string()
    };
    draw_text(
        &label,
        sx + w * 0.5 - measure_text(&label, None, ((14.0 * cam.zoom).clamp(10.0, 16.0)) as u16, 1.0).width * 0.5,
        sy + h + 16.0,
        (14.0 * cam.zoom).clamp(10.0, 16.0),
        if blocked {
            Color::from_rgba(255, 160, 160, 255)
        } else if in_storm {
            ACCENT
        } else {
            CYAN
        },
    );
    if kind == BuildingKind::PowerPole {
        let (cx, cy) = cam.world_to_screen(x + probe.w() * 0.5, y + probe.h() * 0.5);
        draw_circle_lines(cx, cy, POLE_RADIUS * cam.zoom, 1.0, POWER_DIM);
    }
    if kind == BuildingKind::Totem {
        let (cx, cy) = cam.world_to_screen(x + probe.w() * 0.5, y + probe.h() * 0.5);
        draw_circle_lines(
            cx,
            cy,
            TOTEM_CLEAR_RADIUS * STORM_HARD_CLEAR_SCALE * cam.zoom,
            1.2,
            Color::from_rgba(140, 100, 220, 120),
        );
    }
    if kind == BuildingKind::Turret {
        let (cx, cy) = cam.world_to_screen(x + probe.w() * 0.5, y + probe.h() * 0.5);
        draw_circle_lines(cx, cy, TURRET_RANGE * cam.zoom, 1.0, Color::from_rgba(200, 90, 90, 100));
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
        // Remote drone (other players only — never the local one).
        player::draw_drone_remote(
            &peer.drone,
            app.cam.x,
            app.cam.y,
            app.cam.zoom,
            color,
            Some(&peer_label(peer.id)),
        );
        // Mouse cursor marker matching drone color.
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
    let pad = s(14.0);
    let sidebar_w = s(148.0);
    let detail_h = s(64.0);
    let search_h = s(36.0);
    let header_h = s(56.0);

    draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, NODE_BORDER);
    draw_text("Build", r.x + pad, r.y + s(28.0), s(26.0), TEXT);
    draw_text(
        "Click equip · drag to hotbar · 1–9 pin",
        r.x + s(100.0),
        r.y + s(26.0),
        s(14.0),
        TEXT_DIM,
    );

    // --- Left category sidebar ---
    let side_x = r.x + pad;
    let side_y = r.y + header_h;
    let side_h = r.h - header_h - pad;
    draw_rectangle(
        side_x,
        side_y,
        sidebar_w,
        side_h,
        Color::from_rgba(22, 26, 32, 255),
    );

    let cat_row_h = s(36.0);
    // All + existing categories
    let all_label = "All";
    let cat_count = 1 + BuildCategory::ALL.len();
    for i in 0..cat_count {
        let (cat, label): (Option<BuildCategory>, &str) = if i == 0 {
            (None, all_label)
        } else {
            let c = BuildCategory::ALL[i - 1];
            (Some(c), c.label())
        };
        let y = side_y + s(8.0) + i as f32 * (cat_row_h + s(4.0));
        let row = Rect {
            x: side_x + s(6.0),
            y,
            w: sidebar_w - s(12.0),
            h: cat_row_h,
        };
        let active = app.ui.build_category == cat;
        let hovered = mouse.0 >= row.x
            && mouse.0 <= row.x + row.w
            && mouse.1 >= row.y
            && mouse.1 <= row.y + row.h;
        draw_rectangle(
            row.x,
            row.y,
            row.w,
            row.h,
            if active {
                Color::from_rgba(48, 58, 72, 255)
            } else if hovered {
                Color::from_rgba(36, 42, 52, 255)
            } else {
                Color::from_rgba(26, 30, 38, 255)
            },
        );
        if active {
            draw_rectangle(row.x, row.y, s(3.0), row.h, CYAN);
        }
        draw_text(
            label,
            row.x + s(14.0),
            row.y + s(24.0),
            s(16.0),
            if active { CYAN } else { TEXT },
        );
        if hovered && is_mouse_button_pressed(MouseButton::Left) && app.ui.palette_drag.is_none() {
            app.ui.build_category = cat;
            app.ui.build_scroll = 0.0;
        }
    }

    // --- Content area (search + grid + detail) ---
    let content_x = side_x + sidebar_w + pad;
    let content_w = r.w - sidebar_w - pad * 3.0;
    let content_top = side_y;

    // Search field
    let search = Rect {
        x: content_x,
        y: content_top,
        w: content_w,
        h: search_h,
    };
    let search_hovered = mouse.0 >= search.x
        && mouse.0 <= search.x + search.w
        && mouse.1 >= search.y
        && mouse.1 <= search.y + search.h;
    if search_hovered && is_mouse_button_pressed(MouseButton::Left) {
        app.ui.build_search_focus = true;
    }
    draw_rectangle(
        search.x,
        search.y,
        search.w,
        search.h,
        Color::from_rgba(16, 18, 24, 255),
    );
    draw_rectangle_lines(
        search.x,
        search.y,
        search.w,
        search.h,
        1.2,
        if app.ui.build_search_focus {
            CYAN
        } else {
            NODE_BORDER
        },
    );
    let search_label = if app.ui.build_search.is_empty() {
        if app.ui.build_search_focus {
            "Type to filter…"
        } else {
            "Search buildings…"
        }
    } else {
        app.ui.build_search.as_str()
    };
    draw_text(
        search_label,
        search.x + s(12.0),
        search.y + s(24.0),
        s(16.0),
        if app.ui.build_search.is_empty() {
            TEXT_DIM
        } else {
            TEXT
        },
    );

    // Detail strip at bottom of content
    let detail = Rect {
        x: content_x,
        y: r.y + r.h - pad - detail_h,
        w: content_w,
        h: detail_h,
    };

    // Icon grid between search and detail
    let grid = Rect {
        x: content_x,
        y: search.y + search.h + s(10.0),
        w: content_w,
        h: detail.y - (search.y + search.h + s(10.0)) - s(8.0),
    };
    draw_rectangle(
        grid.x,
        grid.y,
        grid.w,
        grid.h,
        Color::from_rgba(20, 24, 30, 255),
    );

    let cell = s(72.0);
    let gap = s(8.0);
    let cols = ((grid.w - gap) / (cell + gap)).floor().max(1.0) as usize;
    let row_stride = cell + gap;
    let items = app.ui.filtered_buildings();
    let rows = if items.is_empty() {
        0
    } else {
        (items.len() + cols - 1) / cols
    };
    let content_h = rows as f32 * row_stride + gap;
    let max_scroll = (content_h - grid.h).max(0.0);

    let over_grid = mouse.0 >= grid.x
        && mouse.0 <= grid.x + grid.w
        && mouse.1 >= grid.y
        && mouse.1 <= grid.y + grid.h;
    if over_grid {
        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            app.ui.build_scroll = (app.ui.build_scroll - wheel * row_stride).clamp(0.0, max_scroll);
        }
    }
    app.ui.build_scroll = app.ui.build_scroll.clamp(0.0, max_scroll);

    let mut detail_kind: Option<BuildingKind> =
        app.ui.palette_drag.or(app.ui.selected).filter(|k| items.contains(k));

    if items.is_empty() {
        draw_text(
            "No buildings match",
            grid.x + s(16.0),
            grid.y + s(32.0),
            s(18.0),
            TEXT_DIM,
        );
    } else {
        let first_row = (app.ui.build_scroll / row_stride).floor().max(0.0) as usize;
        let visible_rows = ((grid.h / row_stride).ceil() as usize) + 1;
        let last_row = (first_row + visible_rows).min(rows);

        for row in first_row..last_row {
            for col in 0..cols {
                let idx = row * cols + col;
                if idx >= items.len() {
                    break;
                }
                let kind = items[idx];
                let x = grid.x + gap + col as f32 * (cell + gap);
                let y = grid.y + gap + row as f32 * row_stride - app.ui.build_scroll;
                if y + cell < grid.y || y > grid.y + grid.h {
                    continue;
                }
                let hovered = mouse.0 >= x
                    && mouse.0 <= x + cell
                    && mouse.1 >= y
                    && mouse.1 <= y + cell
                    && over_grid;
                let selected =
                    app.ui.selected == Some(kind) || app.ui.palette_drag == Some(kind);
                if hovered {
                    detail_kind = Some(kind);
                }
                // Clip-ish: only draw if mostly inside grid
                draw_rectangle(
                    x,
                    y,
                    cell,
                    cell,
                    if hovered || selected {
                        Color::from_rgba(44, 54, 68, 255)
                    } else {
                        Color::from_rgba(28, 32, 40, 255)
                    },
                );
                draw_rectangle_lines(
                    x,
                    y,
                    cell,
                    cell,
                    1.2,
                    if selected {
                        CYAN
                    } else if hovered {
                        Color::from_rgba(90, 110, 130, 255)
                    } else {
                        NODE_BORDER
                    },
                );
                let sw = s(28.0);
                draw_rectangle(
                    x + (cell - sw) * 0.5,
                    y + s(12.0),
                    sw,
                    s(20.0),
                    kind_swatch(kind),
                );
                let short = kind.short();
                let tw = measure_text(short, None, s(14.0) as u16, 1.0).width;
                draw_text(
                    short,
                    x + (cell - tw) * 0.5,
                    y + cell - s(12.0),
                    s(14.0),
                    TEXT,
                );
                if hovered
                    && is_mouse_button_pressed(MouseButton::Left)
                    && app.ui.palette_drag.is_none()
                {
                    app.ui.palette_drag = Some(kind);
                    app.ui.palette_drag_origin = mouse;
                    app.ui.selected = Some(kind);
                    app.ui.context_menu = None;
                }
            }
        }

        // Scrollbar when needed
        if max_scroll > 1.0 {
            let bar_w = s(5.0);
            let track_h = grid.h - s(8.0);
            let thumb_h = (track_h * (grid.h / content_h)).clamp(s(24.0), track_h);
            let thumb_y = grid.y
                + s(4.0)
                + (track_h - thumb_h) * (app.ui.build_scroll / max_scroll);
            draw_rectangle(
                grid.x + grid.w - bar_w - s(4.0),
                thumb_y,
                bar_w,
                thumb_h,
                Color::from_rgba(70, 82, 98, 220),
            );
        }
    }

    // Detail strip
    draw_rectangle(
        detail.x,
        detail.y,
        detail.w,
        detail.h,
        Color::from_rgba(24, 28, 36, 255),
    );
    draw_rectangle_lines(detail.x, detail.y, detail.w, detail.h, 1.0, NODE_BORDER);
    if let Some(kind) = detail_kind {
        draw_rectangle(
            detail.x + s(14.0),
            detail.y + s(18.0),
            s(28.0),
            s(20.0),
            kind_swatch(kind),
        );
        draw_text(
            kind.label(),
            detail.x + s(54.0),
            detail.y + s(24.0),
            s(18.0),
            TEXT,
        );
        draw_text(
            kind.hint(),
            detail.x + s(54.0),
            detail.y + s(46.0),
            s(14.0),
            TEXT_DIM,
        );
    } else {
        draw_text(
            "Select a building",
            detail.x + s(16.0),
            detail.y + s(36.0),
            s(16.0),
            TEXT_DIM,
        );
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
        app.ui.close_build();
    }
}
