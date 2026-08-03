//! Buildings, power wires, and Factorio-style grid belts.

use std::collections::{HashMap, HashSet, VecDeque};

pub use crate::belts::{snap_building_xy, tile_origin, world_to_tile, BeltGrid, TILE_SIZE};

pub const ORE_RATE: f32 = 7.3;
pub const SMELT_RATE: f32 = 3.9;
pub const SOLAR_POWER: f32 = 12.0;
pub const ORE_POWER_DRAW: f32 = 4.0;
pub const SMELT_POWER_DRAW: f32 = 8.0;
pub const TURRET_POWER_DRAW: f32 = 5.0;
pub const NODE_BUFFER: f32 = 100.0;
pub const POLE_RADIUS: f32 = 260.0;
/// Max Manhattan reach for a power wire — place poles for longer runs.
pub const POWER_WIRE_MAX_REACH: f32 = 420.0;
/// Factorio-style copper budget per energy socket.
pub const MAX_POWER_LINKS_PER_PORT: usize = 5;
pub const TOTEM_CLEAR_RADIUS: f32 = 1000.0;
/// Accent-only storm damage — nests/raids are the real pressure.
pub const LIGHTNING_DAMAGE: f32 = 16.0;

pub const NEST_HP: f32 = 220.0;
pub const NEST_RADIUS: f32 = 36.0;
pub const NEST_COUNT_DEFAULT: usize = 10;
/// Base seconds between attack waves (shrinks with evolution).
pub const NEST_WAVE_INTERVAL: f32 = 28.0;
pub const NEST_WAVE_MIN_INTERVAL: f32 = 12.0;
pub const RAIDER_HP: f32 = 40.0;
pub const RAIDER_RADIUS: f32 = 12.0;
pub const RAIDER_SPEED: f32 = 78.0;
pub const RAIDER_DAMAGE: f32 = 14.0;
pub const RAIDER_ATTACK_RANGE: f32 = 28.0;
pub const RAIDER_ATTACK_INTERVAL: f32 = 0.7;
pub const MAX_RAIDERS: usize = 80;
/// Flocking: stay near allies / don't stack.
pub const SWARM_SEP_RADIUS: f32 = 28.0;
pub const SWARM_COH_RADIUS: f32 = 120.0;
/// Hunters / saboteurs only chase specialty targets inside this radius.
pub const ROLE_SEEK_RANGE: f32 = 560.0;
/// How often raiders re-evaluate nearest / role targets.
pub const RETARGET_INTERVAL: f32 = 0.5;
/// Warning time after a nest is revealed before the first swarm.
pub const NEST_REVEAL_WINDUP: f32 = 3.5;
/// Clear within this distance of a nest still builds breach scent.
pub const BREACH_PROXIMITY: f32 = 480.0;
/// Extra score weight: prefer targets near the clear rim.
pub const RIM_TARGET_WEIGHT: f32 = 2.4;
pub const FOG_BLOT_RADIUS: f32 = 95.0;
pub const FOG_BLOT_LIFE: f32 = 4.5;
pub const FOG_BLOT_DAMAGE: f32 = 8.0;
pub const FOG_BLOT_TICK: f32 = 0.55;
pub const TURRET_RANGE: f32 = 340.0;
pub const TURRET_FIRE_INTERVAL: f32 = 0.4;
pub const TURRET_DAMAGE: f32 = 18.0;
/// Hard clear scale must match main.rs STORM_HARD_CLEAR_SCALE for nest activation.
pub const CLEAR_HARD_SCALE: f32 = 0.72;
/// Click / collision half-width for physical conveyor corridors.
pub const WIRE_HIT_HALF: f32 = 10.0;

