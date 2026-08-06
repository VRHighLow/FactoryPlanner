//! Buildings, power wires, and Factorio-style grid belts.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::content::{self, content, TechState};
use crate::recipes::{self, MachineKind, Recipe};

pub use crate::belts::{
    snap_building_xy_size, tile_origin, world_to_tile, BeltGrid, TILE_SIZE,
};

pub const ORE_RATE: f32 = 7.3;
pub const SOLAR_POWER: f32 = 12.0;
pub const ORE_POWER_DRAW: f32 = 4.0;
pub const TURRET_POWER_DRAW: f32 = 5.0;
pub const NODE_BUFFER: f32 = 100.0;
pub const POLE_RADIUS: f32 = 260.0;
/// Max Manhattan reach for a power wire — place poles for longer runs.
pub const POWER_WIRE_MAX_REACH: f32 = 420.0;
/// Snap cursor to an energy port within this radius when starting/ending a wire.
pub const WIRE_PORT_SNAP: f32 = 32.0;
/// Hit half-width for erasing routed wires.
pub const WIRE_PATH_HIT: f32 = 12.0;
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
/// Hunter (Eye) cannon stand-off range — fires instead of melee.
pub const EYE_CANNON_RANGE: f32 = 200.0;
pub const EYE_CANNON_INTERVAL: f32 = 1.05;
pub const EYE_RECOIL_TIME: f32 = 0.22;
/// CombatShot.style for Eye cannon bolts (CPU tracer).
pub const SHOT_STYLE_EYE: u8 = 2;
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
/// Seconds to fully charge a bolt while locked on target.
pub const TURRET_CHARGE_TIME: f32 = 1.65;
/// Recovery after each shot before charging can resume.
pub const TURRET_FIRE_INTERVAL: f32 = 0.85;
pub const TURRET_DAMAGE: f32 = 52.0;
/// Gun turn speed (rad/s) — deliberate charge-cannon tracking.
pub const TURRET_TURN_RATE: f32 = 1.55;
/// Must be aimed this close (radians) before charging starts.
pub const TURRET_AIM_LOCK: f32 = 0.09;
/// Visual bolt lifetime.
pub const TURRET_SHOT_LIFE: f32 = 0.55;
/// Hard clear scale must match main.rs STORM_HARD_CLEAR_SCALE for nest activation.
pub const CLEAR_HARD_SCALE: f32 = 0.72;
/// Click / collision half-width for physical conveyor corridors.
pub const WIRE_HIT_HALF: f32 = 10.0;

pub fn building_max_hp(kind: BuildingKind) -> f32 {
    match kind {
        BuildingKind::Totem => 160.0,
        BuildingKind::PowerPole => 70.0,
        BuildingKind::Solar => 90.0,
        BuildingKind::Turret | BuildingKind::BallisticTurret | BuildingKind::LaserTurret => 130.0,
        BuildingKind::Wall => 200.0,
        BuildingKind::ReinforcedWall => 320.0,
        BuildingKind::PowerWire => 40.0,
        BuildingKind::Conveyor => 55.0,
        BuildingKind::Nexus | BuildingKind::NexusSite => 500.0,
        _ => 110.0,
    }
}

/// Interned content item id (legacy slots 0..23 preserved for saves).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct Item(pub u16);

#[allow(non_upper_case_globals)]
impl Item {
    pub const IronOre: Item = Item(0);
    pub const IronIngot: Item = Item(1);
    pub const CopperOre: Item = Item(2);
    pub const Stone: Item = Item(3);
    pub const Coal: Item = Item(4);
    pub const CrudeOil: Item = Item(5);
    pub const CopperIngot: Item = Item(6);
    pub const Slag: Item = Item(7);
    pub const Coke: Item = Item(8);
    pub const Gear: Item = Item(9);
    pub const Wire: Item = Item(10);
    pub const Rivet: Item = Item(11);
    pub const Brick: Item = Item(12);
    pub const Pipe: Item = Item(13);
    pub const Frame: Item = Item(14);
    pub const CircuitShard: Item = Item(15);
    pub const BeltLink: Item = Item(16);
    pub const PoleKit: Item = Item(17);
    pub const SolarCell: Item = Item(18);
    pub const ShellCasing: Item = Item(19);
    pub const ChargeCell: Item = Item(20);
    pub const TotemCore: Item = Item(21);
    pub const ScienceRed: Item = Item(22);
    pub const ScienceGreen: Item = Item(23);

    /// Legacy closed set size (pre–Era 1 pack). Prefer [`Item::count`].
    pub const COUNT: usize = 24;

    pub const ALL: &'static [Item] = &[
        Self::IronOre,
        Self::IronIngot,
        Self::CopperOre,
        Self::Stone,
        Self::Coal,
        Self::CrudeOil,
        Self::CopperIngot,
        Self::Slag,
        Self::Coke,
        Self::Gear,
        Self::Wire,
        Self::Rivet,
        Self::Brick,
        Self::Pipe,
        Self::Frame,
        Self::CircuitShard,
        Self::BeltLink,
        Self::PoleKit,
        Self::SolarCell,
        Self::ShellCasing,
        Self::ChargeCell,
        Self::TotemCore,
        Self::ScienceRed,
        Self::ScienceGreen,
    ];

    pub fn count() -> usize {
        content::try_content()
            .map(|c| c.item_count().max(Self::COUNT))
            .unwrap_or(Self::COUNT)
    }

    pub fn as_u8(self) -> u8 {
        self.0.min(255) as u8
    }

    pub fn as_u16(self) -> u16 {
        self.0
    }

    pub fn from_u8(v: u8) -> Self {
        Item(v as u16)
    }

    pub fn from_u16(v: u16) -> Self {
        Item(v)
    }

    pub fn is_fluid(self) -> bool {
        content().is_fluid(self)
    }
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
    /// Free spawn tools for combat testing.
    Debug,
}

