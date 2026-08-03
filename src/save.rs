//! Settings + single-player save/load (JSON under `userdata/`).

use crate::belts::{BeltItem, BeltTile};
use crate::sim::{BuildingKind, Facing, Item, Link, Nest, Node, Raider, World};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const AUTOSAVE_SLOTS: usize = 3;
pub const AUTOSAVE_INTERVAL_SECS: f32 = 300.0;
pub const SAVE_VERSION: u32 = 2;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub display_mode: DisplayMode,
    pub vsync: bool,
    pub window_w: i32,
    pub window_h: i32,
    pub show_fps: bool,
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
            show_fps: false,
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
    /// Legacy port-to-port belts (v1). Ignored on load for v2+.
    #[serde(default)]
    pub belts: Vec<LegacyBeltSave>,
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinkSave {
    pub from_node: u32,
    pub from_port: usize,
    pub to_node: u32,
    pub to_port: usize,
    #[serde(default)]
    pub cable_id: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegacyBeltSave {
    pub from_node: u32,
    pub from_port: usize,
    pub to_node: u32,
    pub to_port: usize,
    #[serde(default)]
    pub cable_id: u32,
    #[serde(default)]
    pub lane0: Vec<BeltItemSave>,
    #[serde(default)]
    pub lane1: Vec<BeltItemSave>,
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
    pub item: u8,
    /// v1: distance along link. v2: progress 0..1 on tile.
    pub dist: f32,
}

#[derive(Clone, Debug)]
pub struct SaveInfo {
    pub path: PathBuf,
    pub label: String,
    pub saved_at: u64,
}

fn item_to_u8(item: Item) -> u8 {
    match item {
        Item::IronOre => 0,
        Item::IronIngot => 1,
    }
}

fn item_from_u8(v: u8) -> Item {
    if v == 1 {
        Item::IronIngot
    } else {
        Item::IronOre
    }
}

pub fn capture_save(
    world: &World,
    cam_x: f32,
    cam_y: f32,
    cam_zoom: f32,
    hotbar: &[Option<BuildingKind>; 9],
    hotbar_index: usize,
    label: &str,
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
                })
                .collect(),
            lane1: t.lanes[1]
                .items
                .iter()
                .map(|it| BeltItemSave {
                    item: item_to_u8(it.item),
                    dist: it.progress,
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
        belts: Vec::new(),
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
    }
}

pub fn apply_save(world: &mut World, save: &GameSave) -> Result<(), String> {
    if save.version != 1 && save.version != SAVE_VERSION {
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
        world.nodes.insert(n.id, node);
    }
    for l in &save.links {
        world.links.push(Link {
            from_node: l.from_node,
            from_port: l.from_port,
            to_node: l.to_node,
            to_port: l.to_port,
            cable_id: l.cable_id,
        });
    }
    for b in &save.belt_tiles {
        let mut tile = BeltTile::new(Facing::from_u8(b.dir));
        tile.lanes[0].items = b
            .lane0
            .iter()
            .map(|it| BeltItem {
                item: item_from_u8(it.item),
                progress: it.dist.clamp(0.0, 1.0),
            })
            .collect();
        tile.lanes[1].items = b
            .lane1
            .iter()
            .map(|it| BeltItem {
                item: item_from_u8(it.item),
                progress: it.dist.clamp(0.0, 1.0),
            })
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
        if let Ok(save) = read_save(&path) {
            out.push(SaveInfo {
                path,
                label: save.label,
                saved_at: save.saved_at,
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