pub fn building_max_hp(kind: BuildingKind) -> f32 {
    match kind {
        BuildingKind::Totem => 160.0,
        BuildingKind::PowerPole => 70.0,
        BuildingKind::Solar => 90.0,
        BuildingKind::Turret => 130.0,
        BuildingKind::PowerWire => 40.0,
        BuildingKind::Conveyor => 55.0,
        _ => 110.0,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Item {
    IronOre,
    IronIngot,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Facing {
    E,
    S,
    W,
    N,
}

impl Facing {
    pub fn rotate_cw(self) -> Self {
        match self {
            Facing::E => Facing::S,
            Facing::S => Facing::W,
            Facing::W => Facing::N,
            Facing::N => Facing::E,
        }
    }
    pub fn as_u8(self) -> u8 {
        match self {
            Facing::E => 0,
            Facing::S => 1,
            Facing::W => 2,
            Facing::N => 3,
        }
    }
    pub fn from_u8(v: u8) -> Self {
        match v % 4 {
            1 => Facing::S,
            2 => Facing::W,
            3 => Facing::N,
            _ => Facing::E,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuildCategory {
    Energy,
    Resource,
    Processing,
    Storage,
    Transport,
    Defense,
}

impl BuildCategory {
    pub const ALL: [BuildCategory; 6] = [
        Self::Energy,
        Self::Resource,
        Self::Processing,
        Self::Storage,
        Self::Transport,
        Self::Defense,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Energy => "Energy",
            Self::Resource => "Resource",
            Self::Processing => "Processing",
            Self::Storage => "Storage",
            Self::Transport => "Transport",
            Self::Defense => "Defense",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuildingKind {
    Solar,
    PowerPole,
    OreNode,
    Smelter,
    Box,
    Splitter,
    Totem,
    Turret,
    /// Physical power cable between ports — connection tool in the build menu.
    PowerWire,
    /// Physical conveyor between ports — connection tool in the build menu.
    Conveyor,
}

impl BuildingKind {
    pub fn category(self) -> BuildCategory {
        match self {
            Self::Solar | Self::PowerPole | Self::Totem | Self::PowerWire => BuildCategory::Energy,
            Self::OreNode => BuildCategory::Resource,
            Self::Smelter => BuildCategory::Processing,
            Self::Box => BuildCategory::Storage,
            Self::Splitter | Self::Conveyor => BuildCategory::Transport,
            Self::Turret => BuildCategory::Defense,
        }
    }
    pub fn in_category(cat: BuildCategory) -> Vec<BuildingKind> {
        Self::ALL
            .into_iter()
            .filter(|k| k.category() == cat)
            .collect()
    }

    /// Placeable buildings + connection tools shown in the build menu / hotbar.
    pub const ALL: [BuildingKind; 10] = [
        Self::Solar,
        Self::PowerPole,
        Self::OreNode,
        Self::Smelter,
        Self::Box,
        Self::Splitter,
        Self::Totem,
        Self::Turret,
        Self::PowerWire,
        Self::Conveyor,
    ];

    pub fn all() -> &'static [BuildingKind] {
        &Self::ALL
    }

    /// Connection tool: selected to link energy ports (not placed as a ground building).
    pub fn is_cable(self) -> bool {
        matches!(self, Self::PowerWire)
    }

    /// Placeable belt tiles (Factorio-style), painted on the grid.
    pub fn is_belt_tool(self) -> bool {
        matches!(self, Self::Conveyor)
    }

    /// Case-insensitive substring match against full and short names.
    pub fn matches_query(self, query: &str) -> bool {
        let q = query.trim();
        if q.is_empty() {
            return true;
        }
        let q = q.to_ascii_lowercase();
        self.label().to_ascii_lowercase().contains(&q)
            || self.short().to_ascii_lowercase().contains(&q)
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Solar => "Generates power for the grid.",
            Self::PowerPole => "Distributes power in a radius.",
            Self::OreNode => "Mines ore. Requires power.",
            Self::Smelter => "Smelts ore into ingots. Requires power.",
            Self::Box => "Stores items from belts.",
            Self::Splitter => "Belt splitter — 3 wide. Alternates output evenly.",
            Self::Totem => "Powered clear zone — shelters builds and reveals nests.",
            Self::Turret => "Auto-fires at raiders and nests. Requires power.",
            Self::PowerWire => "Select, then click two energy ports to connect.",
            Self::Conveyor => "Drag to paint belt tiles. R rotates. Loops sideload to change lanes.",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Solar => "Solar Panel",
            Self::PowerPole => "Power Pole",
            Self::OreNode => "Iron Ore Node",
            Self::Smelter => "Smelter",
            Self::Box => "Storage Box",
            Self::Splitter => "Splitter",
            Self::Totem => "Storm Totem",
            Self::Turret => "Gun Turret",
            Self::PowerWire => "Power Wire",
            Self::Conveyor => "Conveyor",
        }
    }
    pub fn short(self) -> &'static str {
        match self {
            Self::Solar => "Solar",
            Self::PowerPole => "Pole",
            Self::OreNode => "Ore",
            Self::Smelter => "Smelt",
            Self::Box => "Box",
            Self::Splitter => "Split",
            Self::Totem => "Totem",
            Self::Turret => "Turret",
            Self::PowerWire => "Wire",
            Self::Conveyor => "Belt",
        }
    }
    pub fn size(self) -> (f32, f32) {
        // Footprints are tile multiples (TILE_SIZE=40) and match silhouettes.
        match self {
            Self::PowerPole => (40.0, 80.0),
            Self::Splitter => (40.0, 120.0), // 1 deep × 3 wide (rotated with facing)
            Self::Totem => (80.0, 120.0),
            Self::Turret => (80.0, 80.0),
            Self::Solar => (160.0, 120.0),
            Self::OreNode => (120.0, 120.0),
            Self::Smelter => (160.0, 120.0),
            Self::Box => (120.0, 120.0),
            // Wire is a link tool; conveyor is a tile brush (no building AABB).
            Self::PowerWire | Self::Conveyor => (40.0, 40.0),
        }
    }
    pub fn needs_power(self) -> bool {
        matches!(self, Self::OreNode | Self::Smelter | Self::Totem | Self::Turret)
    }
    pub fn can_rotate(self) -> bool {
        !matches!(
            self,
            Self::PowerPole | Self::Totem | Self::Turret | Self::PowerWire | Self::Conveyor
        )
    }
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Solar => 0,
            Self::PowerPole => 1,
            Self::OreNode => 2,
            Self::Smelter => 3,
            Self::Box => 4,
            Self::Splitter => 5,
            Self::Totem => 6,
            Self::Turret => 7,
            Self::PowerWire => 8,
            Self::Conveyor => 9,
        }
    }
    pub fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            0 => Self::Solar,
            1 => Self::PowerPole,
            2 => Self::OreNode,
            3 => Self::Smelter,
            4 => Self::Box,
            5 => Self::Splitter,
            6 => Self::Totem,
            7 => Self::Turret,
            8 => Self::PowerWire,
            9 => Self::Conveyor,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PortKind {
    EnergyOut,
    EnergyAny,
    ItemOut(Item),
    ItemIn(Item),
    AnyIn,
    AnyOut,
}

impl PortKind {
    pub fn is_energy(self) -> bool {
        matches!(self, Self::EnergyOut | Self::EnergyAny)
    }
    /// True output (pushes power/items).
    pub fn is_output(self) -> bool {
        matches!(self, Self::EnergyOut | Self::ItemOut(_) | Self::AnyOut)
    }
    /// True input (receives).
    pub fn is_input(self) -> bool {
        matches!(self, Self::ItemIn(_) | Self::AnyIn)
    }
    /// Bidirectional energy socket (poles / totems / turrets).
    pub fn is_bidirectional(self) -> bool {
        matches!(self, Self::EnergyAny)
    }
}

#[derive(Clone, Debug)]
pub struct Port {
    pub kind: PortKind,
    pub ox: f32,
    pub oy: f32,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: BuildingKind,
    pub x: f32,
    pub y: f32,
    pub facing: Facing,
    pub in_ore: f32,
    pub out_ore: f32,
    pub out_ingot: f32,
    pub store_ore: f32,
    pub store_ingot: f32,
    pub buf_ore: f32,
    pub buf_ingot: f32,
    /// Splitter: whole items waiting per belt lane (0 = left). Sums match buf_*.
    pub split_ore: [u16; 2],
    pub split_ingot: [u16; 2],
    /// Splitter: next output side (0/1) per input lane for even balancing.
    pub split_side: [u8; 2],
    pub working: bool,
    pub powered: bool,
    pub hp: f32,
    pub max_hp: f32,
    /// Turret fire cooldown (seconds).
    pub cooldown: f32,
    /// Cable endpoint ports (PowerWire / Conveyor only).
    pub cable_a: Option<(u32, usize)>,
    pub cable_b: Option<(u32, usize)>,
    pub ports: Vec<Port>,
}

impl Node {
    pub fn new(kind: BuildingKind, x: f32, y: f32, facing: Facing) -> Self {
        let max_hp = building_max_hp(kind);
        let mut n = Self {
            kind,
            x,
            y,
            facing,
            in_ore: 0.0,
            out_ore: 0.0,
            out_ingot: 0.0,
            store_ore: 0.0,
            store_ingot: 0.0,
            buf_ore: 0.0,
            buf_ingot: 0.0,
            split_ore: [0, 0],
            split_ingot: [0, 0],
            split_side: [0, 0],
            working: false,
            powered: false,
            hp: max_hp,
            max_hp,
            cooldown: 0.0,
            cable_a: None,
            cable_b: None,
            ports: Vec::new(),
        };
        n.rebuild_ports();
        n
    }
    pub fn rebuild_ports(&mut self) {
        let (w, h) = self.size();
        self.ports = ports_for(self.kind, w, h, self.facing);
    }
    pub fn size(&self) -> (f32, f32) {
        let (bw, bh) = self.kind.size();
        match self.facing {
            Facing::E | Facing::W => (bw, bh),
            Facing::N | Facing::S => (bh, bw),
        }
    }
    pub fn w(&self) -> f32 {
        self.size().0
    }
    pub fn h(&self) -> f32 {
        self.size().1
    }
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w() * 0.5, self.y + self.h() * 0.5)
    }
    pub fn contains(&self, wx: f32, wy: f32) -> bool {
        wx >= self.x && wy >= self.y && wx <= self.x + self.w() && wy <= self.y + self.h()
    }
    pub fn overlaps_rect(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        self.x < x + w && self.x + self.w() > x && self.y < y + h && self.y + self.h() > y
    }
    pub fn port_world(&self, i: usize) -> Option<(f32, f32)> {
        let p = self.ports.get(i)?;
        Some((self.x + p.ox, self.y + p.oy))
    }
    pub fn set_facing(&mut self, facing: Facing) {
        let (cx, cy) = self.center();
        self.facing = facing;
        let (w, h) = self.size();
        self.x = cx - w * 0.5;
        self.y = cy - h * 0.5;
        self.rebuild_ports();
    }
}

fn edge(w: f32, h: f32, side: Facing, along: f32) -> (f32, f32) {
    match side {
        Facing::W => (0.0, h * along),
        Facing::E => (w, h * along),
        Facing::N => (w * along, 0.0),
        Facing::S => (w * along, h),
    }
}

fn ports_for(kind: BuildingKind, w: f32, h: f32, facing: Facing) -> Vec<Port> {
    let back = match facing {
        Facing::E => Facing::W,
        Facing::S => Facing::N,
        Facing::W => Facing::E,
        Facing::N => Facing::S,
    };
    let m = 0.5;
    match kind {
        BuildingKind::Solar => {
            let (ox, oy) = edge(w, h, facing, m);
            vec![Port {
                kind: PortKind::EnergyOut,
                ox,
                oy,
            }]
        }
        BuildingKind::PowerPole => {
            let a = edge(w, h, back, m);
            let b = edge(w, h, facing, m);
            vec![
                Port {
                    kind: PortKind::EnergyAny,
                    ox: a.0,
                    oy: a.1,
                },
                Port {
                    kind: PortKind::EnergyAny,
                    ox: b.0,
                    oy: b.1,
                },
            ]
        }
        BuildingKind::OreNode => {
            let (ox, oy) = edge(w, h, facing, m);
            vec![Port {
                kind: PortKind::ItemOut(Item::IronOre),
                ox,
                oy,
            }]
        }
        BuildingKind::Smelter => {
            let i = edge(w, h, back, m);
            let o = edge(w, h, facing, m);
            vec![
                Port {
                    kind: PortKind::ItemIn(Item::IronOre),
                    ox: i.0,
                    oy: i.1,
                },
                Port {
                    kind: PortKind::ItemOut(Item::IronIngot),
                    ox: o.0,
                    oy: o.1,
                },
            ]
        }
        BuildingKind::Box => {
            let (ox, oy) = edge(w, h, back, m);
            vec![Port {
                kind: PortKind::AnyIn,
                ox,
                oy,
            }]
        }
        BuildingKind::Splitter => {
            // 3-wide belt: input on the back center, outputs on the front
            // left / right tile centers (Factorio-style dual out).
            let i = edge(w, h, back, 0.5);
            let o0 = edge(w, h, facing, 1.0 / 6.0);
            let o1 = edge(w, h, facing, 5.0 / 6.0);
            vec![
                Port {
                    kind: PortKind::AnyIn,
                    ox: i.0,
                    oy: i.1,
                },
                Port {
                    kind: PortKind::AnyOut,
                    ox: o0.0,
                    oy: o0.1,
                },
                Port {
                    kind: PortKind::AnyOut,
                    ox: o1.0,
                    oy: o1.1,
                },
            ]
        }
        BuildingKind::Totem | BuildingKind::Turret => {
            let (ox, oy) = edge(w, h, facing, m);
            vec![Port {
                kind: PortKind::EnergyAny,
                ox,
                oy,
            }]
        }
        BuildingKind::PowerWire | BuildingKind::Conveyor => Vec::new(),
    }
}

/// Manhattan path corners matching draw_power_manhattan (H → mid-X → V → H).
pub fn manhattan_corners(ax: f32, ay: f32, bx: f32, by: f32) -> [(f32, f32); 4] {
    let mx = (ax + bx) * 0.5;
    [(ax, ay), (mx, ay), (mx, by), (bx, by)]
}

fn dist_point_segment(px: f32, py: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len2 = dx * dx + dy * dy;
    if len2 < 1e-6 {
        return (px - x0).hypot(py - y0);
    }
    let t = ((px - x0) * dx + (py - y0) * dy) / len2;
    let t = t.clamp(0.0, 1.0);
    let cx = x0 + dx * t;
    let cy = y0 + dy * t;
    (px - cx).hypot(py - cy)
}

pub fn dist_to_manhattan(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let c = manhattan_corners(ax, ay, bx, by);
    let mut best = f32::MAX;
    for i in 0..3 {
        best = best.min(dist_point_segment(px, py, c[i].0, c[i].1, c[i + 1].0, c[i + 1].1));
    }
    best
}

#[derive(Clone, Debug)]
pub struct Nest {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub max_hp: f32,
    /// Becomes true when a hard clear zone overlaps the nest.
    pub active: bool,
    /// Countdown to next attack wave.
    pub wave_cd: f32,
    /// 0..=1 — grows while active; larger/faster waves.
    pub evolution: f32,
    /// Spikes when damaged or when factory pollution is nearby.
    pub anger: f32,
    /// Next launch is the big reveal swarm.
    pub first_wave: bool,
    /// Went dark after clear retracted — next reveal is nastier.
    pub dormant_hate: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RaiderRole {
    /// Swarm meat — nearest rim building.
    Assault,
    /// Prefer turrets / defenses in seek range, else nearest rim.
    Hunter,
    /// Prefer power (solar, poles, totems) in seek range, else nearest rim.
    Saboteur,
    /// Tanky — leaves temporary storm blots that damage buildings.
    Fogcaller,
}

impl RaiderRole {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Assault => 0,
            Self::Hunter => 1,
            Self::Saboteur => 2,
            Self::Fogcaller => 3,
        }
    }
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Hunter,
            2 => Self::Saboteur,
            3 => Self::Fogcaller,
            _ => Self::Assault,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Raider {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub target_node: Option<u32>,
    pub attack_cd: f32,
    /// Shared wave id for flocking with allies.
    pub wave_id: u32,
    pub vx: f32,
    pub vy: f32,
    pub role: RaiderRole,
    pub retarget_cd: f32,
}

#[derive(Clone, Debug)]
pub struct StormBlot {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub life: f32,
    pub tick_cd: f32,
}

#[derive(Clone, Debug, Default)]
pub struct CombatReport {
    pub destroyed: usize,
    pub nests_revealed: usize,
    pub nests_reawakened: usize,
    pub waves_launched: usize,
}

#[derive(Clone, Debug)]
pub struct CombatShot {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub life: f32,
}

fn point_in_hard_clear(x: f32, y: f32, zones: &[(f32, f32, f32)]) -> bool {
    clear_signed_depth(x, y, zones) >= 0.0
}

/// Signed depth into hard clear: >=0 inside (distance to edge), <0 outside (neg. dist to edge).
fn clear_signed_depth(x: f32, y: f32, zones: &[(f32, f32, f32)]) -> f32 {
    let mut best_inside = f32::NEG_INFINITY;
    let mut best_outside = f32::INFINITY;
    let mut any_zone = false;
    for &(cx, cy, radius) in zones {
        let r = radius * CLEAR_HARD_SCALE;
        if r < 1.0 {
            continue;
        }
        any_zone = true;
        let d = (x - cx).hypot(y - cy);
        if d <= r {
            best_inside = best_inside.max(r - d);
        } else {
            best_outside = best_outside.min(d - r);
        }
    }
    if !any_zone {
        return f32::NEG_INFINITY;
    }
    if best_inside >= 0.0 && best_inside > f32::NEG_INFINITY {
        best_inside
    } else if best_outside < f32::INFINITY {
        -best_outside
    } else {
        f32::NEG_INFINITY
    }
}

/// Absolute distance to the nearest clear boundary (0 = on the rim).
fn dist_to_clear_rim(x: f32, y: f32, zones: &[(f32, f32, f32)]) -> f32 {
    clear_signed_depth(x, y, zones).abs()
}

/// How hard the clear is pressing this nest (0..~2+).
fn breach_scent(nest_x: f32, nest_y: f32, zones: &[(f32, f32, f32)]) -> f32 {
    let depth = clear_signed_depth(nest_x, nest_y, zones);
    if depth >= 0.0 {
        // Deeper clear over the nest = stronger breach.
        1.0 + (depth / 220.0).min(1.5)
    } else {
        let outside = -depth;
        if outside >= BREACH_PROXIMITY {
            0.0
        } else {
            (1.0 - outside / BREACH_PROXIMITY).powf(1.2)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Link {
    pub from_node: u32,
    pub from_port: usize,
    pub to_node: u32,
    pub to_port: usize,
    /// Physical PowerWire node created with this link.
    pub cable_id: u32,
}

pub struct World {
    pub nodes: HashMap<u32, Node>,
    pub links: Vec<Link>,
    /// Factorio-style belt tile grid.
    pub belt_tiles: BeltGrid,
    pub next_id: u32,
    pub network_energy: HashMap<u32, f32>,
    pub energy_prod: f32,
    pub energy_use: f32,
    pub nests: Vec<Nest>,
    pub raiders: Vec<Raider>,
    pub combat_shots: Vec<CombatShot>,
    pub storm_blots: Vec<StormBlot>,
    pub next_nest_id: u32,
    pub next_raider_id: u32,
    pub next_wave_id: u32,
}

impl World {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            links: Vec::new(),
            belt_tiles: HashMap::new(),
            next_id: 1,
            network_energy: HashMap::new(),
            energy_prod: 0.0,
            energy_use: 0.0,
            nests: Vec::new(),
            raiders: Vec::new(),
            combat_shots: Vec::new(),
            storm_blots: Vec::new(),
            next_nest_id: 1,
            next_raider_id: 1,
            next_wave_id: 1,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Place dormant nests in the storm ring outside the starting clear pocket.
    pub fn seed_nests(&mut self, storm_cx: f32, storm_cy: f32, storm_safe_r: f32) {
        self.nests.clear();
        self.raiders.clear();
        self.combat_shots.clear();
        self.storm_blots.clear();
        self.next_nest_id = 1;
        self.next_raider_id = 1;
        self.next_wave_id = 1;
        let hard = storm_safe_r * CLEAR_HARD_SCALE;
        let count = NEST_COUNT_DEFAULT;
        for i in 0..count {
            let ang = (i as f32 / count as f32) * std::f32::consts::TAU + 0.41;
            let t = ((i * 19 + 7) % 100) as f32 / 100.0;
            let dist = hard * (1.28 + t * 1.05);
            let x = storm_cx + ang.cos() * dist;
            let y = storm_cy + ang.sin() * dist;
            let id = self.next_nest_id;
            self.next_nest_id += 1;
            self.nests.push(Nest {
                id,
                x,
                y,
                hp: NEST_HP,
                max_hp: NEST_HP,
                active: false,
                wave_cd: 8.0 + t * 14.0,
                evolution: 0.0,
                anger: 0.0,
                first_wave: true,
                dormant_hate: false,
            });
        }
    }

    pub fn set_id_namespace(&mut self, player_id: u8) {
        let base = (player_id as u32 + 1) * 1_000_000;
        if self.next_id < base {
            self.next_id = base;
        }
    }

    pub fn place_node(&mut self, kind: BuildingKind, x: f32, y: f32, facing: Facing) -> Option<u32> {
        if kind.is_cable() || kind.is_belt_tool() {
            return None;
        }
        let probe = Node::new(kind, x, y, facing);
        if self.collides(probe.x, probe.y, probe.w(), probe.h(), None) {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, probe);
        Some(id)
    }

    pub fn place_node_with_id(
        &mut self,
        id: u32,
        kind: BuildingKind,
        x: f32,
        y: f32,
        facing: Facing,
    ) -> bool {
        if kind.is_cable() || kind.is_belt_tool() {
            return false;
        }
        let probe = Node::new(kind, x, y, facing);
        if id >= self.next_id {
            self.next_id = id + 1;
        }
        // Remote sync always wins so peers converge even after local collisions.
        self.nodes.insert(id, probe);
        true
    }

    pub fn try_move_node(&mut self, id: u32, x: f32, y: f32) -> bool {
        let (w, h, kind) = match self.nodes.get(&id) {
            Some(n) => (n.w(), n.h(), n.kind),
            None => return false,
        };
        if kind.is_cable() || kind.is_belt_tool() {
            return false;
        }
        if self.collides(x, y, w, h, Some(id)) {
            return false;
        }
        if let Some(n) = self.nodes.get_mut(&id) {
            n.x = x;
            n.y = y;
        } else {
            return false;
        }
        self.sync_cable_anchors();
        true
    }

    pub fn force_move_node(&mut self, id: u32, x: f32, y: f32) {
        if let Some(n) = self.nodes.get_mut(&id) {
            if n.kind.is_cable() {
                return;
            }
            n.x = x;
            n.y = y;
        }
        self.sync_cable_anchors();
    }

    pub fn try_rotate_node(&mut self, id: u32) -> bool {
        let (x, y, w, h, next) = {
            let Some(n) = self.nodes.get(&id) else {
                return false;
            };
            if !n.kind.can_rotate() {
                return false;
            }
            let next = n.facing.rotate_cw();
            let (cx, cy) = n.center();
            let (bw, bh) = n.kind.size();
            let (nw, nh) = match next {
                Facing::E | Facing::W => (bw, bh),
                Facing::N | Facing::S => (bh, bw),
            };
            (cx - nw * 0.5, cy - nh * 0.5, nw, nh, next)
        };
        if self.collides(x, y, w, h, Some(id)) {
            return false;
        }
        if let Some(n) = self.nodes.get_mut(&id) {
            n.set_facing(next);
        } else {
            return false;
        }
        self.sync_cable_anchors();
        true
    }

    pub fn force_set_facing(&mut self, id: u32, facing: Facing) {
        if let Some(n) = self.nodes.get_mut(&id) {
            n.set_facing(facing);
        }
    }

    pub fn collides(&self, x: f32, y: f32, w: f32, h: f32, ignore: Option<u32>) -> bool {
        if self.tile_blocked_by_belt(x, y, w, h) {
            return true;
        }
        for (&id, n) in &self.nodes {
            if Some(id) == ignore {
                continue;
            }
            if n.kind == BuildingKind::PowerWire {
                continue; // copper-style wires don't block placement
            }
            // Legacy conveyor cable nodes (pre-grid) — ignore for collision.
            if n.kind == BuildingKind::Conveyor {
                continue;
            }
            if n.overlaps_rect(x, y, w, h) {
                return true;
            }
        }
        false
    }

    pub fn remove_node(&mut self, id: u32) {
        let is_cable = self
            .nodes
            .get(&id)
            .map(|n| n.kind.is_cable())
            .unwrap_or(false);
        if is_cable {
            self.links.retain(|l| l.cable_id != id);
            self.nodes.remove(&id);
            return;
        }

        let cable_ids: Vec<u32> = self
            .links
            .iter()
            .filter(|l| l.from_node == id || l.to_node == id)
            .map(|l| l.cable_id)
            .collect();

        self.nodes.remove(&id);
        self.links
            .retain(|l| l.from_node != id && l.to_node != id);
        for cid in cable_ids {
            self.nodes.remove(&cid);
        }
    }

    pub fn hit_node(&self, wx: f32, wy: f32) -> Option<u32> {
        let mut best_building = None;
        for (&id, n) in &self.nodes {
            if n.kind.is_cable() {
                continue;
            }
            if n.contains(wx, wy) {
                best_building = Some(id);
            }
        }
        if best_building.is_some() {
            return best_building;
        }

        let mut best_cable: Option<(u32, f32)> = None;
        for (&id, n) in &self.nodes {
            if !n.kind.is_cable() {
                continue;
            }
            let Some(((ax, ay), (bx, by))) = self.cable_endpoints(n) else {
                continue;
            };
            let half = WIRE_HIT_HALF;
            let d = dist_to_manhattan(wx, wy, ax, ay, bx, by);
            if d <= half && best_cable.map(|(_, bd)| d < bd).unwrap_or(true) {
                best_cable = Some((id, d));
            }
        }
        best_cable.map(|(id, _)| id)
    }

    fn cable_endpoints(&self, n: &Node) -> Option<((f32, f32), (f32, f32))> {
        let (a, b) = (n.cable_a?, n.cable_b?);
        let na = self.nodes.get(&a.0)?;
        let nb = self.nodes.get(&b.0)?;
        let pa = na.port_world(a.1)?;
        let pb = nb.port_world(b.1)?;
        Some((pa, pb))
    }

    /// Keep cable anchor nodes centered on their Manhattan midpoints (for HP / storm).
    pub fn sync_cable_anchors(&mut self) {
        let updates: Vec<(u32, f32, f32)> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.kind.is_cable())
            .filter_map(|(&id, n)| {
                let ((ax, ay), (bx, by)) = self.cable_endpoints(n)?;
                let mx = (ax + bx) * 0.5;
                let my = (ay + by) * 0.5;
                let (w, h) = n.size();
                Some((id, mx - w * 0.5, my - h * 0.5))
            })
            .collect();
        for (id, x, y) in updates {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.x = x;
                n.y = y;
            }
        }
    }

    fn spawn_cable(
        &mut self,
        kind: BuildingKind,
        from: (u32, usize),
        to: (u32, usize),
    ) -> Option<u32> {
        let ((ax, ay), (bx, by)) = {
            let na = self.nodes.get(&from.0)?;
            let nb = self.nodes.get(&to.0)?;
            (na.port_world(from.1)?, nb.port_world(to.1)?)
        };
        let mx = (ax + bx) * 0.5;
        let my = (ay + by) * 0.5;
        let (w, h) = kind.size();
        let id = self.next_id;
        self.next_id += 1;
        let mut node = Node::new(kind, mx - w * 0.5, my - h * 0.5, Facing::E);
        node.cable_a = Some(from);
        node.cable_b = Some(to);
        self.nodes.insert(id, node);
        Some(id)
    }

    /// Rebuild missing cable entities for old saves / desynced links.
    pub fn ensure_cable_entities(&mut self) {
        for i in 0..self.links.len() {
            let (from, to, cid) = {
                let l = &self.links[i];
                (
                    (l.from_node, l.from_port),
                    (l.to_node, l.to_port),
                    l.cable_id,
                )
            };
            let ok = self
                .nodes
                .get(&cid)
                .map(|n| n.kind == BuildingKind::PowerWire)
                .unwrap_or(false);
            if !ok {
                if let Some(new_id) = self.spawn_cable(BuildingKind::PowerWire, from, to) {
                    self.links[i].cable_id = new_id;
                }
            }
        }
        // Drop orphan cable nodes not referenced by any power link.
        let used: HashSet<u32> = self.links.iter().map(|l| l.cable_id).collect();
        let orphans: Vec<u32> = self
            .nodes
            .iter()
            .filter(|(&id, n)| n.kind.is_cable() && !used.contains(&id))
            .map(|(&id, _)| id)
            .collect();
        for id in orphans {
            self.nodes.remove(&id);
        }
        // Purge legacy Conveyor cable nodes from pre-grid saves.
        let legacy: Vec<u32> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == BuildingKind::Conveyor)
            .map(|(&id, _)| id)
            .collect();
        for id in legacy {
            self.nodes.remove(&id);
        }
    }

    pub fn hit_port(&self, wx: f32, wy: f32, radius: f32) -> Option<(u32, usize)> {
        let r2 = radius * radius;
        let mut best = None;
        for (&id, n) in &self.nodes {
            for (pi, p) in n.ports.iter().enumerate() {
                let d2 = (n.x + p.ox - wx).powi(2) + (n.y + p.oy - wy).powi(2);
                if d2 <= r2 && best.map(|(_, _, bd)| d2 < bd).unwrap_or(true) {
                    best = Some((id, pi, d2));
                }
            }
        }
        best.map(|(a, b, _)| (a, b))
    }

    fn port_manhattan(&self, from: (u32, usize), to: (u32, usize)) -> Option<f32> {
        let a = self.nodes.get(&from.0)?.port_world(from.1)?;
        let b = self.nodes.get(&to.0)?.port_world(to.1)?;
        Some((b.0 - a.0).abs() + (b.1 - a.1).abs())
    }

    fn power_links_on_port(&self, node: u32, port: usize) -> usize {
        self.links
            .iter()
            .filter(|l| {
                (l.from_node == node && l.from_port == port)
                    || (l.to_node == node && l.to_port == port)
            })
            .count()
    }

    pub fn can_connect_power(&self, from: (u32, usize), to: (u32, usize)) -> bool {
        self.power_connect_fail(from, to).is_none()
            || self.power_connect_fail(to, from).is_none()
    }

    /// Human-readable reason if neither orientation of a power link works.
    pub fn connect_fail_hint(
        &self,
        from: (u32, usize),
        to: (u32, usize),
        _power: bool,
    ) -> Option<&'static str> {
        match (
            self.power_connect_fail(from, to),
            self.power_connect_fail(to, from),
        ) {
            (None, _) | (_, None) => None,
            (Some(a), Some(b)) => Some(prefer_connect_hint(a, b)),
        }
    }

    /// Legacy no-op — belts are grid tiles now.
    pub fn can_connect_belt(&self, _from: (u32, usize), _to: (u32, usize)) -> bool {
        false
    }

    pub fn connect_belt(&mut self, _from: (u32, usize), _to: (u32, usize)) -> bool {
        false
    }

    pub fn power_connect_fail(&self, from: (u32, usize), to: (u32, usize)) -> Option<&'static str> {
        if from.0 == to.0 {
            return Some("Can't wire a building to itself");
        }
        let Some(pa) = self.nodes.get(&from.0).and_then(|n| n.ports.get(from.1)) else {
            return Some("Missing port");
        };
        let Some(pb) = self.nodes.get(&to.0).and_then(|n| n.ports.get(to.1)) else {
            return Some("Missing port");
        };
        let ok_kinds = matches!(
            (pa.kind, pb.kind),
            (PortKind::EnergyOut, PortKind::EnergyAny)
                | (PortKind::EnergyAny, PortKind::EnergyOut)
                | (PortKind::EnergyAny, PortKind::EnergyAny)
        );
        if !ok_kinds {
            return Some("Need energy sockets (OUT → socket, or socket ↔ socket)");
        }
        if let Some(d) = self.port_manhattan(from, to) {
            if d > POWER_WIRE_MAX_REACH {
                return Some("Too far — place a Power Pole closer");
            }
        }
        if self.power_links_on_port(from.0, from.1) >= MAX_POWER_LINKS_PER_PORT
            || self.power_links_on_port(to.0, to.1) >= MAX_POWER_LINKS_PER_PORT
        {
            return Some("Socket full — max wires on that port");
        }
        if self.links.iter().any(|l| {
            (l.from_node, l.from_port, l.to_node, l.to_port) == (from.0, from.1, to.0, to.1)
                || (l.from_node, l.from_port, l.to_node, l.to_port) == (to.0, to.1, from.0, from.1)
        }) {
            return Some("Already wired");
        }
        None
    }

    pub fn connect_power(&mut self, from: (u32, usize), to: (u32, usize)) -> bool {
        let (from, to) = if self.power_connect_fail(from, to).is_none() {
            let fa = self
                .nodes
                .get(&from.0)
                .and_then(|n| n.ports.get(from.1))
                .map(|p| p.kind);
            let tb = self
                .nodes
                .get(&to.0)
                .and_then(|n| n.ports.get(to.1))
                .map(|p| p.kind);
            match (fa, tb) {
                (Some(PortKind::EnergyAny), Some(PortKind::EnergyOut)) => (to, from),
                _ => (from, to),
            }
        } else if self.power_connect_fail(to, from).is_none() {
            let fa = self
                .nodes
                .get(&to.0)
                .and_then(|n| n.ports.get(to.1))
                .map(|p| p.kind);
            let tb = self
                .nodes
                .get(&from.0)
                .and_then(|n| n.ports.get(from.1))
                .map(|p| p.kind);
            match (fa, tb) {
                (Some(PortKind::EnergyAny), Some(PortKind::EnergyOut)) => (from, to),
                _ => (to, from),
            }
        } else {
            return false;
        };
        let Some(cable_id) = self.spawn_cable(BuildingKind::PowerWire, from, to) else {
            return false;
        };
        self.links.push(Link {
            from_node: from.0,
            from_port: from.1,
            to_node: to.0,
            to_port: to.1,
            cable_id,
        });
        true
    }

    pub fn tick(&mut self, dt: f32) {
        let (node_net, gen_by_net, powered_poles) = self.power_step(dt);
        self.machine_step(dt, &node_net, &gen_by_net, &powered_poles);
        self.belt_grid_step(dt);
    }

    /// Nest activation, attack waves, swarming raids, and turret fire.
    pub fn combat_step(&mut self, dt: f32, clear_zones: &[(f32, f32, f32)]) -> CombatReport {
        let mut report = CombatReport::default();

        for shot in &mut self.combat_shots {
            shot.life -= dt;
        }
        self.combat_shots.retain(|s| s.life > 0.0);

        // Tick fogcaller storm blots — damage buildings caught in the blot.
        let mut blot_damage: Vec<(u32, f32)> = Vec::new();
        for blot in &mut self.storm_blots {
            blot.life -= dt;
            blot.tick_cd -= dt;
            if blot.life <= 0.0 || blot.tick_cd > 0.0 {
                continue;
            }
            blot.tick_cd = FOG_BLOT_TICK;
            let r2 = blot.radius * blot.radius;
            for (&id, n) in &self.nodes {
                if n.kind.is_cable() {
                    continue;
                }
                let (cx, cy) = n.center();
                if (cx - blot.x).powi(2) + (cy - blot.y).powi(2) <= r2 {
                    blot_damage.push((id, FOG_BLOT_DAMAGE));
                }
            }
        }
        self.storm_blots.retain(|b| b.life > 0.0);
        for (id, dmg) in blot_damage {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.hp = (n.hp - dmg).max(0.0);
            }
        }
        let blot_kills: Vec<u32> = self
            .nodes
            .iter()
            .filter(|(_, n)| !n.kind.is_cable() && n.hp <= 0.0)
            .map(|(&id, _)| id)
            .collect();
        for id in blot_kills {
            self.remove_node(id);
            report.destroyed += 1;
        }

        for nest in &mut self.nests {
            if nest.hp <= 0.0 {
                continue;
            }
            let in_clear = point_in_hard_clear(nest.x, nest.y, clear_zones);
            let scent = breach_scent(nest.x, nest.y, clear_zones);
            let rim_dist = dist_to_clear_rim(nest.x, nest.y, clear_zones);

            // Dormant hate: clear retracted — hide again, keep evolution/anger.
            if nest.active && !in_clear {
                nest.active = false;
                nest.dormant_hate = true;
                nest.anger = (nest.anger + 20.0).min(140.0);
                nest.first_wave = true;
                continue;
            }

            if !nest.active && in_clear {
                let reawaken = nest.dormant_hate;
                nest.active = true;
                nest.first_wave = true;
                nest.wave_cd = if reawaken {
                    NEST_REVEAL_WINDUP * 0.65
                } else {
                    NEST_REVEAL_WINDUP
                };
                if reawaken {
                    nest.anger = (nest.anger + 40.0 + nest.evolution * 45.0).min(160.0);
                    nest.dormant_hate = false;
                    report.nests_reawakened += 1;
                } else {
                    nest.anger = (nest.anger + 55.0).max(55.0);
                    report.nests_revealed += 1;
                }
            }

            if nest.active {
                // Edge-of-fog nests attune faster (half-revealed pressure).
                let edge_bonus = if rim_dist < 140.0 { 1.6 } else { 1.0 };
                nest.evolution =
                    (nest.evolution + dt * 0.006 * edge_bonus * (0.7 + scent * 0.5)).min(1.0);
                nest.anger += dt * (0.35 + scent * 2.2 + nest.evolution * 1.1);
            } else if nest.dormant_hate {
                // Still stewing in the fog.
                nest.evolution = (nest.evolution + dt * 0.003).min(1.0);
                nest.anger = (nest.anger + dt * 0.15).min(140.0);
            }
        }

        // Decide which nests launch a wave this step.
        let mut launches: Vec<(f32, f32, usize, bool)> = Vec::new();
        for nest in &mut self.nests {
            if nest.hp <= 0.0 || !nest.active {
                continue;
            }
            nest.wave_cd -= dt;
            let interval =
                (NEST_WAVE_INTERVAL - nest.evolution * 12.0).max(NEST_WAVE_MIN_INTERVAL);
            let anger_trigger = nest.anger >= 45.0 + nest.evolution * 18.0;
            if nest.wave_cd > 0.0 && !anger_trigger {
                continue;
            }
            nest.wave_cd = interval;
            nest.anger = (nest.anger * 0.2).max(0.0);
            let first = nest.first_wave;
            nest.first_wave = false;
            let count = if first {
                (9 + (nest.evolution * 6.0).round() as usize).min(18)
            } else {
                let base = 5 + (nest.evolution * 9.0).round() as usize;
                let bonus = if anger_trigger { 3 } else { 0 };
                (base + bonus).min(16)
            };
            launches.push((nest.x, nest.y, count, first));
        }
        for (sx, sy, count, _first) in launches {
            self.spawn_wave(sx, sy, count, clear_zones);
            report.waves_launched += 1;
        }

        let centers: HashMap<u32, (f32, f32)> = self
            .nodes
            .iter()
            .filter(|(_, n)| !n.kind.is_cable())
            .map(|(&id, n)| (id, n.center()))
            .collect();
        let kinds: HashMap<u32, BuildingKind> = self
            .nodes
            .iter()
            .filter(|(_, n)| !n.kind.is_cable())
            .map(|(&id, n)| (id, n.kind))
            .collect();
        let alive_nodes: HashSet<u32> = centers.keys().copied().collect();

        // Role-aware retargeting toward rim / specialty targets.
        let retargets: Vec<(usize, RaiderRole, f32, f32)> = self
            .raiders
            .iter()
            .enumerate()
            .filter(|(_, r)| r.hp > 0.0)
            .map(|(i, r)| (i, r.role, r.x, r.y))
            .collect();
        for (i, role, x, y) in retargets {
            let Some(r) = self.raiders.get_mut(i) else {
                continue;
            };
            r.retarget_cd -= dt;
            let lost = r
                .target_node
                .map(|tid| !alive_nodes.contains(&tid))
                .unwrap_or(true);
            if lost || r.retarget_cd <= 0.0 {
                r.target_node =
                    Self::pick_target_among(&centers, &kinds, clear_zones, x, y, role);
                r.retarget_cd = RETARGET_INTERVAL * (0.85 + (r.id % 5) as f32 * 0.06);
            }
        }

        // Flocking snapshot: (wave_id, x, y)
        let positions: Vec<(u32, f32, f32)> = self
            .raiders
            .iter()
            .filter(|r| r.hp > 0.0)
            .map(|r| (r.wave_id, r.x, r.y))
            .collect();

        let mut damage: Vec<(u32, f32)> = Vec::new();
        let mut new_blots: Vec<StormBlot> = Vec::new();
        for raider in &mut self.raiders {
            if raider.hp <= 0.0 {
                continue;
            }
            let (tx, ty) = raider
                .target_node
                .and_then(|tid| centers.get(&tid).copied())
                .unwrap_or((0.0, 0.0));

            let mut sep_x = 0.0;
            let mut sep_y = 0.0;
            let mut coh_x = 0.0;
            let mut coh_y = 0.0;
            let mut coh_n = 0.0;
            for &(wid, ox, oy) in &positions {
                if wid != raider.wave_id {
                    continue;
                }
                let dx = raider.x - ox;
                let dy = raider.y - oy;
                let d2 = dx * dx + dy * dy;
                if d2 < 0.01 {
                    continue;
                }
                let d = d2.sqrt();
                if d < SWARM_SEP_RADIUS {
                    sep_x += dx / d;
                    sep_y += dy / d;
                }
                if d < SWARM_COH_RADIUS {
                    coh_x += ox;
                    coh_y += oy;
                    coh_n += 1.0;
                }
            }
            if coh_n > 0.0 {
                coh_x = coh_x / coh_n - raider.x;
                coh_y = coh_y / coh_n - raider.y;
            }

            let dx = tx - raider.x;
            let dy = ty - raider.y;
            let dist = (dx * dx + dy * dy).sqrt().max(0.001);
            let seek_x = dx / dist;
            let seek_y = dy / dist;

            // Hunters rush defenses a bit harder; saboteurs slightly sneakier (slower).
            let role_speed = match raider.role {
                RaiderRole::Assault => 1.0,
                RaiderRole::Hunter => 1.12,
                RaiderRole::Saboteur => 0.92,
                RaiderRole::Fogcaller => 0.72,
            };
            let mut steer_x = seek_x * 1.15 + sep_x * 1.25 + coh_x * 0.012;
            let mut steer_y = seek_y * 1.15 + sep_y * 1.25 + coh_y * 0.012;
            let sl = (steer_x * steer_x + steer_y * steer_y).sqrt().max(0.001);
            steer_x /= sl;
            steer_y /= sl;

            let speed = RAIDER_SPEED * role_speed * (0.88 + (raider.wave_id % 5) as f32 * 0.03);
            raider.vx = raider.vx * 0.8 + steer_x * speed * 0.2;
            raider.vy = raider.vy * 0.8 + steer_y * speed * 0.2;
            let vlen = (raider.vx * raider.vx + raider.vy * raider.vy)
                .sqrt()
                .max(0.001);
            if vlen > speed {
                raider.vx *= speed / vlen;
                raider.vy *= speed / vlen;
            }

            if dist > RAIDER_ATTACK_RANGE {
                raider.x += raider.vx * dt;
                raider.y += raider.vy * dt;
            } else if let Some(tid) = raider.target_node {
                raider.vx *= 0.45;
                raider.vy *= 0.45;
                raider.attack_cd -= dt;
                if raider.attack_cd <= 0.0 {
                    raider.attack_cd = RAIDER_ATTACK_INTERVAL;
                    let dmg = match raider.role {
                        RaiderRole::Assault => RAIDER_DAMAGE,
                        RaiderRole::Hunter => RAIDER_DAMAGE * 1.15,
                        RaiderRole::Saboteur => RAIDER_DAMAGE * 1.25,
                        RaiderRole::Fogcaller => RAIDER_DAMAGE * 0.85,
                    };
                    damage.push((tid, dmg));
                    if raider.role == RaiderRole::Fogcaller {
                        new_blots.push(StormBlot {
                            x: raider.x,
                            y: raider.y,
                            radius: FOG_BLOT_RADIUS * 0.65,
                            life: FOG_BLOT_LIFE * 0.55,
                            tick_cd: 0.15,
                        });
                    }
                }
            }
        }
        self.storm_blots.extend(new_blots);

        let mut dead_buildings: Vec<u32> = Vec::new();
        for (tid, dmg) in damage {
            if let Some(n) = self.nodes.get_mut(&tid) {
                n.hp = (n.hp - dmg).max(0.0);
                if n.hp <= 0.0 {
                    dead_buildings.push(tid);
                }
            }
        }
        for id in dead_buildings {
            if self.nodes.contains_key(&id) {
                self.remove_node(id);
                report.destroyed += 1;
            }
        }

        let turret_ids: Vec<u32> = self
            .nodes
            .iter()
            .filter_map(|(&id, n)| {
                if n.kind == BuildingKind::Turret && n.powered {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();
        for tid in turret_ids {
            let (tcx, tcy, mut cd) = {
                let Some(n) = self.nodes.get(&tid) else {
                    continue;
                };
                let (cx, cy) = n.center();
                (cx, cy, n.cooldown)
            };
            cd = (cd - dt).max(0.0);
            if cd > 0.0 {
                if let Some(n) = self.nodes.get_mut(&tid) {
                    n.cooldown = cd;
                    n.working = true;
                }
                continue;
            }

            let range2 = TURRET_RANGE * TURRET_RANGE;
            let mut best_raider: Option<(usize, f32)> = None;
            for (i, r) in self.raiders.iter().enumerate() {
                if r.hp <= 0.0 {
                    continue;
                }
                let d2 = (r.x - tcx).powi(2) + (r.y - tcy).powi(2);
                if d2 <= range2 && best_raider.map(|(_, bd)| d2 < bd).unwrap_or(true) {
                    best_raider = Some((i, d2));
                }
            }

            let mut shot_to: Option<(f32, f32)> = None;
            if let Some((i, _)) = best_raider {
                if let Some(r) = self.raiders.get_mut(i) {
                    r.hp -= TURRET_DAMAGE;
                    shot_to = Some((r.x, r.y));
                }
            } else {
                let mut best_nest: Option<(usize, f32)> = None;
                for (i, nest) in self.nests.iter().enumerate() {
                    if nest.hp <= 0.0 || !point_in_hard_clear(nest.x, nest.y, clear_zones) {
                        continue;
                    }
                    let d2 = (nest.x - tcx).powi(2) + (nest.y - tcy).powi(2);
                    if d2 <= range2 && best_nest.map(|(_, bd)| d2 < bd).unwrap_or(true) {
                        best_nest = Some((i, d2));
                    }
                }
                if let Some((i, _)) = best_nest {
                    if let Some(nest) = self.nests.get_mut(i) {
                        nest.hp -= TURRET_DAMAGE;
                        nest.anger += 40.0;
                        nest.wave_cd = nest.wave_cd.min(1.2);
                        shot_to = Some((nest.x, nest.y));
                    }
                }
            }

            if let Some(n) = self.nodes.get_mut(&tid) {
                if shot_to.is_some() {
                    n.cooldown = TURRET_FIRE_INTERVAL;
                    n.working = true;
                } else {
                    n.cooldown = 0.0;
                    n.working = n.powered;
                }
            }
            if let Some((x1, y1)) = shot_to {
                self.combat_shots.push(CombatShot {
                    x0: tcx,
                    y0: tcy,
                    x1,
                    y1,
                    life: 0.12,
                });
            }
        }

        // Fogcallers leave a storm blot on death (turret or otherwise).
        let death_blots: Vec<StormBlot> = self
            .raiders
            .iter()
            .filter(|r| r.hp <= 0.0 && r.role == RaiderRole::Fogcaller)
            .map(|r| StormBlot {
                x: r.x,
                y: r.y,
                radius: FOG_BLOT_RADIUS,
                life: FOG_BLOT_LIFE,
                tick_cd: 0.1,
            })
            .collect();
        self.storm_blots.extend(death_blots);

        self.raiders.retain(|r| r.hp > 0.0);
        self.nests.retain(|n| n.hp > 0.0);
        report
    }

    fn spawn_wave(
        &mut self,
        nest_x: f32,
        nest_y: f32,
        count: usize,
        clear_zones: &[(f32, f32, f32)],
    ) {
        let room = MAX_RAIDERS.saturating_sub(self.raiders.len());
        let count = count.min(room);
        if count == 0 {
            return;
        }
        let wave_id = self.next_wave_id;
        self.next_wave_id = self.next_wave_id.wrapping_add(1).max(1);

        let centers: HashMap<u32, (f32, f32)> = self
            .nodes
            .iter()
            .filter(|(_, n)| !n.kind.is_cable())
            .map(|(&id, n)| (id, n.center()))
            .collect();
        let kinds: HashMap<u32, BuildingKind> = self
            .nodes
            .iter()
            .filter(|(_, n)| !n.kind.is_cable())
            .map(|(&id, n)| (id, n.kind))
            .collect();

        // Mixed composition: assault, hunters, saboteurs, fogcallers.
        for i in 0..count {
            let role = match i % 6 {
                0 | 1 | 2 => RaiderRole::Assault,
                3 => RaiderRole::Hunter,
                4 => RaiderRole::Saboteur,
                _ => RaiderRole::Fogcaller,
            };
            let ang = (i as f32 / count as f32) * std::f32::consts::TAU;
            let spread = 18.0 + (i % 3) as f32 * 6.0;
            let x = nest_x + ang.cos() * spread;
            let y = nest_y + ang.sin() * spread;
            let target = Self::pick_target_among(&centers, &kinds, clear_zones, x, y, role);
            let id = self.next_raider_id;
            self.next_raider_id = self.next_raider_id.wrapping_add(1).max(1);
            let hp = if role == RaiderRole::Fogcaller {
                RAIDER_HP * 1.45
            } else {
                RAIDER_HP
            };
            self.raiders.push(Raider {
                id,
                x,
                y,
                hp,
                target_node: target,
                attack_cd: 0.15 + (i as f32) * 0.03,
                wave_id,
                vx: ang.cos() * RAIDER_SPEED * 0.3,
                vy: ang.sin() * RAIDER_SPEED * 0.3,
                role,
                retarget_cd: RETARGET_INTERVAL * 0.5,
            });
        }
    }

    fn is_defense(kind: BuildingKind) -> bool {
        matches!(kind, BuildingKind::Turret)
    }

    fn is_power_target(kind: BuildingKind) -> bool {
        matches!(
            kind,
            BuildingKind::Solar | BuildingKind::PowerPole | BuildingKind::Totem
        )
    }

    fn pick_target_among(
        centers: &HashMap<u32, (f32, f32)>,
        kinds: &HashMap<u32, BuildingKind>,
        clear_zones: &[(f32, f32, f32)],
        from_x: f32,
        from_y: f32,
        role: RaiderRole,
    ) -> Option<u32> {
        let seek2 = ROLE_SEEK_RANGE * ROLE_SEEK_RANGE;
        let mut nearest_any: Option<(u32, f32)> = None;
        let mut nearest_role: Option<(u32, f32)> = None;
        for (&id, &(cx, cy)) in centers {
            let Some(&kind) = kinds.get(&id) else {
                continue;
            };
            let d2 = (cx - from_x).powi(2) + (cy - from_y).powi(2);
            // Prefer clear-rim buildings: deep interior targets score worse.
            let rim = dist_to_clear_rim(cx, cy, clear_zones);
            let score = d2 + rim * rim * RIM_TARGET_WEIGHT;
            if nearest_any.map(|(_, bd)| score < bd).unwrap_or(true) {
                nearest_any = Some((id, score));
            }
            let specialty = match role {
                RaiderRole::Assault | RaiderRole::Fogcaller => false,
                RaiderRole::Hunter => Self::is_defense(kind),
                RaiderRole::Saboteur => Self::is_power_target(kind),
            };
            if specialty && d2 <= seek2 && nearest_role.map(|(_, bd)| score < bd).unwrap_or(true)
            {
                nearest_role = Some((id, score));
            }
        }
        match role {
            RaiderRole::Assault | RaiderRole::Fogcaller => nearest_any.map(|(id, _)| id),
            RaiderRole::Hunter | RaiderRole::Saboteur => {
                nearest_role.or(nearest_any).map(|(id, _)| id)
            }
        }
    }


    fn power_step(&mut self, dt: f32) -> (HashMap<u32, u32>, HashMap<u32, f32>, HashSet<u32>) {
        let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
        for (&id, n) in &self.nodes {
            if matches!(n.kind, BuildingKind::Solar | BuildingKind::PowerPole) {
                adj.entry(id).or_default();
            }
        }
        for l in &self.links {
            adj.entry(l.from_node).or_default().push(l.to_node);
            adj.entry(l.to_node).or_default().push(l.from_node);
        }
        let mut visited = HashSet::new();
        let mut node_net = HashMap::new();
        let mut gen_by_net: HashMap<u32, f32> = HashMap::new();
        for &start in adj.keys() {
            if !visited.insert(start) {
                continue;
            }
            let mut q = VecDeque::from([start]);
            let mut members = Vec::new();
            let mut gen = 0.0;
            let mut root = start;
            while let Some(id) = q.pop_front() {
                members.push(id);
                root = root.min(id);
                if self.nodes.get(&id).map(|n| n.kind) == Some(BuildingKind::Solar) {
                    gen += SOLAR_POWER;
                }
                if let Some(neis) = adj.get(&id) {
                    for &n in neis {
                        if visited.insert(n) {
                            q.push_back(n);
                        }
                    }
                }
            }
            for id in members {
                node_net.insert(id, root);
            }
            gen_by_net.insert(root, gen);
        }
        let mut total = 0.0;
        for (&root, &gen) in &gen_by_net {
            total += gen;
            let e = self.network_energy.entry(root).or_insert(0.0);
            *e = (*e + gen * dt).min(2000.0);
        }
        self.energy_prod = total;
        let mut poles = HashSet::new();
        for (&id, n) in &self.nodes {
            if n.kind != BuildingKind::PowerPole {
                continue;
            }
            if let Some(&root) = node_net.get(&id) {
                if gen_by_net.get(&root).copied().unwrap_or(0.0) > 0.0
                    || self.network_energy.get(&root).copied().unwrap_or(0.0) > 0.0
                {
                    poles.insert(id);
                }
            }
        }
        let ids: Vec<u32> = self.nodes.keys().copied().collect();
        for id in ids {
            let covered = {
                let Some(n) = self.nodes.get(&id) else {
                    continue;
                };
                if !n.kind.needs_power() {
                    true
                } else {
                    let (cx, cy) = n.center();
                    poles.iter().any(|pid| {
                        self.nodes.get(pid).map(|p| {
                            let (px, py) = p.center();
                            (cx - px).powi(2) + (cy - py).powi(2) <= POLE_RADIUS * POLE_RADIUS
                        }) == Some(true)
                    })
                }
            };
            if let Some(n) = self.nodes.get_mut(&id) {
                n.powered = covered;
                n.working = match n.kind {
                    BuildingKind::Solar => node_net.contains_key(&id),
                    BuildingKind::PowerPole => poles.contains(&id),
                    BuildingKind::Totem | BuildingKind::Turret => covered,
                    _ => n.working,
                };
            }
        }
        (node_net, gen_by_net, poles)
    }

    fn machine_step(
        &mut self,
        dt: f32,
        node_net: &HashMap<u32, u32>,
        _gen: &HashMap<u32, f32>,
        poles: &HashSet<u32>,
    ) {
        let mut energy_draw = 0.0;
        let mut ids: Vec<u32> = self.nodes.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            let pay = {
                let Some(n) = self.nodes.get(&id) else {
                    continue;
                };
                if !n.kind.needs_power() {
                    None
                } else {
                    let (cx, cy) = n.center();
                    let mut best = None;
                    for &pid in poles {
                        let Some(p) = self.nodes.get(&pid) else {
                            continue;
                        };
                        let (px, py) = p.center();
                        let d2 = (cx - px).powi(2) + (cy - py).powi(2);
                        if d2 <= POLE_RADIUS * POLE_RADIUS {
                            if let Some(&root) = node_net.get(&pid) {
                                if best.map(|(_, bd)| d2 < bd).unwrap_or(true) {
                                    best = Some((root, d2));
                                }
                            }
                        }
                    }
                    best.map(|(r, _)| r)
                }
            };
            let kind = self.nodes.get(&id).map(|n| n.kind);
            let powered = self.nodes.get(&id).map(|n| n.powered).unwrap_or(false);
            match kind {
                Some(BuildingKind::OreNode) if powered => {
                    if let Some(root) = pay {
                        let cost = ORE_POWER_DRAW * dt;
                        let has = self.network_energy.get(&root).copied().unwrap_or(0.0);
                        if has >= cost {
                            if let Some(n) = self.nodes.get_mut(&id) {
                                if n.out_ore < NODE_BUFFER {
                                    let made = (ORE_RATE * dt).min(NODE_BUFFER - n.out_ore);
                                    if made > 0.0 {
                                        n.out_ore += made;
                                        n.working = true;
                                        if let Some(e) = self.network_energy.get_mut(&root) {
                                            *e -= cost;
                                        }
                                        energy_draw += ORE_POWER_DRAW;
                                    } else {
                                        n.working = false;
                                    }
                                } else {
                                    n.working = false;
                                }
                            }
                        } else if let Some(n) = self.nodes.get_mut(&id) {
                            n.working = false;
                        }
                    }
                }
                Some(BuildingKind::Smelter) if powered => {
                    if let Some(root) = pay {
                        let cost = SMELT_POWER_DRAW * dt;
                        let has = self.network_energy.get(&root).copied().unwrap_or(0.0);
                        if has >= cost {
                            if let Some(n) = self.nodes.get_mut(&id) {
                                if n.in_ore > 0.0 && n.out_ingot < NODE_BUFFER {
                                    let can = (SMELT_RATE * dt)
                                        .min(n.in_ore)
                                        .min(NODE_BUFFER - n.out_ingot);
                                    if can > 0.0 {
                                        n.in_ore -= can;
                                        n.out_ingot += can;
                                        n.working = true;
                                        if let Some(e) = self.network_energy.get_mut(&root) {
                                            *e -= cost;
                                        }
                                        energy_draw += SMELT_POWER_DRAW;
                                    } else {
                                        n.working = false;
                                    }
                                } else {
                                    n.working = false;
                                }
                            }
                        } else if let Some(n) = self.nodes.get_mut(&id) {
                            n.working = false;
                        }
                    }
                }
                Some(BuildingKind::Splitter) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.working = n.buf_ore + n.buf_ingot > 0.05;
                    }
                }
                Some(BuildingKind::Turret) if powered => {
                    if let Some(root) = pay {
                        let cost = TURRET_POWER_DRAW * dt;
                        let has = self.network_energy.get(&root).copied().unwrap_or(0.0);
                        if has >= cost {
                            if let Some(e) = self.network_energy.get_mut(&root) {
                                *e -= cost;
                            }
                            energy_draw += TURRET_POWER_DRAW;
                            if let Some(n) = self.nodes.get_mut(&id) {
                                n.working = true;
                            }
                        } else if let Some(n) = self.nodes.get_mut(&id) {
                            n.working = false;
                        }
                    }
                }
                Some(BuildingKind::Totem) if powered => {
                    // Totems only need pole coverage — no continuous draw for now.
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.working = true;
                    }
                }
                _ => {}
            }
        }
        self.energy_use = energy_draw;
    }

}

fn prefer_connect_hint(a: &'static str, b: &'static str) -> &'static str {
    const RANK: &[&str] = &[
        "Too far — place a Power Pole closer",
        "Socket full — max wires on that port",
        "Need energy sockets (OUT → socket, or socket ↔ socket)",
        "Already wired",
        "Can't wire a building to itself",
    ];
    let score = |s: &str| RANK.iter().position(|&r| r == s).unwrap_or(RANK.len());
    if score(a) <= score(b) {
        a
    } else {
        b
    }
}