impl BuildCategory {
    pub const ALL: [BuildCategory; 7] = [
        Self::Energy,
        Self::Resource,
        Self::Processing,
        Self::Storage,
        Self::Transport,
        Self::Defense,
        Self::Debug,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Energy => "Energy",
            Self::Resource => "Resource",
            Self::Processing => "Processing",
            Self::Storage => "Storage",
            Self::Transport => "Transport",
            Self::Defense => "Defense",
            Self::Debug => "Debug",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuildingKind {
    Solar,
    PowerPole,
    OreNode,
    Smelter,
    /// Crafts Assemble recipes from the recipe registry.
    Assembler,
    Box,
    Splitter,
    Totem,
    /// Charge Cannon (energy prototype).
    Turret,
    /// Physical power cable between ports — connection tool in the build menu.
    PowerWire,
    /// Physical conveyor between ports — connection tool in the build menu.
    Conveyor,
    /// Generic Era 1 machine — `Node.machine_id` selects the def.
    Machine,
    Lab,
    FluidTank,
    Pipe,
    Wall,
    ReinforcedWall,
    BallisticTurret,
    LaserTurret,
    NexusSite,
    Nexus,
    // --- Debug spawn tools (not real buildings) ---
    SpawnAssault,
    SpawnHunter,
    SpawnSaboteur,
    SpawnFogcaller,
    SpawnNest,
}

impl BuildingKind {
    pub fn category(self) -> BuildCategory {
        match self {
            Self::Solar | Self::PowerPole | Self::Totem | Self::PowerWire => BuildCategory::Energy,
            Self::OreNode => BuildCategory::Resource,
            Self::Smelter
            | Self::Assembler
            | Self::Machine
            | Self::Lab
            | Self::NexusSite
            | Self::Nexus => BuildCategory::Processing,
            Self::Box | Self::FluidTank => BuildCategory::Storage,
            Self::Splitter | Self::Conveyor | Self::Pipe => BuildCategory::Transport,
            Self::Turret
            | Self::Wall
            | Self::ReinforcedWall
            | Self::BallisticTurret
            | Self::LaserTurret => BuildCategory::Defense,
            Self::SpawnAssault
            | Self::SpawnHunter
            | Self::SpawnSaboteur
            | Self::SpawnFogcaller
            |             Self::SpawnNest => BuildCategory::Debug,
        }
    }

    pub const DEBUG_TOOLS: [BuildingKind; 5] = [
        Self::SpawnAssault,
        Self::SpawnHunter,
        Self::SpawnSaboteur,
        Self::SpawnFogcaller,
        Self::SpawnNest,
    ];

    /// Connection tool: selected to link energy ports (not placed as a ground building).
    pub fn is_cable(self) -> bool {
        matches!(self, Self::PowerWire)
    }

    /// Placeable belt tiles (Factorio-style), painted on the grid.
    pub fn is_belt_tool(self) -> bool {
        matches!(self, Self::Conveyor)
    }

    /// Free combat testing spawners (build menu Debug category).
    pub fn is_debug_tool(self) -> bool {
        matches!(
            self,
            Self::SpawnAssault
                | Self::SpawnHunter
                | Self::SpawnSaboteur
                | Self::SpawnFogcaller
                | Self::SpawnNest
        )
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
            Self::OreNode => "Mining drill — place on a revealed vein. Yield % shared by taps.",
            Self::Smelter => "Legacy smelter — also runs Era thermal recipes when unlocked.",
            Self::Assembler => "Legacy assembler — Era assembly recipes when unlocked.",
            Self::Machine => "Generic Era 1 crafter — pick a machine recipe category at place.",
            Self::Lab => "Consumes science data packs to research technologies.",
            Self::Box => "Stores items from belts.",
            Self::FluidTank => "Stores fluids from pipes / machine fluid ports.",
            Self::Splitter => "Belt splitter — 3 wide. Alternates output evenly.",
            Self::Pipe => "Moves fluids between tanks and machine fluid ports.",
            Self::Totem => "Powered clear zone — shelters builds and reveals nests.",
            Self::Wall => "Blocks raider pathing. Cheap early defense.",
            Self::ReinforcedWall => "Harder wall for mid-threat evolution tiers.",
            Self::BallisticTurret => "Ammo-fed ballistic turret. Needs standard ammunition.",
            Self::Turret => "Charge Cannon — energy prototype. Mid/late defense.",
            Self::LaserTurret => "Laser turret — needs optical silicon tech + power.",
            Self::NexusSite => "Multi-stage construction site for the Planetary Nexus.",
            Self::Nexus => "Planetary Fabrication Nexus — Era 1 victory landmark.",
            Self::PowerWire => {
                "Click ◆ port, click to place corner anchors, click ◆ port to finish. RMB undo/erase."
            }
            Self::Conveyor => "Drag to paint belt tiles. R rotates. Loops sideload to change lanes.",
            Self::SpawnAssault => "DEBUG — spawn an assault raider at the cursor. Free.",
            Self::SpawnHunter => "DEBUG — spawn an Eye hunter (ranged cannon, prefers defenses). Free.",
            Self::SpawnSaboteur => "DEBUG — spawn a saboteur (prefers power). Free.",
            Self::SpawnFogcaller => "DEBUG — spawn a fogcaller (storm blot on death). Free.",
            Self::SpawnNest => "DEBUG — spawn an active nest that can launch waves. Free.",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Solar => "Solar Panel",
            Self::PowerPole => "Power Pole",
            Self::OreNode => "Mining Drill",
            Self::Smelter => "Smelter",
            Self::Assembler => "Assembler",
            Self::Machine => "Era Crafter",
            Self::Lab => "Research Lab",
            Self::Box => "Storage Box",
            Self::FluidTank => "Fluid Tank",
            Self::Splitter => "Splitter",
            Self::Pipe => "Fluid Pipe",
            Self::Totem => "Storm Totem",
            Self::Wall => "Wall",
            Self::ReinforcedWall => "Reinforced Wall",
            Self::BallisticTurret => "Ballistic Turret",
            Self::Turret => "Charge Cannon",
            Self::LaserTurret => "Laser Turret",
            Self::NexusSite => "Nexus Construction Site",
            Self::Nexus => "Planetary Nexus",
            Self::PowerWire => "Power Wire",
            Self::Conveyor => "Conveyor",
            Self::SpawnAssault => "Spawn Assault",
            Self::SpawnHunter => "Spawn Eye",
            Self::SpawnSaboteur => "Spawn Saboteur",
            Self::SpawnFogcaller => "Spawn Fogcaller",
            Self::SpawnNest => "Spawn Nest",
        }
    }
    pub fn short(self) -> &'static str {
        match self {
            Self::Solar => "Solar",
            Self::PowerPole => "Pole",
            Self::OreNode => "Drill",
            Self::Smelter => "Smelt",
            Self::Assembler => "Asm",
            Self::Machine => "Craft",
            Self::Lab => "Lab",
            Self::Box => "Box",
            Self::FluidTank => "Tank",
            Self::Splitter => "Split",
            Self::Pipe => "Pipe",
            Self::Totem => "Totem",
            Self::Wall => "Wall",
            Self::ReinforcedWall => "RWall",
            Self::BallisticTurret => "Ballistic",
            Self::Turret => "Cannon",
            Self::LaserTurret => "Laser",
            Self::NexusSite => "NSite",
            Self::Nexus => "Nexus",
            Self::PowerWire => "Wire",
            Self::Conveyor => "Belt",
            Self::SpawnAssault => "Assault",
            Self::SpawnHunter => "Eye",
            Self::SpawnSaboteur => "Saboteur",
            Self::SpawnFogcaller => "Fogcall",
            Self::SpawnNest => "Nest",
        }
    }
    pub fn size(self) -> (f32, f32) {
        // Footprints are tile multiples (TILE_SIZE=40) and match silhouettes.
        match self {
            Self::PowerPole => (40.0, 80.0),
            Self::Splitter => (40.0, 120.0), // 1 deep × 3 wide (rotated with facing)
            Self::Totem => (80.0, 120.0),
            Self::Turret | Self::BallisticTurret | Self::LaserTurret => (80.0, 80.0),
            Self::Wall | Self::ReinforcedWall => (40.0, 40.0),
            Self::Solar => (160.0, 120.0),
            Self::OreNode => (120.0, 120.0),
            Self::Smelter | Self::Assembler | Self::Machine | Self::Lab => (160.0, 120.0),
            Self::Box | Self::FluidTank => (120.0, 120.0),
            Self::Pipe => (40.0, 40.0),
            Self::NexusSite => (200.0, 200.0),
            Self::Nexus => (400.0, 400.0),
            // Wire is a link tool; conveyor is a tile brush (no building AABB).
            Self::PowerWire | Self::Conveyor => (40.0, 40.0),
            Self::SpawnAssault
            | Self::SpawnHunter
            | Self::SpawnSaboteur
            | Self::SpawnFogcaller
            | Self::SpawnNest => (40.0, 40.0),
        }
    }
    pub fn needs_power(self) -> bool {
        matches!(
            self,
            Self::OreNode
                | Self::Smelter
                | Self::Assembler
                | Self::Machine
                | Self::Lab
                | Self::Totem
                | Self::Turret
                | Self::BallisticTurret
                | Self::LaserTurret
                | Self::NexusSite
                | Self::FluidTank
        )
    }
    pub fn can_rotate(self) -> bool {
        !matches!(
            self,
            Self::PowerPole
                | Self::Totem
                | Self::Turret
                | Self::BallisticTurret
                | Self::LaserTurret
                | Self::Wall
                | Self::ReinforcedWall
                | Self::Nexus
                | Self::NexusSite
                | Self::PowerWire
                | Self::Conveyor
                | Self::Pipe
                | Self::SpawnAssault
                | Self::SpawnHunter
                | Self::SpawnSaboteur
                | Self::SpawnFogcaller
                | Self::SpawnNest
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
            Self::SpawnAssault => 10,
            Self::SpawnHunter => 11,
            Self::SpawnSaboteur => 12,
            Self::SpawnFogcaller => 13,
            Self::SpawnNest => 14,
            Self::Assembler => 15,
            Self::Machine => 16,
            Self::Lab => 17,
            Self::FluidTank => 18,
            Self::Pipe => 19,
            Self::Wall => 20,
            Self::ReinforcedWall => 21,
            Self::BallisticTurret => 22,
            Self::LaserTurret => 23,
            Self::NexusSite => 24,
            Self::Nexus => 25,
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
            10 => Self::SpawnAssault,
            11 => Self::SpawnHunter,
            12 => Self::SpawnSaboteur,
            13 => Self::SpawnFogcaller,
            14 => Self::SpawnNest,
            15 => Self::Assembler,
            16 => Self::Machine,
            17 => Self::Lab,
            18 => Self::FluidTank,
            19 => Self::Pipe,
            20 => Self::Wall,
            21 => Self::ReinforcedWall,
            22 => Self::BallisticTurret,
            23 => Self::LaserTurret,
            24 => Self::NexusSite,
            25 => Self::Nexus,
            _ => return None,
        })
    }

    /// Tech gate id required to place this building (empty / basic = always).
    pub fn tech_unlock(self) -> &'static str {
        match self {
            Self::OreNode => "era1_tech_basic_extraction",
            Self::Smelter | Self::Machine => "era1_tech_basic_metallurgy",
            Self::Assembler => "era1_tech_industrial_automation",
            Self::Lab => "era1_tech_research_infrastructure",
            Self::FluidTank | Self::Pipe => "era1_tech_fluid_engineering",
            Self::Wall | Self::ReinforcedWall | Self::BallisticTurret => "era1_tech_defense_industry",
            Self::Turret => "era1_tech_defense_research",
            Self::LaserTurret => "era1_tech_laser_defense",
            Self::NexusSite => "era1_tech_nexus_construction",
            Self::Nexus => "era1_tech_era_transition",
            _ => "era1_tech_basic_recovery",
        }
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
    FluidIn,
    FluidOut,
}

