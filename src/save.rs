//! Settings + single-player save/load (JSON under `userdata/`).

use crate::belts::{BeltItem, BeltTile};
use crate::sim::{BuildingKind, Facing, Item, Link, Nest, Node, Raider, World};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const AUTOSAVE_SLOTS: usize = 3;
pub const AUTOSAVE_INTERVAL_SECS: f32 = 300.0;
pub const SAVE_VERSION: u32 = 10;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DisplayMode {
    Windowed,
    Borderless,
    Fullscreen,
}

impl DisplayMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Windowed => "Windowed",
            Self::Borderless => "Borderless",
            Self::Fullscreen => "Fullscreen",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Windowed => Self::Borderless,
            Self::Borderless => Self::Fullscreen,
            Self::Fullscreen => Self::Windowed,
        }
    }

    pub fn is_windowed(self) -> bool {
        matches!(self, Self::Windowed)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EffectQuality {
    Low,
    #[default]
    Medium,
    High,
}

impl EffectQuality {
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Low,
        }
    }
}

/// Render FPS cap. Simulation stays at fixed 60 UPS regardless.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FpsLimit {
    Fps30,
    Fps60,
    #[default]
    Fps120,
    Fps144,
    Fps240,
    Unlimited,
}

impl FpsLimit {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fps30 => "30",
            Self::Fps60 => "60",
            Self::Fps120 => "120",
            Self::Fps144 => "144",
            Self::Fps240 => "240",
            Self::Unlimited => "Unlimited",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Fps30 => Self::Fps60,
            Self::Fps60 => Self::Fps120,
            Self::Fps120 => Self::Fps144,
            Self::Fps144 => Self::Fps240,
            Self::Fps240 => Self::Unlimited,
            Self::Unlimited => Self::Fps30,
        }
    }

    /// `None` = no software sleep (run as fast as the GPU/CPU allow).
    pub fn frame_budget(self) -> Option<std::time::Duration> {
        let fps = match self {
            Self::Fps30 => 30.0,
            Self::Fps60 => 60.0,
            Self::Fps120 => 120.0,
            Self::Fps144 => 144.0,
            Self::Fps240 => 240.0,
            Self::Unlimited => return None,
        };
        Some(std::time::Duration::from_secs_f64(1.0 / fps))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub display_mode: DisplayMode,
    pub vsync: bool,
    pub window_w: i32,
    pub window_h: i32,
    pub show_fps: bool,
    /// Storm / gas / lightning fidelity.
    #[serde(default)]
    pub effect_quality: EffectQuality,
    /// Software render FPS cap (player speed uses fixed UPS, not this).
    #[serde(default)]
    pub fps_limit: FpsLimit,
    /// Next autosave slot index (0..AUTOSAVE_SLOTS).
    pub autosave_next: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            display_mode: DisplayMode::Windowed,
            vsync: false,
            window_w: 1400,
            window_h: 900,
            show_fps: true,
            effect_quality: EffectQuality::Medium,
            fps_limit: FpsLimit::Fps120,
            autosave_next: 0,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = settings_path();
        match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        ensure_dirs()?;
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(settings_path(), text).map_err(|e| e.to_string())
    }

    pub fn apply_runtime(&self) {
        macroquad::prelude::set_fullscreen(!self.display_mode.is_windowed());
        if self.display_mode.is_windowed() {
            macroquad::prelude::request_new_screen_size(
                self.window_w as f32,
                self.window_h as f32,
            );
        }
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn userdata_dir() -> PathBuf {
    PathBuf::from("userdata")
}

fn saves_dir() -> PathBuf {
    userdata_dir().join("saves")
}

fn settings_path() -> PathBuf {
    userdata_dir().join("settings.json")
}

fn ensure_dirs() -> Result<(), String> {
    fs::create_dir_all(saves_dir()).map_err(|e| e.to_string())?;
    Ok(())
}

/// New-game / save mode — Creative skips costs and unlock gates.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    #[default]
    Survival,
    Creative,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Survival => "Survival",
            Self::Creative => "Creative",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Self::Survival => "Tech gates build menu · spend materials to place",
            Self::Creative => "Everything unlocked · free placement",
        }
    }

    pub fn is_creative(self) -> bool {
        matches!(self, Self::Creative)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameSave {
    pub version: u32,
    pub saved_at: u64,
    pub label: String,
    pub cam_x: f32,
    pub cam_y: f32,
    pub cam_zoom: f32,
    pub hotbar: [Option<u8>; 9],
    pub hotbar_index: usize,
    pub next_id: u32,
    pub nodes: Vec<NodeSave>,
    pub links: Vec<LinkSave>,
    /// Factorio-style belt tile grid (v2+).
    #[serde(default)]
    pub belt_tiles: Vec<BeltTileSave>,
    #[serde(default)]
    pub nests: Vec<NestSave>,
    #[serde(default)]
    pub raiders: Vec<RaiderSave>,
    #[serde(default)]
    pub next_nest_id: u32,
    #[serde(default)]
    pub next_raider_id: u32,
    /// Player inventory (v3+). Missing on older saves → starter kit applied on load.
    #[serde(default)]
    pub inv_ore: Option<u32>,
    #[serde(default)]
    pub inv_ingot: Option<u32>,
    /// Legacy circular deposits (v4).
    #[serde(default)]
    pub deposits: Vec<DepositSave>,
    /// Legacy Factorio-style ore tiles (v5).
    #[serde(default)]
    pub ore_tiles: Vec<OreTileSave>,
    /// Living resource veins (v6+).
    #[serde(default)]
    pub veins: Vec<VeinSave>,
    /// Survival vs Creative (v9+). Older saves default to Survival.
    #[serde(default)]
    pub game_mode: GameMode,
    /// Accumulated unpaused playtime in seconds (v10+).
    #[serde(default)]
    pub play_seconds: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NestSave {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub active: bool,
    #[serde(default)]
    pub wave_cd: f32,
    #[serde(default)]
    pub evolution: f32,
    #[serde(default)]
    pub anger: f32,
    #[serde(default = "default_true")]
    pub first_wave: bool,
    #[serde(default)]
    pub dormant_hate: bool,
    #[serde(default)]
    pub spawn_cd: f32,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaiderSave {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub target_node: Option<u32>,
    pub attack_cd: f32,
    #[serde(default)]
    pub wave_id: u32,
    #[serde(default)]
    pub vx: f32,
    #[serde(default)]
    pub vy: f32,
    #[serde(default)]
    pub role: u8,
    #[serde(default)]
    pub retarget_cd: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeSave {
    pub id: u32,
    pub kind: u8,
    pub x: f32,
    pub y: f32,
    pub facing: u8,
    pub in_ore: f32,
    pub out_ore: f32,
    pub out_ingot: f32,
    pub store_ore: f32,
    pub store_ingot: f32,
    pub buf_ore: f32,
    pub buf_ingot: f32,
    #[serde(default)]
    pub split_ore: [u16; 2],
    #[serde(default)]
    pub split_ingot: [u16; 2],
    #[serde(default)]
    pub split_side: [u8; 2],
    #[serde(default)]
    pub hp: Option<f32>,
    #[serde(default)]
    pub cable_a: Option<(u32, usize)>,
    #[serde(default)]
    pub cable_b: Option<(u32, usize)>,
    #[serde(default)]
    pub mine_item: Option<u8>,
    #[serde(default)]
    pub mine_vein: Option<u32>,
    #[serde(default)]
    pub store_copper: f32,
    #[serde(default)]
    pub store_stone: f32,
    #[serde(default)]
    pub store_coal: f32,
    #[serde(default)]
    pub store_oil: f32,
    #[serde(default)]
    pub stocks: Vec<f32>,
    #[serde(default)]
    pub craft_recipe: u16,
    #[serde(default)]
    pub craft_t: f32,
    #[serde(default)]
    pub machine_id: Option<u16>,
    #[serde(default)]
    pub era_craft: bool,
    #[serde(default)]
    pub ammo: f32,
    #[serde(default)]
    pub stock_purity: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepositSave {
    pub kind: u8,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub amount: f32,
    pub amount_max: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OreTileSave {
    pub tx: i32,
    pub ty: i32,
    pub kind: u8,
    pub amount: f32,
    pub amount_max: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VeinSave {
    pub id: u32,
    pub kind: u8,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    /// v7+ Factorio-oil-style yield percent.
    #[serde(default)]
    pub yield_pct: f32,
    #[serde(default)]
    pub yield_max: f32,
    /// Legacy v6 fields (migrated on load).
    #[serde(default)]
    pub reserve: f32,
    #[serde(default)]
    pub reserve_max: f32,
    #[serde(default)]
    pub potency: f32,
    pub seed: u32,
    #[serde(default)]
    pub purity: f32,
    #[serde(default)]
    pub stability: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinkSave {
    pub from_node: u32,
    pub from_port: usize,
    pub to_node: u32,
    pub to_port: usize,
    #[serde(default)]
    pub cable_id: u32,
    /// Freehand route points [x,y,x,y,...]. Empty = Manhattan.
    #[serde(default)]
    pub path: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeltTileSave {
    pub tx: i32,
    pub ty: i32,
    pub dir: u8,
    pub lane0: Vec<BeltItemSave>,
    pub lane1: Vec<BeltItemSave>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeltItemSave {
    /// Legacy u8 item id; prefer `item16` when present.
    pub item: u8,
    /// v1: distance along link. v2: progress 0..1 on tile.
    pub dist: f32,
    #[serde(default)]
    pub item16: Option<u16>,
    #[serde(default)]
    pub purity: f32,
}

#[derive(Clone, Debug)]
pub struct SaveInfo {
    pub path: PathBuf,
    pub label: String,
    pub saved_at: u64,
    pub play_seconds: f32,
    pub game_mode: GameMode,
    pub buildings: usize,
    pub belts: usize,
    pub version: u32,
    pub file_bytes: u64,
    pub preview_path: Option<PathBuf>,
}

fn item_to_u8(item: Item) -> u8 {
    item.as_u8()
}

fn item_from_save(it: &BeltItemSave) -> Item {
    if let Some(v) = it.item16 {
        Item::from_u16(v)
    } else {
        Item::from_u8(it.item)
    }
}

pub fn capture_save(
    world: &World,
    cam_x: f32,
    cam_y: f32,
    cam_zoom: f32,
    hotbar: &[Option<BuildingKind>; 9],
    hotbar_index: usize,
    inv_ore: u32,
    inv_ingot: u32,
    label: &str,
    game_mode: GameMode,
    play_seconds: f32,
) -> GameSave {
    let nodes = world
        .nodes
        .iter()
        .filter(|(_, n)| n.kind != BuildingKind::Conveyor)
        .map(|(&id, n)| NodeSave {
            id,
            kind: n.kind.as_u8(),
            x: n.x,
            y: n.y,
            facing: n.facing.as_u8(),
            in_ore: n.in_ore,
            out_ore: n.out_ore,
            out_ingot: n.out_ingot,
            store_ore: n.store_ore,
            store_ingot: n.store_ingot,
            buf_ore: n.buf_ore,
            buf_ingot: n.buf_ingot,
            split_ore: n.split_ore,
            split_ingot: n.split_ingot,
            split_side: n.split_side,
            hp: Some(n.hp),
            cable_a: n.cable_a,
            cable_b: n.cable_b,
            mine_item: n.mine_item.map(|i| i.as_u8()),
            mine_vein: n.mine_vein,
            store_copper: n.store_copper,
            store_stone: n.store_stone,
            store_coal: n.store_coal,
            store_oil: n.store_oil,
            stocks: n.stocks.to_vec(),
            craft_recipe: n.craft_recipe,
            craft_t: n.craft_t,
            machine_id: n.machine_id,
            era_craft: n.era_craft,
            ammo: n.ammo,
            stock_purity: n.stock_purity.clone(),
        })
        .collect();
    let links = world
        .links
        .iter()
        .map(|l| LinkSave {
            from_node: l.from_node,
            from_port: l.from_port,
            to_node: l.to_node,
            to_port: l.to_port,
            cable_id: l.cable_id,
            path: l
                .path
                .iter()
                .flat_map(|(x, y)| [*x, *y])
                .collect(),
        })
        .collect();
    let mut belt_tiles: Vec<BeltTileSave> = world
        .belt_tiles
        .iter()
        .map(|(&(tx, ty), t)| BeltTileSave {
            tx,
            ty,
            dir: t.dir.as_u8(),
            lane0: t.lanes[0]
                .items
                .iter()
                .map(|it| BeltItemSave {
                    item: item_to_u8(it.item),
                    dist: it.progress,
                    item16: Some(it.item.as_u16()),
                    purity: it.purity,
                })
                .collect(),
            lane1: t.lanes[1]
                .items
                .iter()
                .map(|it| BeltItemSave {
                    item: item_to_u8(it.item),
                    dist: it.progress,
                    item16: Some(it.item.as_u16()),
                    purity: it.purity,
                })
                .collect(),
        })
        .collect();
    belt_tiles.sort_by_key(|b| (b.ty, b.tx));
    let mut hb = [None; 9];
    for (i, k) in hotbar.iter().enumerate() {
        hb[i] = k.map(|k| k.as_u8());
    }
    GameSave {
        version: SAVE_VERSION,
        saved_at: now_unix(),
        label: label.into(),
        cam_x,
        cam_y,
        cam_zoom,
        hotbar: hb,
        hotbar_index,
        next_id: world.next_id,
        nodes,
        links,
        belt_tiles,
        nests: world
            .nests
            .iter()
            .map(|n| NestSave {
                id: n.id,
                x: n.x,
                y: n.y,
                hp: n.hp,
                active: n.active,
                wave_cd: n.wave_cd,
                evolution: n.evolution,
                anger: n.anger,
                first_wave: n.first_wave,
                dormant_hate: n.dormant_hate,
                spawn_cd: 0.0,
            })
            .collect(),
        raiders: world
            .raiders
            .iter()
            .map(|r| RaiderSave {
                id: r.id,
                x: r.x,
                y: r.y,
                hp: r.hp,
                target_node: r.target_node,
                attack_cd: r.attack_cd,
                wave_id: r.wave_id,
                vx: r.vx,
                vy: r.vy,
                role: r.role.as_u8(),
                retarget_cd: r.retarget_cd,
            })
            .collect(),
        next_nest_id: world.next_nest_id,
        next_raider_id: world.next_raider_id,
        inv_ore: Some(inv_ore),
        inv_ingot: Some(inv_ingot),
        deposits: Vec::new(),
        ore_tiles: Vec::new(),
        veins: world
            .veins
            .iter()
            .map(|v| VeinSave {
                id: v.id,
                kind: v.kind.as_u8(),
                x: v.x,
                y: v.y,
                radius: v.radius,
                yield_pct: v.yield_pct,
                yield_max: v.yield_max,
                reserve: 0.0,
                reserve_max: 0.0,
                potency: 0.0,
                seed: v.seed,
                purity: v.purity,
                stability: v.stability,
            })
            .collect(),
        game_mode,
        play_seconds,
    }
}

pub fn apply_save(world: &mut World, save: &GameSave) -> Result<(), String> {
    if save.version != 1
        && save.version != 2
        && save.version != 3
        && save.version != 4
        && save.version != 5
        && save.version != 6
        && save.version != 8
        && save.version != 9
        && save.version != SAVE_VERSION
    {
        return Err(format!("Unsupported save version {}", save.version));
    }
    world.clear();
    world.next_id = save.next_id.max(1);
    for n in &save.nodes {
        let kind = BuildingKind::from_u8(n.kind).ok_or_else(|| format!("bad kind {}", n.kind))?;
        if kind == BuildingKind::Conveyor {
            continue; // legacy cable conveyors discarded
        }
        let facing = Facing::from_u8(n.facing);
        let mut node = Node::new(kind, n.x, n.y, facing);
        node.in_ore = n.in_ore;
        node.out_ore = n.out_ore;
        node.out_ingot = n.out_ingot;
        node.store_ore = n.store_ore;
        node.store_ingot = n.store_ingot;
        node.buf_ore = n.buf_ore;
        node.buf_ingot = n.buf_ingot;
        node.split_ore = n.split_ore;
        node.split_ingot = n.split_ingot;
        node.split_side = [n.split_side[0].min(1), n.split_side[1].min(1)];
        // Legacy saves only had totals — park backlog on left lane.
        if kind == BuildingKind::Splitter {
            let ore_sum = node.split_ore[0] as f32 + node.split_ore[1] as f32;
            let ing_sum = node.split_ingot[0] as f32 + node.split_ingot[1] as f32;
            if ore_sum < 0.5 && node.buf_ore >= 1.0 {
                node.split_ore[0] = node.buf_ore.floor() as u16;
            }
            if ing_sum < 0.5 && node.buf_ingot >= 1.0 {
                node.split_ingot[0] = node.buf_ingot.floor() as u16;
            }
            node.buf_ore = (node.split_ore[0] + node.split_ore[1]) as f32;
            node.buf_ingot = (node.split_ingot[0] + node.split_ingot[1]) as f32;
        }
        if let Some(hp) = n.hp {
            node.hp = hp.clamp(0.0, node.max_hp);
        }
        node.cable_a = n.cable_a;
        node.cable_b = n.cable_b;
        node.store_copper = n.store_copper;
        node.store_stone = n.store_stone;
        node.store_coal = n.store_coal;
        node.store_oil = n.store_oil;
        node.ensure_stock_len();
        if !n.stocks.is_empty() {
            for (i, v) in n.stocks.iter().enumerate() {
                if i < node.stocks.len() {
                    node.stocks[i] = *v;
                }
            }
            node.sync_legacy_from_stocks();
        } else {
            node.sync_stocks_from_legacy();
        }
        if !n.stock_purity.is_empty() {
            for (i, v) in n.stock_purity.iter().enumerate() {
                if i < node.stock_purity.len() {
                    node.stock_purity[i] = *v;
                }
            }
        }
        node.craft_recipe = n.craft_recipe;
        node.craft_t = n.craft_t;
        node.machine_id = n.machine_id.or(node.machine_id);
        node.era_craft = n.era_craft;
        node.ammo = n.ammo;
        if let Some(mi) = n.mine_item {
            node.mine_item = Some(Item::from_u8(mi));
            node.rebuild_ports();
        }
        node.mine_vein = n.mine_vein;
        world.nodes.insert(n.id, node);
    }
    for l in &save.links {
        world.links.push(Link {
            from_node: l.from_node,
            from_port: l.from_port,
            to_node: l.to_node,
            to_port: l.to_port,
            cable_id: l.cable_id,
            path: l
                .path
                .chunks_exact(2)
                .map(|c| (c[0], c[1]))
                .collect(),
        });
    }
    for b in &save.belt_tiles {
        let mut tile = BeltTile::new(Facing::from_u8(b.dir));
        tile.lanes[0].items = b
            .lane0
            .iter()
            .map(|it| BeltItem::with_purity(
                item_from_save(it),
                it.dist.clamp(0.0, 1.0),
                if it.purity > 0.0 { it.purity } else { 50.0 },
            ))
            .collect();
        tile.lanes[1].items = b
            .lane1
            .iter()
            .map(|it| BeltItem::with_purity(
                item_from_save(it),
                it.dist.clamp(0.0, 1.0),
                if it.purity > 0.0 { it.purity } else { 50.0 },
            ))
            .collect();
        world.belt_tiles.insert((b.tx, b.ty), tile);
    }
    world.nests = save
        .nests
        .iter()
        .map(|n| Nest {
            id: n.id,
            x: n.x,
            y: n.y,
            hp: n.hp,
            max_hp: crate::sim::NEST_HP,
            active: n.active,
            wave_cd: if n.wave_cd > 0.0 {
                n.wave_cd
            } else {
                n.spawn_cd.max(4.0)
            },
            evolution: n.evolution,
            anger: n.anger,
            first_wave: n.first_wave,
            dormant_hate: n.dormant_hate,
        })
        .collect();
    world.raiders = save
        .raiders
        .iter()
        .map(|r| {
            use crate::sim::RaiderRole;
            Raider {
                id: r.id,
                x: r.x,
                y: r.y,
                hp: r.hp,
                target_node: r.target_node,
                attack_cd: r.attack_cd,
                wave_id: r.wave_id,
                vx: r.vx,
                vy: r.vy,
                role: RaiderRole::from_u8(r.role),
                retarget_cd: r.retarget_cd,
            }
        })
        .collect();
    world.next_nest_id = save.next_nest_id.max(1);
    world.next_raider_id = save.next_raider_id.max(1);
    world.veins.clear();
    if !save.veins.is_empty() {
        for v in &save.veins {
            let Some(kind) = crate::deposits::ResourceKind::from_u8(v.kind) else {
                continue;
            };
            let (yield_pct, yield_max) = if v.yield_max > 1.0 {
                (v.yield_pct.max(20.0), v.yield_max)
            } else if v.potency > 0.0 {
                let ym = (v.potency * 140.0).clamp(80.0, 400.0);
                (ym, ym)
            } else if v.reserve_max > 1.0 {
                let ym = (80.0 + v.reserve_max / 200.0).clamp(80.0, 1800.0);
                let yp = ym * (v.reserve / v.reserve_max).clamp(0.2, 1.0);
                (yp, ym)
            } else {
                (120.0, 120.0)
            };
            // Old saves used ~280–760 field radii; scale those up to the gas-vent footprint.
            let mut radius = v.radius.max(40.0);
            if radius < 800.0 {
                radius = (radius * 2.85).clamp(900.0, 2200.0);
            }
            world.veins.push(crate::deposits::Vein {
                id: v.id,
                kind,
                x: v.x,
                y: v.y,
                radius,
                yield_pct,
                yield_max: yield_max.max(yield_pct).max(20.0),
                seed: v.seed,
                clear_factor: 1.0,
                taps: 0,
                purity: if v.purity > 0.0 {
                    v.purity
                } else {
                    kind.base_purity()
                },
                stability: if v.stability > 0.0 { v.stability } else { 0.85 },
            });
        }
    } else if !save.deposits.is_empty() {
        let mut id = 1u32;
        for d in &save.deposits {
            let Some(kind) = crate::deposits::ResourceKind::from_u8(d.kind) else {
                continue;
            };
            world.veins.push(crate::deposits::vein_from_legacy(
                id,
                kind,
                d.x,
                d.y,
                d.radius,
                d.amount.max(1.0),
            ));
            id += 1;
        }
    } else if !save.ore_tiles.is_empty() {
        // Collapse v5 tile carpets into approximate veins by kind centroid.
        use std::collections::HashMap;
        let mut acc: HashMap<u8, (f32, f32, f32, f32, u32)> = HashMap::new();
        for t in &save.ore_tiles {
            if t.amount <= 1.0 {
                continue;
            }
            let wx = t.tx as f32 * 40.0 + 20.0;
            let wy = t.ty as f32 * 40.0 + 20.0;
            let e = acc.entry(t.kind).or_insert((0.0, 0.0, 0.0, 0.0, 0));
            e.0 += wx * t.amount;
            e.1 += wy * t.amount;
            e.2 += t.amount;
            e.3 += t.amount;
            e.4 += 1;
        }
        let mut id = 1u32;
        for (kind_u8, (sx, sy, wsum, amount, n)) in acc {
            let Some(kind) = crate::deposits::ResourceKind::from_u8(kind_u8) else {
                continue;
            };
            if wsum <= 1.0 {
                continue;
            }
            let radius = (80.0 + (n as f32).sqrt() * 28.0).clamp(100.0, 320.0);
            world.veins.push(crate::deposits::vein_from_legacy(
                id,
                kind,
                sx / wsum,
                sy / wsum,
                radius,
                amount,
            ));
            id += 1;
        }
    }
    world.next_vein_id = world
        .veins
        .iter()
        .map(|v| v.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
        .max(1);
    // Rebind drills that lost vein linkage.
    let miner_ids: Vec<u32> = world
        .nodes
        .iter()
        .filter(|(_, n)| n.kind == BuildingKind::OreNode)
        .map(|(&id, _)| id)
        .collect();
    for id in miner_ids {
        let needs = world
            .nodes
            .get(&id)
            .map(|n| n.mine_vein.is_none() || n.mine_item.is_none())
            .unwrap_or(false);
        if needs {
            let _ = world.bind_miner(id);
        }
    }
    world.ensure_cable_entities();
    Ok(())
}

pub fn write_save(path: &Path, save: &GameSave) -> Result<(), String> {
    ensure_dirs()?;
    let text = serde_json::to_string_pretty(save).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

pub fn read_save(path: &Path) -> Result<GameSave, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn manual_save_path() -> PathBuf {
    saves_dir().join("manual.json")
}

pub fn autosave_path(slot: usize) -> PathBuf {
    saves_dir().join(format!("autosave_{slot}.json"))
}

pub fn write_manual_save(save: &GameSave) -> Result<PathBuf, String> {
    let path = manual_save_path();
    write_save(&path, save)?;
    Ok(path)
}

pub fn write_autosave(settings: &mut Settings, save: &mut GameSave) -> Result<PathBuf, String> {
    let slot = settings.autosave_next % AUTOSAVE_SLOTS;
    save.label = format!("Autosave {}", slot + 1);
    let path = autosave_path(slot);
    write_save(&path, save)?;
    settings.autosave_next = (slot + 1) % AUTOSAVE_SLOTS;
    settings.save()?;
    Ok(path)
}

/// Sidecar preview next to a save JSON: `foo.json` → `foo_preview.png`.
pub fn preview_path_for(save_json: &Path) -> PathBuf {
    let stem = save_json
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("save");
    save_json
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{stem}_preview.png"))
}

pub fn delete_save(info: &SaveInfo) -> Result<(), String> {
    if let Some(prev) = &info.preview_path {
        let _ = fs::remove_file(prev);
    } else {
        let _ = fs::remove_file(preview_path_for(&info.path));
    }
    fs::remove_file(&info.path).map_err(|e| e.to_string())
}

pub fn list_saves() -> Vec<SaveInfo> {
    let Ok(entries) = fs::read_dir(saves_dir()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        // Skip accidental non-save json
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.ends_with("_preview.png") {
            continue;
        }
        if let Ok(save) = read_save(&path) {
            let meta = fs::metadata(&path).ok();
            let preview = preview_path_for(&path);
            out.push(SaveInfo {
                path,
                label: save.label,
                saved_at: save.saved_at,
                play_seconds: save.play_seconds,
                game_mode: save.game_mode,
                buildings: save.nodes.len(),
                belts: save.belt_tiles.len(),
                version: save.version,
                file_bytes: meta.map(|m| m.len()).unwrap_or(0),
                preview_path: preview.exists().then_some(preview),
            });
        }
    }
    out.sort_by(|a, b| b.saved_at.cmp(&a.saved_at));
    out
}

pub fn most_recent_save() -> Option<SaveInfo> {
    list_saves().into_iter().next()
}

pub fn format_saved_at(ts: u64) -> String {
    let now = now_unix();
    let ago = now.saturating_sub(ts);
    if ago < 60 {
        "just now".into()
    } else if ago < 3600 {
        format!("{}m ago", ago / 60)
    } else if ago < 86400 {
        format!("{}h ago", ago / 3600)
    } else {
        format!("{}d ago", ago / 86400)
    }
}

pub fn format_playtime(secs: f32) -> String {
    let total = secs.max(0.0) as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