impl PortKind {
    pub fn is_energy(self) -> bool {
        matches!(self, Self::EnergyOut | Self::EnergyAny)
    }
    pub fn is_fluid(self) -> bool {
        matches!(self, Self::FluidIn | Self::FluidOut)
    }
    /// True output (pushes power/items).
    pub fn is_output(self) -> bool {
        matches!(
            self,
            Self::EnergyOut | Self::ItemOut(_) | Self::AnyOut | Self::FluidOut
        )
    }
    /// True input (receives).
    pub fn is_input(self) -> bool {
        matches!(self, Self::ItemIn(_) | Self::AnyIn | Self::FluidIn)
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
    /// Turret post-shot recovery (seconds).
    pub cooldown: f32,
    /// Turret gun aim angle (radians; 0 = north / -Y).
    pub aim_angle: f32,
    /// Turret charge 0..1 while locked on a target.
    pub charge: f32,
    /// Cable endpoint ports (PowerWire / Conveyor only).
    pub cable_a: Option<(u32, usize)>,
    pub cable_b: Option<(u32, usize)>,
    pub ports: Vec<Port>,
    /// True while the local player is dragging this building — freezes I/O.
    pub held: bool,
    /// Mining drill: which resource this node extracts.
    pub mine_item: Option<Item>,
    /// Vein this drill is tapping (`Vein.id`).
    pub mine_vein: Option<u32>,
    /// Box storage for non-iron solids / oil (legacy; mirrored into `stocks`).
    pub store_copper: f32,
    pub store_stone: f32,
    pub store_coal: f32,
    pub store_oil: f32,
    /// Generic per-item buffers (dense by interned Item id).
    pub stocks: Vec<f32>,
    /// Mean purity 0..100 for purity-supporting items (parallel to stocks).
    pub stock_purity: Vec<f32>,
    /// Active craft recipe id (`0` = idle / none). Legacy shim or Era recipe index.
    pub craft_recipe: u16,
    /// Seconds elapsed on the current craft.
    pub craft_t: f32,
    /// Era machine def index when `kind == Machine` (or optional override).
    pub machine_id: Option<u16>,
    /// True while crafting via ContentRegistry (vs legacy recipes.rs).
    pub era_craft: bool,
    /// Ballistic turret ammo buffer.
    pub ammo: f32,
    /// Preferred fluid type for tanks/pipes (None = any).
    pub fluid_filter: Option<Item>,
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
            aim_angle: facing_aim_angle(facing),
            charge: 0.0,
            cable_a: None,
            cable_b: None,
            ports: Vec::new(),
            held: false,
            mine_item: None,
            mine_vein: None,
            store_copper: 0.0,
            store_stone: 0.0,
            store_coal: 0.0,
            store_oil: 0.0,
            stocks: vec![0.0; Item::count()],
            stock_purity: vec![50.0; Item::count()],
            craft_recipe: 0,
            craft_t: 0.0,
            machine_id: default_machine_id(kind),
            era_craft: false,
            ammo: 0.0,
            fluid_filter: None,
        };
        n.rebuild_ports();
        n
    }

    pub fn ensure_stock_len(&mut self) {
        let n = Item::count();
        if self.stocks.len() < n {
            self.stocks.resize(n, 0.0);
        }
        if self.stock_purity.len() < n {
            self.stock_purity.resize(n, 50.0);
        }
    }

    pub fn stock(&self, item: Item) -> f32 {
        self.stocks
            .get(item.as_u16() as usize)
            .copied()
            .unwrap_or(0.0)
    }

    pub fn purity(&self, item: Item) -> f32 {
        self.stock_purity
            .get(item.as_u16() as usize)
            .copied()
            .unwrap_or(50.0)
    }

    pub fn stock_mut(&mut self, item: Item) -> &mut f32 {
        self.ensure_stock_len();
        let i = item.as_u16() as usize;
        &mut self.stocks[i]
    }

    pub fn add_stock(&mut self, item: Item, amt: f32) {
        self.add_stock_purity(item, amt, 50.0);
    }

    pub fn add_stock_purity(&mut self, item: Item, amt: f32, purity: f32) {
        if amt <= 0.0 {
            return;
        }
        self.ensure_stock_len();
        let i = item.as_u16() as usize;
        let old = self.stocks[i];
        let new = (old + amt).min(NODE_BUFFER);
        let added = new - old;
        if added > 0.0 {
            let p = self.stock_purity[i];
            self.stock_purity[i] = if new > 1e-4 {
                (p * old + purity * added) / new
            } else {
                purity
            };
        }
        *self.stock_mut(item) = new;
        self.sync_legacy_from_stocks();
    }

    pub fn try_take_stock(&mut self, item: Item, amt: f32) -> bool {
        self.ensure_stock_len();
        let slot = self.stock_mut(item);
        if *slot + 1e-4 < amt {
            return false;
        }
        *slot -= amt;
        self.sync_legacy_from_stocks();
        true
    }

    /// Push generic stocks into legacy iron/box fields for UI + older save paths.
    pub fn sync_legacy_from_stocks(&mut self) {
        self.ensure_stock_len();
        self.in_ore = self.stocks[Item::IronOre.as_u16() as usize];
        self.out_ingot = self.stocks[Item::IronIngot.as_u16() as usize];
        self.store_ore = self.stocks[Item::IronOre.as_u16() as usize];
        self.store_ingot = self.stocks[Item::IronIngot.as_u16() as usize];
        self.store_copper = self.stocks[Item::CopperOre.as_u16() as usize];
        self.store_stone = self.stocks[Item::Stone.as_u16() as usize];
        self.store_coal = self.stocks[Item::Coal.as_u16() as usize];
        self.store_oil = self.stocks[Item::CrudeOil.as_u16() as usize];
    }

    /// Pull legacy fields into stocks (load path / pre-stock code).
    pub fn sync_stocks_from_legacy(&mut self) {
        self.ensure_stock_len();
        // Prefer the richer of legacy machine vs box fields for iron.
        let ore = self.in_ore.max(self.store_ore).max(self.out_ore);
        // out_ore is miner output — leave stocks alone for miners.
        if self.kind != BuildingKind::OreNode {
            self.stocks[Item::IronOre.as_u16() as usize] =
                self.in_ore.max(self.store_ore);
            self.stocks[Item::IronIngot.as_u16() as usize] =
                self.out_ingot.max(self.store_ingot);
        } else {
            let _ = ore;
        }
        self.stocks[Item::CopperOre.as_u16() as usize] = self.store_copper;
        self.stocks[Item::Stone.as_u16() as usize] = self.store_stone;
        self.stocks[Item::Coal.as_u16() as usize] = self.store_coal;
        self.stocks[Item::CrudeOil.as_u16() as usize] = self.store_oil;
    }

    pub fn rebuild_ports(&mut self) {
        let (w, h) = self.size();
        self.ports = ports_for(self.kind, w, h, self.facing, self.mine_item);
    }
    pub fn size(&self) -> (f32, f32) {
        let (bw, bh) = self.footprint();
        match self.facing {
            Facing::E | Facing::W => (bw, bh),
            Facing::N | Facing::S => (bh, bw),
        }
    }

    /// Unrotated footprint — Era machines use data-pack tile sizes.
    pub fn footprint(&self) -> (f32, f32) {
        if let Some(mid) = self.machine_id {
            if let Some(m) = content::try_content().and_then(|c| c.machine(mid)) {
                let w = (m.size_tiles[0].max(1) as f32) * TILE_SIZE;
                let h = (m.size_tiles[1].max(1) as f32) * TILE_SIZE;
                return (w, h);
            }
        }
        self.kind.size()
    }

    pub fn set_machine_id(&mut self, mid: Option<u16>) {
        self.machine_id = mid;
        self.rebuild_ports();
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

fn ports_for(
    kind: BuildingKind,
    w: f32,
    h: f32,
    facing: Facing,
    mine_item: Option<Item>,
) -> Vec<Port> {
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
                kind: PortKind::ItemOut(mine_item.unwrap_or(Item::IronOre)),
                ox,
                oy,
            }]
        }
        BuildingKind::Smelter
        | BuildingKind::Assembler
        | BuildingKind::Machine
        | BuildingKind::Lab
        | BuildingKind::NexusSite => {
            let i = edge(w, h, back, m);
            let o = edge(w, h, facing, m);
            let mut ports = vec![
                Port {
                    kind: PortKind::AnyIn,
                    ox: i.0,
                    oy: i.1,
                },
                Port {
                    kind: PortKind::AnyOut,
                    ox: o.0,
                    oy: o.1,
                },
            ];
            // Fluid side ports for machines that declare fluid IO.
            let f0 = edge(w, h, Facing::N, 0.25);
            let f1 = edge(w, h, Facing::S, 0.25);
            ports.push(Port {
                kind: PortKind::FluidIn,
                ox: f0.0,
                oy: f0.1,
            });
            ports.push(Port {
                kind: PortKind::FluidOut,
                ox: f1.0,
                oy: f1.1,
            });
            ports
        }
        BuildingKind::Box | BuildingKind::FluidTank => {
            let (ox, oy) = edge(w, h, back, m);
            let kind = if matches!(kind, BuildingKind::FluidTank) {
                PortKind::FluidIn
            } else {
                PortKind::AnyIn
            };
            vec![Port { kind, ox, oy }]
        }
        BuildingKind::Pipe => {
            let a = edge(w, h, back, m);
            let b = edge(w, h, facing, m);
            vec![
                Port {
                    kind: PortKind::FluidIn,
                    ox: a.0,
                    oy: a.1,
                },
                Port {
                    kind: PortKind::FluidOut,
                    ox: b.0,
                    oy: b.1,
                },
            ]
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
        BuildingKind::Totem | BuildingKind::Turret | BuildingKind::LaserTurret => {
            let (ox, oy) = edge(w, h, facing, m);
            vec![Port {
                kind: PortKind::EnergyAny,
                ox,
                oy,
            }]
        }
        BuildingKind::BallisticTurret => {
            let e = edge(w, h, facing, m);
            let i = edge(w, h, back, m);
            let ammo = content()
                .item_index("era1_military_standard_ammunition")
                .map(Item::from_u16)
                .unwrap_or(Item::ShellCasing);
            vec![
                Port {
                    kind: PortKind::EnergyAny,
                    ox: e.0,
                    oy: e.1,
                },
                Port {
                    kind: PortKind::ItemIn(ammo),
                    ox: i.0,
                    oy: i.1,
                },
            ]
        }
        BuildingKind::Wall
        | BuildingKind::ReinforcedWall
        | BuildingKind::Nexus
        | BuildingKind::PowerWire
        | BuildingKind::Conveyor
        | BuildingKind::SpawnAssault
        | BuildingKind::SpawnHunter
        | BuildingKind::SpawnSaboteur
        | BuildingKind::SpawnFogcaller
        | BuildingKind::SpawnNest => Vec::new(),
    }
}

fn default_machine_id(kind: BuildingKind) -> Option<u16> {
    let id = match kind {
        BuildingKind::Smelter => "era1_machine_thermal_smelter_mk1",
        BuildingKind::Assembler => "era1_machine_assembler_mk1",
        BuildingKind::Lab => "era1_machine_research_laboratory",
        BuildingKind::Machine => "era1_machine_crusher_mk1",
        BuildingKind::NexusSite => "era1_machine_construction_site",
        BuildingKind::BallisticTurret => "era1_machine_ballistic_turret",
        BuildingKind::LaserTurret => "era1_machine_laser_turret",
        BuildingKind::Turret => "era1_machine_charge_cannon",
        BuildingKind::FluidTank => "era1_machine_fluid_tank",
        _ => return None,
    };
    content::try_content().and_then(|c| c.machine_index(id))
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

pub fn dist_to_polyline(px: f32, py: f32, pts: &[(f32, f32)]) -> f32 {
    if pts.is_empty() {
        return f32::MAX;
    }
    if pts.len() == 1 {
        return (px - pts[0].0).hypot(py - pts[0].1);
    }
    let mut best = f32::MAX;
    for w in pts.windows(2) {
        best = best.min(dist_point_segment(px, py, w[0].0, w[0].1, w[1].0, w[1].1));
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

impl Nest {
    /// Named Era 1 threat ladder from evolution 0..1.
    pub fn threat_tier(&self) -> &'static str {
        match self.evolution {
            e if e < 0.2 => "Scouts",
            e if e < 0.4 => "Raiders",
            e if e < 0.6 => "Breachers",
            e if e < 0.8 => "Siege Pack",
            _ => "Storm Host",
        }
    }

    /// Extra raider HP multiplier by threat tier.
    pub fn threat_hp_mult(&self) -> f32 {
        1.0 + self.evolution * 1.4
    }
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
    /// Facing for Eye draw / muzzle (radians, 0 = +X).
    pub aim_angle: f32,
    /// Seconds remaining on barrel recoil animation.
    pub recoil_t: f32,
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
    /// Hunter (Eye) death positions for pixel-blood FX.
    pub hunter_deaths: Vec<(f32, f32)>,
}

#[derive(Clone, Debug)]
pub struct CombatShot {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub life: f32,
    pub max_life: f32,
    /// 0 = thin tracer, 1 = charge cannon bolt.
    pub style: u8,
}

pub(crate) fn facing_aim_angle(facing: Facing) -> f32 {
    match facing {
        Facing::E => std::f32::consts::FRAC_PI_2,
        Facing::W => -std::f32::consts::FRAC_PI_2,
        Facing::S => std::f32::consts::PI,
        Facing::N => 0.0,
    }
}

pub(crate) fn aim_angle_from_dir(dx: f32, dy: f32) -> f32 {
    dx.atan2(-dy)
}

fn angle_diff(from: f32, to: f32) -> f32 {
    let mut d = to - from;
    let pi = std::f32::consts::PI;
    let tau = std::f32::consts::TAU;
    while d > pi {
        d -= tau;
    }
    while d < -pi {
        d += tau;
    }
    d
}

fn rotate_toward(current: f32, target: f32, max_delta: f32) -> f32 {
    let d = angle_diff(current, target);
    current + d.clamp(-max_delta, max_delta)
}

fn aim_unit(angle: f32) -> (f32, f32) {
    // 0 = north (-Y), positive clockwise toward east (+X) to match art rotation.
    (angle.sin(), -angle.cos())
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
    /// Freehand/corner world-space route (includes endpoints). Empty → Manhattan fallback.
    pub path: Vec<(f32, f32)>,
}

pub struct World {
    pub nodes: HashMap<u32, Node>,
    pub links: Vec<Link>,
    /// Factorio-style belt tile grid.
    pub belt_tiles: BeltGrid,
    /// Living resource veins (pressure shared by drills).
    pub veins: Vec<crate::deposits::Vein>,
    pub next_vein_id: u32,
    pub next_id: u32,
    pub network_energy: HashMap<u32, f32>,
    pub nests: Vec<Nest>,
    pub raiders: Vec<Raider>,
    pub combat_shots: Vec<CombatShot>,
    pub storm_blots: Vec<StormBlot>,
    pub next_nest_id: u32,
    pub next_raider_id: u32,
    pub next_wave_id: u32,
    /// Era 1 technology / Nexus campaign state.
    pub tech: TechState,
    /// Toast when a tech completes (UI drains).
    pub tech_completed: Option<String>,
    /// Milestone banner when Nexus commissions.
    pub era1_complete: bool,
}

impl World {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            links: Vec::new(),
            belt_tiles: HashMap::new(),
            veins: Vec::new(),
            next_vein_id: 1,
            next_id: 1,
            network_energy: HashMap::new(),
            nests: Vec::new(),
            raiders: Vec::new(),
            combat_shots: Vec::new(),
            storm_blots: Vec::new(),
            next_nest_id: 1,
            next_raider_id: 1,
            next_wave_id: 1,
            tech: TechState::default(),
            tech_completed: None,
            era1_complete: false,
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

    pub fn seed_deposits(&mut self, storm_cx: f32, storm_cy: f32, storm_safe_r: f32) {
        self.veins = crate::deposits::seed_veins(storm_cx, storm_cy, storm_safe_r);
        self.next_vein_id = self
            .veins
            .iter()
            .map(|v| v.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
            .max(1);
    }

    pub fn vein_index(&self, id: u32) -> Option<usize> {
        self.veins.iter().position(|v| v.id == id)
    }

    /// Best vein overlapping the footprint (highest current yield).
    pub fn vein_under(&self, x: f32, y: f32, w: f32, h: f32) -> Option<usize> {
        self.veins
            .iter()
            .enumerate()
            .filter(|(_, v)| v.yield_pct > 1.0 && v.overlaps_rect(x, y, w, h))
            .max_by(|a, b| {
                a.1.yield_pct
                    .partial_cmp(&b.1.yield_pct)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
    }

    pub fn has_ore_under(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        self.vein_under(x, y, w, h).is_some()
    }

    pub fn bind_miner(&mut self, id: u32) -> bool {
        let (x, y, w, h) = match self.nodes.get(&id) {
            Some(n) if n.kind == BuildingKind::OreNode => (n.x, n.y, n.w(), n.h()),
            _ => return false,
        };
        let Some(vi) = self.vein_under(x, y, w, h) else {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.mine_item = None;
                n.mine_vein = None;
                n.rebuild_ports();
            }
            return false;
        };
        let vein_id = self.veins[vi].id;
        let item = self.veins[vi].kind.item();
        if let Some(n) = self.nodes.get_mut(&id) {
            n.mine_item = Some(item);
            n.mine_vein = Some(vein_id);
            n.rebuild_ports();
            true
        } else {
            false
        }
    }

    /// Refresh storm clear factors + tap counts before mining.
    pub fn refresh_veins(&mut self, clear_zones: &[(f32, f32, f32)]) {
        for v in &mut self.veins {
            v.taps = 0;
            let clear = point_in_hard_clear(v.x, v.y, clear_zones);
            // Fully open in clear; choked (not dead) under storm so overbuilt fog drills limp.
            v.clear_factor = if clear { 1.0 } else { 0.12 };
        }
        let tap_ids: Vec<u32> = self
            .nodes
            .values()
            .filter(|n| n.kind == BuildingKind::OreNode && !n.held)
            .filter_map(|n| n.mine_vein)
            .collect();
        for vid in tap_ids {
            if let Some(v) = self.veins.iter_mut().find(|v| v.id == vid) {
                v.taps = v.taps.saturating_add(1);
            }
        }
    }

    pub fn set_id_namespace(&mut self, player_id: u8) {
        let base = (player_id as u32 + 1) * 1_000_000;
        if self.next_id < base {
            self.next_id = base;
        }
    }

    pub fn place_node(&mut self, kind: BuildingKind, x: f32, y: f32, facing: Facing) -> Option<u32> {
        self.place_node_machine(kind, None, x, y, facing)
    }

    pub fn place_node_machine(
        &mut self,
        kind: BuildingKind,
        machine_id: Option<u16>,
        x: f32,
        y: f32,
        facing: Facing,
    ) -> Option<u32> {
        if kind.is_cable() || kind.is_belt_tool() || kind.is_debug_tool() {
            return None;
        }
        let mut probe = Node::new(kind, x, y, facing);
        if let Some(mid) = machine_id {
            probe.set_machine_id(Some(mid));
        }
        if self.collides(probe.x, probe.y, probe.w(), probe.h(), None) {
            return None;
        }
        if kind == BuildingKind::OreNode {
            if !self.has_ore_under(probe.x, probe.y, probe.w(), probe.h()) {
                return None;
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, probe);
        if kind == BuildingKind::OreNode {
            let _ = self.bind_miner(id);
        }
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
        if kind == BuildingKind::OreNode {
            let _ = self.bind_miner(id);
        }
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
        self.retarget_wire_path_ends(id);
        self.sync_cable_anchors();
        if self.nodes.get(&id).map(|n| n.kind) == Some(BuildingKind::OreNode) {
            let _ = self.bind_miner(id);
        }
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
        self.retarget_wire_path_ends(id);
        self.sync_cable_anchors();
    }

    /// Keep routed wire endpoints glued to their ports after a building moves.
    fn retarget_wire_path_ends(&mut self, moved_id: u32) {
        let updates: Vec<(usize, Option<(f32, f32)>, Option<(f32, f32)>)> = self
            .links
            .iter()
            .enumerate()
            .filter(|(_, l)| {
                !l.path.is_empty() && (l.from_node == moved_id || l.to_node == moved_id)
            })
            .filter_map(|(i, l)| {
                let a = self
                    .nodes
                    .get(&l.from_node)
                    .and_then(|n| n.port_world(l.from_port));
                let b = self
                    .nodes
                    .get(&l.to_node)
                    .and_then(|n| n.port_world(l.to_port));
                Some((i, a, b))
            })
            .collect();
        for (i, a, b) in updates {
            if let Some(l) = self.links.get_mut(i) {
                if let (Some(a), Some(first)) = (a, l.path.first_mut()) {
                    *first = a;
                }
                if let (Some(b), Some(last)) = (b, l.path.last_mut()) {
                    *last = b;
                }
            }
        }
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
        self.hit_port_where(wx, wy, radius, |_| true)
    }

    /// Nearest energy port within `radius`, or None.
    pub fn nearest_energy_port(&self, wx: f32, wy: f32, radius: f32) -> Option<(u32, usize)> {
        // Prefer energy-only scan so a closer item/fluid port doesn't steal the snap.
        self.hit_port_where(wx, wy, radius, |p| p.kind.is_energy())
            .or_else(|| {
                self.hit_port(wx, wy, radius)
                    .filter(|&(id, pi)| {
                        self.nodes
                            .get(&id)
                            .and_then(|n| n.ports.get(pi))
                            .map(|p| p.kind.is_energy())
                            .unwrap_or(false)
                    })
            })
    }

    fn hit_port_where(
        &self,
        wx: f32,
        wy: f32,
        radius: f32,
        pred: impl Fn(&Port) -> bool,
    ) -> Option<(u32, usize)> {
        let r2 = radius * radius;
        let mut best = None;
        for (&id, n) in &self.nodes {
            if n.kind.is_cable() || n.held {
                continue;
            }
            for (pi, p) in n.ports.iter().enumerate() {
                if !pred(p) {
                    continue;
                }
                let Some((px, py)) = n.port_world(pi) else {
                    continue;
                };
                let d2 = (px - wx).powi(2) + (py - wy).powi(2);
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
    ) -> Option<&'static str> {
        match (
            self.power_connect_fail(from, to),
            self.power_connect_fail(to, from),
        ) {
            (None, _) | (_, None) => None,
            (Some(a), Some(b)) => Some(prefer_connect_hint(a, b)),
        }
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
            path: Vec::new(),
        });
        true
    }

    /// Connect two energy ports with an optional cornered route.
    pub fn connect_power_path(
        &mut self,
        from: (u32, usize),
        to: (u32, usize),
        path: Vec<(f32, f32)>,
    ) -> bool {
        let before = self.links.len();
        if !self.connect_power(from, to) {
            return false;
        }
        if let Some(l) = self.links.get_mut(before) {
            l.path = path;
        }
        true
    }

    pub fn set_node_held(&mut self, id: u32, held: bool) {
        if let Some(n) = self.nodes.get_mut(&id) {
            n.held = held;
            if held {
                n.working = false;
            }
        }
    }

    /// Remove the nearest routed/manhattan power wire under the cursor.
    pub fn remove_wire_at(&mut self, wx: f32, wy: f32) -> bool {
        let mut best: Option<(usize, f32)> = None;
        for (i, l) in self.links.iter().enumerate() {
            let dist = if l.path.len() >= 2 {
                dist_to_polyline(wx, wy, &l.path)
            } else {
                let Some(a) = self.nodes.get(&l.from_node) else {
                    continue;
                };
                let Some(b) = self.nodes.get(&l.to_node) else {
                    continue;
                };
                let Some((ax, ay)) = a.port_world(l.from_port) else {
                    continue;
                };
                let Some((bx, by)) = b.port_world(l.to_port) else {
                    continue;
                };
                dist_to_manhattan(wx, wy, ax, ay, bx, by)
            };
            if dist <= WIRE_PATH_HIT && best.map(|(_, bd)| dist < bd).unwrap_or(true) {
                best = Some((i, dist));
            }
        }
        let Some((idx, _)) = best else {
            return false;
        };
        let cable_id = self.links[idx].cable_id;
        self.links.remove(idx);
        self.nodes.remove(&cable_id);
        true
    }

    pub fn tick(&mut self, dt: f32, clear_zones: &[(f32, f32, f32)]) {
        self.refresh_veins(clear_zones);
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
        let mut eye_shots: Vec<CombatShot> = Vec::new();
        for raider in &mut self.raiders {
            if raider.hp <= 0.0 {
                continue;
            }
            raider.recoil_t = (raider.recoil_t - dt).max(0.0);
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
                if d2 < SWARM_SEP_RADIUS * SWARM_SEP_RADIUS {
                    let d = d2.sqrt();
                    sep_x += dx / d;
                    sep_y += dy / d;
                }
                if d2 < SWARM_COH_RADIUS * SWARM_COH_RADIUS {
                    coh_x += ox;
                    coh_y += oy;
                    coh_n += 1.0;
                }
            }
            if coh_n > 0.0 {
                coh_x = coh_x / coh_n - raider.x;
                coh_y = coh_y / coh_n - raider.y;
            }

            let to_tx = tx - raider.x;
            let to_ty = ty - raider.y;
            let dist = (to_tx * to_tx + to_ty * to_ty).sqrt().max(0.001);
            if dist > 4.0 {
                raider.aim_angle = to_ty.atan2(to_tx);
            } else if raider.vx * raider.vx + raider.vy * raider.vy > 4.0 {
                raider.aim_angle = raider.vy.atan2(raider.vx);
            }

            let engage = if raider.role == RaiderRole::Hunter {
                EYE_CANNON_RANGE
            } else {
                RAIDER_ATTACK_RANGE
            };

            // Seek target with flocking; stop once in engage range.
            let inv = 1.0 / dist;
            let mut ax = to_tx * inv;
            let mut ay = to_ty * inv;
            ax += sep_x * 0.85 + coh_x * 0.012;
            ay += sep_y * 0.85 + coh_y * 0.012;
            let alen = (ax * ax + ay * ay).sqrt().max(0.001);
            ax /= alen;
            ay /= alen;

            let speed = RAIDER_SPEED
                * match raider.role {
                    // Hunters rush defenses a bit harder; saboteurs slightly sneakier (slower).
                    RaiderRole::Assault => 1.0,
                    RaiderRole::Hunter => 1.12,
                    RaiderRole::Saboteur => 0.92,
                    RaiderRole::Fogcaller => 0.72,
                };
            if dist > engage {
                raider.vx = ax * speed;
                raider.vy = ay * speed;
            } else {
                raider.vx *= 0.55;
                raider.vy *= 0.55;
            }
            let vlen = (raider.vx * raider.vx + raider.vy * raider.vy)
                .sqrt()
                .max(0.001);
            if vlen > speed {
                raider.vx *= speed / vlen;
                raider.vy *= speed / vlen;
            }

            if dist > engage {
                raider.x += raider.vx * dt;
                raider.y += raider.vy * dt;
            } else if let Some(tid) = raider.target_node {
                raider.vx *= 0.45;
                raider.vy *= 0.45;
                raider.attack_cd -= dt;
                if raider.attack_cd <= 0.0 {
                    if raider.role == RaiderRole::Hunter {
                        raider.attack_cd = EYE_CANNON_INTERVAL;
                        raider.recoil_t = EYE_RECOIL_TIME;
                        let dmg = RAIDER_DAMAGE * 1.15;
                        damage.push((tid, dmg));
                        let muzzle_r = RAIDER_RADIUS * 1.6;
                        let mx = raider.x + raider.aim_angle.cos() * muzzle_r;
                        let my = raider.y + raider.aim_angle.sin() * muzzle_r;
                        eye_shots.push(CombatShot {
                            x0: mx,
                            y0: my,
                            x1: tx,
                            y1: ty,
                            life: 0.35,
                            max_life: 0.35,
                            style: SHOT_STYLE_EYE,
                        });
                    } else {
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
        }
        self.storm_blots.extend(new_blots);
        self.combat_shots.extend(eye_shots);

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

        let turret_ids: Vec<(u32, BuildingKind)> = self
            .nodes
            .iter()
            .filter_map(|(&id, n)| {
                if matches!(
                    n.kind,
                    BuildingKind::Turret | BuildingKind::BallisticTurret | BuildingKind::LaserTurret
                ) && n.powered
                {
                    Some((id, n.kind))
                } else {
                    None
                }
            })
            .collect();
        for (tid, tkind) in turret_ids {
            let (tcx, tcy, mut cd, mut aim, mut charge, ammo) = {
                let Some(n) = self.nodes.get(&tid) else {
                    continue;
                };
                let (cx, cy) = n.center();
                (cx, cy, n.cooldown, n.aim_angle, n.charge, n.ammo)
            };
            if tkind == BuildingKind::BallisticTurret && ammo < 1.0 {
                continue;
            }
            cd = (cd - dt).max(0.0);

            let range2 = match tkind {
                BuildingKind::LaserTurret => (TURRET_RANGE * 1.15).powi(2),
                BuildingKind::BallisticTurret => (TURRET_RANGE * 0.9).powi(2),
                _ => TURRET_RANGE * TURRET_RANGE,
            };
            let damage = match tkind {
                BuildingKind::LaserTurret => TURRET_DAMAGE * 0.85,
                BuildingKind::BallisticTurret => TURRET_DAMAGE * 0.55,
                _ => TURRET_DAMAGE,
            };
            let charge_time = match tkind {
                BuildingKind::BallisticTurret => 0.35,
                BuildingKind::LaserTurret => 0.9,
                _ => TURRET_CHARGE_TIME,
            };
            enum TurretTarget {
                Raider(usize),
                Nest(usize),
            }
            let mut best_d2 = f32::INFINITY;
            let mut best: Option<(TurretTarget, f32, f32)> = None; // tgt, x, y
            for (i, r) in self.raiders.iter().enumerate() {
                if r.hp <= 0.0 {
                    continue;
                }
                let d2 = (r.x - tcx).powi(2) + (r.y - tcy).powi(2);
                if d2 <= range2 && d2 < best_d2 {
                    best_d2 = d2;
                    best = Some((TurretTarget::Raider(i), r.x, r.y));
                }
            }
            if best.is_none() {
                for (i, nest) in self.nests.iter().enumerate() {
                    if nest.hp <= 0.0 || !point_in_hard_clear(nest.x, nest.y, clear_zones) {
                        continue;
                    }
                    let d2 = (nest.x - tcx).powi(2) + (nest.y - tcy).powi(2);
                    if d2 <= range2 && d2 < best_d2 {
                        best_d2 = d2;
                        best = Some((TurretTarget::Nest(i), nest.x, nest.y));
                    }
                }
            }

            let mut shot_to: Option<(f32, f32)> = None;
            let had_target = best.is_some();
            let mut muzzle = {
                let (ux, uy) = aim_unit(aim);
                (tcx + ux * 36.0, tcy + uy * 36.0)
            };
            if let Some((tgt, tx, ty)) = best {
                let desired = aim_angle_from_dir(tx - tcx, ty - tcy);
                aim = rotate_toward(aim, desired, TURRET_TURN_RATE * dt);
                let (ux, uy) = aim_unit(aim);
                muzzle = (tcx + ux * 36.0, tcy + uy * 36.0);
                let locked = angle_diff(aim, desired).abs() <= TURRET_AIM_LOCK;
                if locked && cd <= 0.0 {
                    charge = (charge + dt / charge_time).min(1.0);
                    if charge >= 1.0 {
                        match tgt {
                            TurretTarget::Raider(i) => {
                                if let Some(r) = self.raiders.get_mut(i) {
                                    r.hp -= damage;
                                    shot_to = Some((r.x, r.y));
                                }
                            }
                            TurretTarget::Nest(i) => {
                                if let Some(nest) = self.nests.get_mut(i) {
                                    nest.hp -= damage;
                                    nest.anger += 40.0;
                                    nest.wave_cd = nest.wave_cd.min(1.2);
                                    shot_to = Some((nest.x, nest.y));
                                }
                            }
                        }
                        charge = 0.0;
                        cd = if tkind == BuildingKind::BallisticTurret {
                            0.28
                        } else {
                            TURRET_FIRE_INTERVAL
                        };
                        if tkind == BuildingKind::BallisticTurret {
                            if let Some(n) = self.nodes.get_mut(&tid) {
                                n.ammo = (n.ammo - 1.0).max(0.0);
                            }
                        }
                    }
                } else if !locked {
                    charge = (charge - dt * 0.85).max(0.0);
                }
            } else {
                charge = (charge - dt * 1.2).max(0.0);
            }

            if let Some(n) = self.nodes.get_mut(&tid) {
                n.cooldown = cd;
                n.aim_angle = aim;
                n.charge = charge;
                n.working = n.powered && (had_target || charge > 0.05 || cd > 0.0);
            }
            if let Some((x1, y1)) = shot_to {
                self.combat_shots.push(CombatShot {
                    x0: muzzle.0,
                    y0: muzzle.1,
                    x1,
                    y1,
                    life: TURRET_SHOT_LIFE,
                    max_life: TURRET_SHOT_LIFE,
                    style: 1,
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

        report.hunter_deaths = self
            .raiders
            .iter()
            .filter(|r| r.hp <= 0.0 && r.role == RaiderRole::Hunter)
            .map(|r| (r.x, r.y))
            .collect();

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

        let evo_mult = self
            .nests
            .iter()
            .find(|n| (n.x - nest_x).abs() < 1.0 && (n.y - nest_y).abs() < 1.0)
            .map(|n| n.threat_hp_mult())
            .unwrap_or(1.0);

        // Mixed composition: assault, hunters, saboteurs, fogcallers.
        // Higher evolution → more hunters/saboteurs (mid threat needs ammo defense).
        for i in 0..count {
            let role = match (i + (evo_mult * 3.0) as usize) % 6 {
                0 | 1 => RaiderRole::Assault,
                2 | 3 => RaiderRole::Hunter,
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
                RAIDER_HP * 1.45 * evo_mult
            } else {
                RAIDER_HP * evo_mult
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
                aim_angle: ang,
                recoil_t: 0.0,
            });
        }
    }

    /// Build-menu Debug tools — spawn a single raider or nest at a world point.
    pub fn spawn_debug_at(
        &mut self,
        kind: BuildingKind,
        x: f32,
        y: f32,
        clear_zones: &[(f32, f32, f32)],
    ) -> bool {
        if kind == BuildingKind::SpawnNest {
            let id = self.next_nest_id;
            self.next_nest_id = self.next_nest_id.wrapping_add(1).max(1);
            self.nests.push(Nest {
                id,
                x,
                y,
                hp: NEST_HP,
                max_hp: NEST_HP,
                active: true,
                wave_cd: 2.0,
                evolution: 0.15,
                anger: 40.0,
                first_wave: true,
                dormant_hate: false,
            });
            return true;
        }
        let role = match kind {
            BuildingKind::SpawnAssault => RaiderRole::Assault,
            BuildingKind::SpawnHunter => RaiderRole::Hunter,
            BuildingKind::SpawnSaboteur => RaiderRole::Saboteur,
            BuildingKind::SpawnFogcaller => RaiderRole::Fogcaller,
            _ => return false,
        };
        if self.raiders.len() >= MAX_RAIDERS {
            return false;
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
        let target = Self::pick_target_among(&centers, &kinds, clear_zones, x, y, role);
        let id = self.next_raider_id;
        self.next_raider_id = self.next_raider_id.wrapping_add(1).max(1);
        let wave_id = self.next_wave_id;
        self.next_wave_id = self.next_wave_id.wrapping_add(1).max(1);
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
            attack_cd: 0.1,
            wave_id,
            vx: 0.0,
            vy: 0.0,
            role,
            retarget_cd: 0.2,
            aim_angle: -std::f32::consts::FRAC_PI_2,
            recoil_t: 0.0,
        });
        true
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
        for (&root, &gen) in &gen_by_net {
            let e = self.network_energy.entry(root).or_insert(0.0);
            *e = (*e + gen * dt).min(2000.0);
        }
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
            let held = self.nodes.get(&id).map(|n| n.held).unwrap_or(false);
            if held {
                if let Some(n) = self.nodes.get_mut(&id) {
                    n.working = false;
                }
                continue;
            }
            match kind {
                Some(BuildingKind::OreNode) if powered => {
                    if let Some(root) = pay {
                        let cost = ORE_POWER_DRAW * dt;
                        let has = self.network_energy.get(&root).copied().unwrap_or(0.0);
                        if has >= cost {
                            let (out_space, mine_vein) = {
                                let Some(n) = self.nodes.get(&id) else {
                                    continue;
                                };
                                ((NODE_BUFFER - n.out_ore).max(0.0), n.mine_vein)
                            };
                            if mine_vein.is_none() || out_space <= 0.0 {
                                if let Some(n) = self.nodes.get_mut(&id) {
                                    n.working = false;
                                }
                            } else if let Some(vid) = mine_vein {
                                let Some(vi) = self.vein_index(vid) else {
                                    if let Some(n) = self.nodes.get_mut(&id) {
                                        n.working = false;
                                    }
                                    let _ = self.bind_miner(id);
                                    continue;
                                };
                                if self.veins[vi].yield_pct <= 1.0 {
                                    if let Some(n) = self.nodes.get_mut(&id) {
                                        n.working = false;
                                    }
                                    let _ = self.bind_miner(id);
                                    continue;
                                }
                                let rate = self.veins[vi].rate_per_tap(ORE_RATE);
                                let want = (rate * dt).min(out_space);
                                let purity = self.veins[vi].purity;
                                let made = self.veins[vi].extract(want);
                                if made > 0.0 {
                                    if let Some(n) = self.nodes.get_mut(&id) {
                                        n.out_ore += made;
                                        if let Some(item) = n.mine_item {
                                            n.add_stock_purity(item, 0.0, purity); // set purity mean
                                            let i = item.as_u16() as usize;
                                            n.ensure_stock_len();
                                            n.stock_purity[i] = purity;
                                        }
                                        n.working = true;
                                    }
                                    if let Some(e) = self.network_energy.get_mut(&root) {
                                        *e -= cost * (made / (ORE_RATE * dt).max(1e-4)).min(1.0);
                                    }
                                } else if let Some(n) = self.nodes.get_mut(&id) {
                                    n.working = false;
                                }
                            }
                        } else if let Some(n) = self.nodes.get_mut(&id) {
                            n.working = false;
                        }
                    }
                }
                Some(BuildingKind::Smelter) if powered => {
                    if !self.step_era_crafter(id, pay, dt) {
                        self.step_crafter(id, pay, dt, MachineKind::Smelt);
                    }
                }
                Some(BuildingKind::Assembler) if powered => {
                    if !self.step_era_crafter(id, pay, dt) {
                        self.step_crafter(id, pay, dt, MachineKind::Assemble);
                    }
                }
                Some(BuildingKind::Machine) if powered => {
                    let _ = self.step_era_crafter(id, pay, dt);
                }
                Some(BuildingKind::Lab) if powered => {
                    self.step_lab(id, pay, dt);
                }
                Some(BuildingKind::NexusSite) if powered => {
                    self.step_nexus_site(id, pay, dt);
                }
                Some(BuildingKind::FluidTank) | Some(BuildingKind::Pipe) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.working = n.stocks.iter().any(|&v| v > 0.05);
                    }
                }
                Some(BuildingKind::Splitter) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.working = n.buf_ore + n.buf_ingot > 0.05;
                    }
                }
                Some(BuildingKind::Turret)
                | Some(BuildingKind::BallisticTurret)
                | Some(BuildingKind::LaserTurret)
                    if powered =>
                {
                    if let Some(root) = pay {
                        let cost = TURRET_POWER_DRAW * dt;
                        let has = self.network_energy.get(&root).copied().unwrap_or(0.0);
                        if has >= cost {
                            if let Some(e) = self.network_energy.get_mut(&root) {
                                *e -= cost;
                            }
                            if let Some(n) = self.nodes.get_mut(&id) {
                                // Ballistic needs ammo in buffer (fed from stocks).
                                if n.kind == BuildingKind::BallisticTurret {
                                    if n.ammo < 1.0 {
                                        let ammo = Item::from_u16(
                                            content()
                                                .item_index("era1_military_standard_ammunition")
                                                .unwrap_or(Item::ShellCasing.as_u16()),
                                        );
                                        if n.try_take_stock(ammo, 1.0) {
                                            n.ammo += 10.0;
                                        }
                                    }
                                    n.working = n.ammo >= 1.0;
                                } else {
                                    n.working = true;
                                }
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
                Some(BuildingKind::Nexus) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.working = true;
                    }
                    if !self.era1_complete {
                        self.era1_complete = true;
                        self.tech.nexus_complete = true;
                        self.tech.era2_unlocked = true;
                    }
                }
                _ => {}
            }
        }

        self.step_fluid_transfer(dt);
    }

    /// Transfer fluids between adjacent FluidIn/Out ports on power/item links and pipes.
    fn step_fluid_transfer(&mut self, dt: f32) {
        let rate = 12.0 * dt;
        let links: Vec<(u32, usize, u32, usize)> = self
            .links
            .iter()
            .map(|l| (l.from_node, l.from_port, l.to_node, l.to_port))
            .collect();
        for (a, ap, b, bp) in links {
            let (Some(na), Some(nb)) = (self.nodes.get(&a), self.nodes.get(&b)) else {
                continue;
            };
            let (Some(pa), Some(pb)) = (na.ports.get(ap), nb.ports.get(bp)) else {
                continue;
            };
            if !(pa.kind.is_fluid() && pb.kind.is_fluid()) {
                continue;
            }
            // Push from FluidOut → FluidIn
            let (src, dst) = match (pa.kind, pb.kind) {
                (PortKind::FluidOut, PortKind::FluidIn) => (a, b),
                (PortKind::FluidIn, PortKind::FluidOut) => (b, a),
                _ => continue,
            };
            // Move any fluid stock present.
            let move_item = {
                let n = self.nodes.get(&src).unwrap();
                n.stocks
                    .iter()
                    .enumerate()
                    .find(|&(i, &v)| v > 0.05 && Item::from_u16(i as u16).is_fluid())
                    .map(|(i, _)| Item::from_u16(i as u16))
            };
            let Some(item) = move_item else {
                continue;
            };
            // Respect destination tank/pipe lock-in filter.
            if let Some(n) = self.nodes.get(&dst) {
                if let Some(f) = n.fluid_filter {
                    if f != item {
                        continue;
                    }
                }
            }
            let avail = self.nodes.get(&src).map(|n| n.stock(item)).unwrap_or(0.0);
            let space = NODE_BUFFER
                - self.nodes.get(&dst).map(|n| n.stock(item)).unwrap_or(0.0);
            let amt = rate.min(avail).min(space.max(0.0));
            if amt <= 1e-4 {
                continue;
            }
            let purity = self.nodes.get(&src).map(|n| n.purity(item)).unwrap_or(50.0);
            if let Some(n) = self.nodes.get_mut(&src) {
                let _ = n.try_take_stock(item, amt);
            }
            if let Some(n) = self.nodes.get_mut(&dst) {
                if n.fluid_filter.is_none() && item.is_fluid() {
                    n.fluid_filter = Some(item);
                }
                n.add_stock_purity(item, amt, purity);
            }
        }
    }

    fn step_lab(&mut self, id: u32, pay: Option<u32>, dt: f32) {
        let Some(root) = pay else {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.working = false;
            }
            return;
        };
        let cost = 3.0 * dt;
        let has = self.network_energy.get(&root).copied().unwrap_or(0.0);
        if has < cost {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.working = false;
            }
            return;
        }
        if let Some(e) = self.network_energy.get_mut(&root) {
            *e -= cost;
        }
        let completed = {
            let tech = &mut self.tech;
            let nodes = &mut self.nodes;
            tech.tick_lab(dt, &mut |item, amt| {
                nodes
                    .get_mut(&id)
                    .map(|n| n.try_take_stock(item, amt))
                    .unwrap_or(false)
            })
        };
        if let Some(tid) = completed {
            self.tech_completed = Some(tid);
        }
        if let Some(n) = self.nodes.get_mut(&id) {
            n.working = self.tech.active.is_some();
        }
    }

    fn step_nexus_site(&mut self, id: u32, pay: Option<u32>, dt: f32) {
        let Some(root) = pay else {
            return;
        };
        let cost = 20.0 * dt;
        let has = self.network_energy.get(&root).copied().unwrap_or(0.0);
        if has < cost || !self.tech.is_researched("era1_tech_nexus_construction") {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.working = false;
            }
            return;
        }
        // Consume structural frames + plates toward completion.
        let frame = Item::Gear; // aliased structural frame
        let plate = Item::IronIngot;
        let need_frame = 0.05 * dt;
        let need_plate = 0.2 * dt;
        let ok = {
            let n = self.nodes.get(&id);
            n.map(|n| n.stock(frame) >= need_frame && n.stock(plate) >= need_plate)
                .unwrap_or(false)
        };
        if !ok {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.working = false;
            }
            return;
        }
        if let Some(e) = self.network_energy.get_mut(&root) {
            *e -= cost;
        }
        if let Some(n) = self.nodes.get_mut(&id) {
            let _ = n.try_take_stock(frame, need_frame);
            let _ = n.try_take_stock(plate, need_plate);
            n.working = true;
        }
        self.tech.nexus_progress = (self.tech.nexus_progress + dt * 0.002).min(1.0);
        if self.tech.nexus_progress >= 1.0 && !self.tech.nexus_complete {
            // Promote site → Nexus landmark.
            if let Some(n) = self.nodes.get_mut(&id) {
                let (x, y) = (n.x, n.y);
                *n = Node::new(BuildingKind::Nexus, x, y, Facing::E);
            }
            self.tech.nexus_complete = true;
            self.tech.era2_unlocked = true;
            self.tech.researched.insert("era1_tech_era_transition".into());
            self.era1_complete = true;
        }
    }

    /// Data-driven Era crafter. Returns true if it handled the node this tick.
    fn step_era_crafter(&mut self, id: u32, pay: Option<u32>, dt: f32) -> bool {
        let Some(root) = pay else {
            return false;
        };
        let (machine_key, era_active, craft_recipe, craft_t) = {
            let Some(n) = self.nodes.get(&id) else {
                return false;
            };
            let key = n
                .machine_id
                .and_then(|i| content().machine(i).map(|m| m.id.clone()))
                .unwrap_or_default();
            if key.is_empty() {
                return false;
            }
            (key, n.era_craft, n.craft_recipe, n.craft_t)
        };

        // Continue active Era craft.
        if era_active && craft_recipe != 0 {
            let Some(recipe) = content().recipe(craft_recipe) else {
                if let Some(n) = self.nodes.get_mut(&id) {
                    n.era_craft = false;
                    n.craft_recipe = 0;
                    n.craft_t = 0.0;
                }
                return true;
            };
            if !self.tech.recipe_unlocked(&recipe.technology_unlock) {
                if let Some(n) = self.nodes.get_mut(&id) {
                    n.working = false;
                }
                return true;
            }
            // Min purity gate for sensitive recipes (laser optics etc.).
            if recipe.purity_effect.abs() > 0.0 || recipe.id.contains("optical") || recipe.id.contains("laser_lens") {
                for io in &recipe.inputs {
                    let item = Item::from_u16(io.item);
                    if let Some(def) = content().item(io.item) {
                        if def.purity_supported {
                            let p = self.nodes.get(&id).map(|n| n.purity(item)).unwrap_or(0.0);
                            let min_p = if recipe.id.contains("laser") || recipe.id.contains("optical") {
                                70.0
                            } else {
                                0.0
                            };
                            if p + 1e-3 < min_p {
                                if let Some(n) = self.nodes.get_mut(&id) {
                                    n.working = false;
                                }
                                return true;
                            }
                        }
                    }
                }
            }
            let cost = recipe.power_kw * dt;
            let has = self.network_energy.get(&root).copied().unwrap_or(0.0);
            if has < cost {
                if let Some(n) = self.nodes.get_mut(&id) {
                    n.working = false;
                }
                return true;
            }
            if let Some(e) = self.network_energy.get_mut(&root) {
                *e -= cost;
            }
            let done = craft_t + dt;
            if done >= recipe.processing_time {
                if era_outputs_fit(self.nodes.get(&id), recipe) {
                    let in_purity = {
                        let n = self.nodes.get(&id).unwrap();
                        recipe
                            .inputs
                            .first()
                            .map(|io| n.purity(Item::from_u16(io.item)))
                            .unwrap_or(50.0)
                    };
                    let out_purity = (in_purity + recipe.purity_effect).clamp(0.0, 100.0);
                    if let Some(n) = self.nodes.get_mut(&id) {
                        for io in recipe.all_outputs() {
                            n.add_stock_purity(Item::from_u16(io.item), io.amount, out_purity);
                        }
                        n.craft_recipe = 0;
                        n.craft_t = 0.0;
                        n.era_craft = false;
                        n.working = true;
                    }
                } else if let Some(n) = self.nodes.get_mut(&id) {
                    n.craft_t = recipe.processing_time;
                    n.working = false;
                }
            } else if let Some(n) = self.nodes.get_mut(&id) {
                n.craft_t = done;
                n.working = true;
            }
            return true;
        }

        // Start a new Era recipe for this machine.
        let candidates: Vec<u16> = {
            let mut list = content().recipes_for_machine(&machine_key).to_vec();
            if list.is_empty() {
                if let Some(m) = content().machine_by_str(&machine_key) {
                    list = content().recipes_for_categories(&m.recipe_categories);
                }
            }
            list
        };
        for rid in candidates {
            let Some(recipe) = content().recipe(rid) else {
                continue;
            };
            if !self.tech.recipe_unlocked(&recipe.technology_unlock) {
                continue;
            }
            let Some(n) = self.nodes.get(&id) else {
                return false;
            };
            let mut ok = true;
            for io in &recipe.inputs {
                if n.stock(Item::from_u16(io.item)) + 1e-4 < io.amount {
                    ok = false;
                    break;
                }
            }
            if !ok || !era_outputs_fit(Some(n), recipe) {
                continue;
            }
            let cost = recipe.power_kw * dt;
            let has = self.network_energy.get(&root).copied().unwrap_or(0.0);
            if has < cost {
                if let Some(n) = self.nodes.get_mut(&id) {
                    n.working = false;
                }
                return true;
            }
            if let Some(e) = self.network_energy.get_mut(&root) {
                *e -= cost;
            }
            if let Some(n) = self.nodes.get_mut(&id) {
                for io in &recipe.inputs {
                    let _ = n.try_take_stock(Item::from_u16(io.item), io.amount);
                }
                n.craft_recipe = rid;
                n.craft_t = dt.min(recipe.processing_time);
                n.era_craft = true;
                n.working = true;
            }
            return true;
        }
        if let Some(n) = self.nodes.get_mut(&id) {
            // No era recipe started — allow legacy fallback for smelter/assembler.
            if matches!(n.kind, BuildingKind::Smelter | BuildingKind::Assembler) {
                return false;
            }
            n.working = false;
        }
        true
    }

    fn step_crafter(&mut self, id: u32, pay: Option<u32>, dt: f32, machine: MachineKind) {
        let Some(root) = pay else {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.working = false;
            }
            return;
        };

        // Finish or continue an in-flight craft.
        let active = self.nodes.get(&id).map(|n| (n.craft_recipe, n.craft_t));
        if let Some((rid, t)) = active {
            if rid != 0 {
                let Some(recipe) = recipes::recipe_by_id(rid) else {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.craft_recipe = 0;
                        n.craft_t = 0.0;
                        n.working = false;
                    }
                    return;
                };
                let cost = recipe.power.max(1.0) * dt;
                let has = self.network_energy.get(&root).copied().unwrap_or(0.0);
                if has < cost {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.working = false;
                    }
                    return;
                }
                if let Some(e) = self.network_energy.get_mut(&root) {
                    *e -= cost;
                }
                let done = t + dt;
                if done >= recipe.craft_time {
                    if outputs_fit(self.nodes.get(&id), recipe) {
                        if let Some(n) = self.nodes.get_mut(&id) {
                            for &(item, qty) in recipe.outputs {
                                n.add_stock(item, qty as f32);
                            }
                            n.craft_recipe = 0;
                            n.craft_t = 0.0;
                            n.working = true;
                        }
                    } else if let Some(n) = self.nodes.get_mut(&id) {
                        // Blocked on output space — hold craft complete until free.
                        n.craft_t = recipe.craft_time;
                        n.working = false;
                    }
                } else if let Some(n) = self.nodes.get_mut(&id) {
                    n.craft_t = done;
                    n.working = true;
                }
                return;
            }
        }

        // Try to start a recipe that fits current stocks.
        let started = {
            let Some(n) = self.nodes.get(&id) else {
                return;
            };
            pick_startable_recipe(n, machine)
        };
        let Some(recipe) = started else {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.working = false;
            }
            return;
        };
        let cost = recipe.power.max(1.0) * dt;
        let has = self.network_energy.get(&root).copied().unwrap_or(0.0);
        if has < cost {
            if let Some(n) = self.nodes.get_mut(&id) {
                n.working = false;
            }
            return;
        }
        if let Some(e) = self.network_energy.get_mut(&root) {
            *e -= cost;
        }
        if let Some(n) = self.nodes.get_mut(&id) {
            for &(item, qty) in recipe.inputs {
                let _ = n.try_take_stock(item, qty as f32);
            }
            n.craft_recipe = recipe.id;
            n.craft_t = dt.min(recipe.craft_time);
            n.working = true;
        }
    }

}

fn outputs_fit(n: Option<&Node>, recipe: &Recipe) -> bool {
    let Some(n) = n else {
        return false;
    };
    for &(item, qty) in recipe.outputs {
        if n.stock(item) + qty as f32 > NODE_BUFFER + 1e-3 {
            return false;
        }
    }
    true
}

fn era_outputs_fit(n: Option<&Node>, recipe: &content::RuntimeRecipe) -> bool {
    let Some(n) = n else {
        return false;
    };
    for io in recipe.all_outputs() {
        if n.stock(Item::from_u16(io.item)) + io.amount > NODE_BUFFER + 1e-3 {
            return false;
        }
    }
    true
}

fn pick_startable_recipe(n: &Node, machine: MachineKind) -> Option<&'static Recipe> {
    for recipe in recipes::recipes_for(machine) {
        let mut ok = true;
        for &(item, qty) in recipe.inputs {
            if n.stock(item) + 1e-4 < qty as f32 {
                ok = false;
                break;
            }
        }
        if !ok || !outputs_fit(Some(n), recipe) {
            continue;
        }
        return Some(recipe);
    }
    None
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

