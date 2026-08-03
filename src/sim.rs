//! Placeable dual-lane conveyors (travel time = length/speed), power wires, docking.

use std::collections::{HashMap, HashSet, VecDeque};

pub const ORE_RATE: f32 = 7.3;
pub const SMELT_RATE: f32 = 3.9;
pub const SOLAR_POWER: f32 = 12.0;
pub const ORE_POWER_DRAW: f32 = 4.0;
pub const SMELT_POWER_DRAW: f32 = 8.0;
pub const NODE_BUFFER: f32 = 100.0;
pub const POLE_RADIUS: f32 = 260.0;
pub const BELT_SPEED: f32 = 120.0;
pub const BELT_ITEM_SPACING: f32 = 18.0;
pub const DOCK_DIST: f32 = 40.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Item { IronOre, IronIngot }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Facing { E, S, W, N }

impl Facing {
    pub fn rotate_cw(self) -> Self {
        match self { Facing::E => Facing::S, Facing::S => Facing::W, Facing::W => Facing::N, Facing::N => Facing::E }
    }
    pub fn as_u8(self) -> u8 { match self { Facing::E => 0, Facing::S => 1, Facing::W => 2, Facing::N => 3 } }
    pub fn from_u8(v: u8) -> Self { match v % 4 { 1 => Facing::S, 2 => Facing::W, 3 => Facing::N, _ => Facing::E } }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuildCategory { Energy, Resource, Processing, Storage, Transport }

impl BuildCategory {
    pub const ALL: [BuildCategory; 5] = [Self::Energy, Self::Resource, Self::Processing, Self::Storage, Self::Transport];
    pub fn label(self) -> &'static str {
        match self {
            Self::Energy => "Energy", Self::Resource => "Resource", Self::Processing => "Processing",
            Self::Storage => "Storage", Self::Transport => "Transport",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuildingKind { Solar, PowerPole, OreNode, Smelter, Box, Conveyor, Splitter }

impl BuildingKind {
    pub fn category(self) -> BuildCategory {
        match self {
            Self::Solar | Self::PowerPole => BuildCategory::Energy,
            Self::OreNode => BuildCategory::Resource,
            Self::Smelter => BuildCategory::Processing,
            Self::Box => BuildCategory::Storage,
            Self::Conveyor | Self::Splitter => BuildCategory::Transport,
        }
    }
    pub fn in_category(cat: BuildCategory) -> Vec<BuildingKind> {
        [Self::Solar, Self::PowerPole, Self::OreNode, Self::Smelter, Self::Box, Self::Conveyor, Self::Splitter]
            .into_iter().filter(|k| k.category() == cat).collect()
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Solar => "Solar Panel", Self::PowerPole => "Power Pole", Self::OreNode => "Iron Ore Node",
            Self::Smelter => "Smelter", Self::Box => "Storage Box", Self::Conveyor => "Conveyor", Self::Splitter => "Splitter",
        }
    }
    pub fn short(self) -> &'static str {
        match self {
            Self::Solar => "Solar", Self::PowerPole => "Pole", Self::OreNode => "Ore", Self::Smelter => "Smelt",
            Self::Box => "Box", Self::Conveyor => "Belt", Self::Splitter => "Split",
        }
    }
    pub fn size(self) -> (f32, f32) {
        match self {
            Self::PowerPole => (100.0, 120.0),
            Self::Conveyor => (112.0, 52.0),
            Self::Splitter => (130.0, 100.0),
            _ => (200.0, 128.0),
        }
    }
    pub fn needs_power(self) -> bool { matches!(self, Self::OreNode | Self::Smelter) }
    pub fn can_rotate(self) -> bool { !matches!(self, Self::PowerPole) }
    pub fn is_belt(self) -> bool { matches!(self, Self::Conveyor | Self::Splitter) }
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Solar => 0, Self::PowerPole => 1, Self::OreNode => 2, Self::Smelter => 3,
            Self::Box => 4, Self::Conveyor => 5, Self::Splitter => 6,
        }
    }
    pub fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            0 => Self::Solar, 1 => Self::PowerPole, 2 => Self::OreNode, 3 => Self::Smelter,
            4 => Self::Box, 5 => Self::Conveyor, 6 => Self::Splitter, _ => return None,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PortKind { EnergyOut, EnergyAny, ItemOut(Item), ItemIn(Item), AnyIn, AnyOut }

impl PortKind {
    pub fn is_energy(self) -> bool { matches!(self, Self::EnergyOut | Self::EnergyAny) }
    pub fn is_item_out(self) -> bool { matches!(self, Self::ItemOut(_) | Self::AnyOut) }
    pub fn is_item_in(self) -> bool { matches!(self, Self::ItemIn(_) | Self::AnyIn) }
}

#[derive(Clone, Debug)]
pub struct Port { pub kind: PortKind, pub ox: f32, pub oy: f32 }

#[derive(Clone, Debug)]
pub struct BeltItem { pub item: Item, pub dist: f32 }

#[derive(Clone, Debug, Default)]
pub struct BeltLane { pub items: Vec<BeltItem> }

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: BuildingKind, pub x: f32, pub y: f32, pub facing: Facing,
    pub in_ore: f32, pub out_ore: f32, pub out_ingot: f32,
    pub store_ore: f32, pub store_ingot: f32, pub buf_ore: f32, pub buf_ingot: f32,
    pub working: bool, pub powered: bool, pub ports: Vec<Port>, pub lanes: [BeltLane; 2],
}

impl Node {
    pub fn new(kind: BuildingKind, x: f32, y: f32, facing: Facing) -> Self {
        let mut n = Self {
            kind, x, y, facing,
            in_ore: 0.0, out_ore: 0.0, out_ingot: 0.0, store_ore: 0.0, store_ingot: 0.0,
            buf_ore: 0.0, buf_ingot: 0.0, working: false, powered: false,
            ports: Vec::new(), lanes: [BeltLane::default(), BeltLane::default()],
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
        match self.facing { Facing::E | Facing::W => (bw, bh), Facing::N | Facing::S => (bh, bw) }
    }
    pub fn w(&self) -> f32 { self.size().0 }
    pub fn h(&self) -> f32 { self.size().1 }
    pub fn belt_length(&self) -> f32 {
        match self.facing { Facing::E | Facing::W => self.w(), Facing::N | Facing::S => self.h() }.max(1.0)
    }
    pub fn center(&self) -> (f32, f32) { (self.x + self.w() * 0.5, self.y + self.h() * 0.5) }
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
    pub fn capacity_per_lane(&self) -> usize {
        ((self.belt_length() / BELT_ITEM_SPACING).floor() as usize).clamp(1, 32)
    }
}

fn edge(w: f32, h: f32, side: Facing, along: f32) -> (f32, f32) {
    match side {
        Facing::W => (0.0, h * along), Facing::E => (w, h * along),
        Facing::N => (w * along, 0.0), Facing::S => (w * along, h),
    }
}

fn ports_for(kind: BuildingKind, w: f32, h: f32, facing: Facing) -> Vec<Port> {
    let back = match facing {
        Facing::E => Facing::W, Facing::S => Facing::N, Facing::W => Facing::E, Facing::N => Facing::S,
    };
    let m = 0.5;
    match kind {
        BuildingKind::Solar => { let (ox, oy) = edge(w, h, facing, m); vec![Port { kind: PortKind::EnergyOut, ox, oy }] }
        BuildingKind::PowerPole => {
            let a = edge(w, h, back, m); let b = edge(w, h, facing, m);
            vec![Port { kind: PortKind::EnergyAny, ox: a.0, oy: a.1 }, Port { kind: PortKind::EnergyAny, ox: b.0, oy: b.1 }]
        }
        BuildingKind::OreNode => { let (ox, oy) = edge(w, h, facing, m); vec![Port { kind: PortKind::ItemOut(Item::IronOre), ox, oy }] }
        BuildingKind::Smelter => {
            let i = edge(w, h, back, m); let o = edge(w, h, facing, m);
            vec![Port { kind: PortKind::ItemIn(Item::IronOre), ox: i.0, oy: i.1 }, Port { kind: PortKind::ItemOut(Item::IronIngot), ox: o.0, oy: o.1 }]
        }
        BuildingKind::Box => { let (ox, oy) = edge(w, h, back, m); vec![Port { kind: PortKind::AnyIn, ox, oy }] }
        BuildingKind::Conveyor => {
            let i = edge(w, h, back, m); let o = edge(w, h, facing, m);
            vec![Port { kind: PortKind::AnyIn, ox: i.0, oy: i.1 }, Port { kind: PortKind::AnyOut, ox: o.0, oy: o.1 }]
        }
        BuildingKind::Splitter => {
            let i = edge(w, h, back, m); let o0 = edge(w, h, facing, 0.28); let o1 = edge(w, h, facing, 0.72);
            vec![
                Port { kind: PortKind::AnyIn, ox: i.0, oy: i.1 },
                Port { kind: PortKind::AnyOut, ox: o0.0, oy: o0.1 },
                Port { kind: PortKind::AnyOut, ox: o1.0, oy: o1.1 },
            ]
        }
    }
}

#[derive(Clone, Debug)]
pub struct Link { pub from_node: u32, pub from_port: usize, pub to_node: u32, pub to_port: usize }

pub struct World {
    pub nodes: HashMap<u32, Node>,
    pub links: Vec<Link>,
    pub next_id: u32,
    pub network_energy: HashMap<u32, f32>,
    pub energy_prod: f32,
    pub energy_use: f32,
}

impl World {
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), links: Vec::new(), next_id: 1, network_energy: HashMap::new(), energy_prod: 0.0, energy_use: 0.0 }
    }
    pub fn clear(&mut self) { *self = Self::new(); }

    pub fn place_node(&mut self, kind: BuildingKind, x: f32, y: f32, facing: Facing) -> Option<u32> {
        let probe = Node::new(kind, x, y, facing);
        if self.collides(probe.x, probe.y, probe.w(), probe.h(), None) { return None; }
        let id = self.next_id; self.next_id += 1;
        self.nodes.insert(id, probe); Some(id)
    }

    pub fn place_node_with_id(&mut self, id: u32, kind: BuildingKind, x: f32, y: f32, facing: Facing) -> bool {
        let probe = Node::new(kind, x, y, facing);
        if self.collides(probe.x, probe.y, probe.w(), probe.h(), Some(id)) { return false; }
        if id >= self.next_id { self.next_id = id + 1; }
        self.nodes.insert(id, probe); true
    }

    pub fn try_move_node(&mut self, id: u32, x: f32, y: f32) -> bool {
        let (w, h) = match self.nodes.get(&id) { Some(n) => (n.w(), n.h()), None => return false };
        if self.collides(x, y, w, h, Some(id)) { return false; }
        if let Some(n) = self.nodes.get_mut(&id) { n.x = x; n.y = y; true } else { false }
    }

    pub fn try_rotate_node(&mut self, id: u32) -> bool {
        let (x, y, w, h, next) = {
            let Some(n) = self.nodes.get(&id) else { return false };
            if !n.kind.can_rotate() { return false; }
            let next = n.facing.rotate_cw();
            let (cx, cy) = n.center();
            let (bw, bh) = n.kind.size();
            let (nw, nh) = match next { Facing::E | Facing::W => (bw, bh), Facing::N | Facing::S => (bh, bw) };
            (cx - nw * 0.5, cy - nh * 0.5, nw, nh, next)
        };
        if self.collides(x, y, w, h, Some(id)) { return false; }
        if let Some(n) = self.nodes.get_mut(&id) { n.set_facing(next); true } else { false }
    }

    pub fn collides(&self, x: f32, y: f32, w: f32, h: f32, ignore: Option<u32>) -> bool {
        self.nodes.iter().any(|(&id, n)| Some(id) != ignore && n.overlaps_rect(x, y, w, h))
    }

    pub fn remove_node(&mut self, id: u32) {
        self.nodes.remove(&id);
        self.links.retain(|l| l.from_node != id && l.to_node != id);
    }

    pub fn hit_node(&self, wx: f32, wy: f32) -> Option<u32> {
        let mut best = None;
        for (&id, n) in &self.nodes { if n.contains(wx, wy) { best = Some(id); } }
        best
    }

    pub fn hit_energy_port(&self, wx: f32, wy: f32, radius: f32) -> Option<(u32, usize)> {
        let r2 = radius * radius;
        let mut best = None;
        for (&id, n) in &self.nodes {
            for (pi, p) in n.ports.iter().enumerate() {
                if !p.kind.is_energy() { continue; }
                let d2 = (n.x + p.ox - wx).powi(2) + (n.y + p.oy - wy).powi(2);
                if d2 <= r2 && best.map(|(_, _, bd)| d2 < bd).unwrap_or(true) { best = Some((id, pi, d2)); }
            }
        }
        best.map(|(a, b, _)| (a, b))
    }

    pub fn can_connect_power(&self, from: (u32, usize), to: (u32, usize)) -> bool {
        if from.0 == to.0 { return false; }
        let Some(pa) = self.nodes.get(&from.0).and_then(|n| n.ports.get(from.1)) else { return false };
        let Some(pb) = self.nodes.get(&to.0).and_then(|n| n.ports.get(to.1)) else { return false };
        matches!((pa.kind, pb.kind), (PortKind::EnergyOut, PortKind::EnergyAny) | (PortKind::EnergyAny, PortKind::EnergyAny))
    }

    pub fn connect_power(&mut self, from: (u32, usize), to: (u32, usize)) -> bool {
        let ordered = if self.can_connect_power(from, to) { Some((from, to)) }
            else if self.can_connect_power(to, from) { Some((to, from)) } else { None };
        let Some((from, to)) = ordered else { return false };
        self.links.push(Link { from_node: from.0, from_port: from.1, to_node: to.0, to_port: to.1 });
        true
    }

    fn compatible(out: PortKind, inn: PortKind) -> bool {
        match (out, inn) {
            (PortKind::ItemOut(Item::IronOre), PortKind::ItemIn(Item::IronOre)) => true,
            (PortKind::ItemOut(Item::IronIngot), PortKind::ItemIn(Item::IronIngot)) => true,
            (PortKind::ItemOut(_), PortKind::AnyIn) => true,
            (PortKind::AnyOut, PortKind::AnyIn | PortKind::ItemIn(_)) => true,
            _ => false,
        }
    }

    pub fn find_docks(&self) -> Vec<(u32, usize, u32, usize)> {
        let mut docks = Vec::new();
        let ids: Vec<u32> = self.nodes.keys().copied().collect();
        for &aid in &ids {
            let Some(a) = self.nodes.get(&aid) else { continue };
            for (ai, ap) in a.ports.iter().enumerate() {
                if !ap.kind.is_item_out() { continue; }
                let (ax, ay) = (a.x + ap.ox, a.y + ap.oy);
                for &bid in &ids {
                    if aid == bid { continue; }
                    let Some(b) = self.nodes.get(&bid) else { continue };
                    for (bi, bp) in b.ports.iter().enumerate() {
                        if !bp.kind.is_item_in() || !Self::compatible(ap.kind, bp.kind) { continue; }
                        let dx = ax - (b.x + bp.ox); let dy = ay - (b.y + bp.oy);
                        if dx * dx + dy * dy <= DOCK_DIST * DOCK_DIST { docks.push((aid, ai, bid, bi)); }
                    }
                }
            }
        }
        docks
    }

    pub fn tick(&mut self, dt: f32) {
        let (node_net, gen_by_net, powered_poles) = self.power_step(dt);
        self.machine_step(dt, &node_net, &gen_by_net, &powered_poles);
        self.belt_advance(dt);
        self.dock_transfer();
    }

    fn power_step(&mut self, dt: f32) -> (HashMap<u32, u32>, HashMap<u32, f32>, HashSet<u32>) {
        let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
        for (&id, n) in &self.nodes {
            if matches!(n.kind, BuildingKind::Solar | BuildingKind::PowerPole) { adj.entry(id).or_default(); }
        }
        for l in &self.links {
            adj.entry(l.from_node).or_default().push(l.to_node);
            adj.entry(l.to_node).or_default().push(l.from_node);
        }
        let mut visited = HashSet::new();
        let mut node_net = HashMap::new();
        let mut gen_by_net: HashMap<u32, f32> = HashMap::new();
        for &start in adj.keys() {
            if !visited.insert(start) { continue; }
            let mut q = VecDeque::from([start]);
            let mut members = Vec::new();
            let mut gen = 0.0; let mut root = start;
            while let Some(id) = q.pop_front() {
                members.push(id); root = root.min(id);
                if self.nodes.get(&id).map(|n| n.kind) == Some(BuildingKind::Solar) { gen += SOLAR_POWER; }
                if let Some(neis) = adj.get(&id) {
                    for &n in neis { if visited.insert(n) { q.push_back(n); } }
                }
            }
            for id in members { node_net.insert(id, root); }
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
            if n.kind != BuildingKind::PowerPole { continue; }
            if let Some(&root) = node_net.get(&id) {
                if gen_by_net.get(&root).copied().unwrap_or(0.0) > 0.0 || self.network_energy.get(&root).copied().unwrap_or(0.0) > 0.0 {
                    poles.insert(id);
                }
            }
        }
        let ids: Vec<u32> = self.nodes.keys().copied().collect();
        for id in ids {
            let covered = {
                let Some(n) = self.nodes.get(&id) else { continue };
                if !n.kind.needs_power() { true } else {
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
                    _ => n.working,
                };
            }
        }
        (node_net, gen_by_net, poles)
    }

    fn machine_step(&mut self, dt: f32, node_net: &HashMap<u32, u32>, _gen: &HashMap<u32, f32>, poles: &HashSet<u32>) {
        let mut energy_draw = 0.0;
        let mut ids: Vec<u32> = self.nodes.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            let pay = {
                let Some(n) = self.nodes.get(&id) else { continue };
                if !n.kind.needs_power() { None } else {
                    let (cx, cy) = n.center();
                    let mut best = None;
                    for &pid in poles {
                        let Some(p) = self.nodes.get(&pid) else { continue };
                        let (px, py) = p.center();
                        let d2 = (cx - px).powi(2) + (cy - py).powi(2);
                        if d2 <= POLE_RADIUS * POLE_RADIUS {
                            if let Some(&root) = node_net.get(&pid) {
                                if best.map(|(_, bd)| d2 < bd).unwrap_or(true) { best = Some((root, d2)); }
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
                                        n.out_ore += made; n.working = true;
                                        if let Some(e) = self.network_energy.get_mut(&root) { *e -= cost; }
                                        energy_draw += ORE_POWER_DRAW;
                                    } else { n.working = false; }
                                } else { n.working = false; }
                            }
                        } else if let Some(n) = self.nodes.get_mut(&id) { n.working = false; }
                    }
                }
                Some(BuildingKind::Smelter) if powered => {
                    if let Some(root) = pay {
                        let cost = SMELT_POWER_DRAW * dt;
                        let has = self.network_energy.get(&root).copied().unwrap_or(0.0);
                        if has >= cost {
                            if let Some(n) = self.nodes.get_mut(&id) {
                                if n.in_ore > 0.0 && n.out_ingot < NODE_BUFFER {
                                    let can = (SMELT_RATE * dt).min(n.in_ore).min(NODE_BUFFER - n.out_ingot);
                                    if can > 0.0 {
                                        n.in_ore -= can; n.out_ingot += can; n.working = true;
                                        if let Some(e) = self.network_energy.get_mut(&root) { *e -= cost; }
                                        energy_draw += SMELT_POWER_DRAW;
                                    } else { n.working = false; }
                                } else { n.working = false; }
                            }
                        } else if let Some(n) = self.nodes.get_mut(&id) { n.working = false; }
                    }
                }
                Some(BuildingKind::Conveyor | BuildingKind::Splitter) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.working = n.lanes.iter().any(|l| !l.items.is_empty()) || n.buf_ore + n.buf_ingot > 0.05;
                    }
                }
                _ => {}
            }
        }
        self.energy_use = energy_draw;
    }

    fn belt_advance(&mut self, dt: f32) {
        let ids: Vec<u32> = self.nodes.keys().copied().collect();
        for id in ids {
            let Some(n) = self.nodes.get_mut(&id) else { continue };
            if !n.kind.is_belt() { continue; }
            let len = n.belt_length();
            let speed = BELT_SPEED * dt;
            for lane in 0..2 {
                n.lanes[lane].items.sort_by(|a, b| b.dist.partial_cmp(&a.dist).unwrap_or(std::cmp::Ordering::Equal));
                let mut prev = f32::INFINITY;
                for it in &mut n.lanes[lane].items {
                    let mut nd = it.dist + speed;
                    if prev.is_finite() { nd = nd.min(prev - BELT_ITEM_SPACING); }
                    it.dist = nd.max(it.dist).clamp(0.0, len);
                    prev = it.dist;
                }
            }
        }
    }

    fn take_item(n: &mut Node, prefer: Option<Item>) -> Option<Item> {
        match prefer {
            Some(Item::IronOre) if n.out_ore >= 1.0 => { n.out_ore -= 1.0; Some(Item::IronOre) }
            Some(Item::IronIngot) if n.out_ingot >= 1.0 => { n.out_ingot -= 1.0; Some(Item::IronIngot) }
            None if n.out_ore >= 1.0 => { n.out_ore -= 1.0; Some(Item::IronOre) }
            None if n.out_ingot >= 1.0 => { n.out_ingot -= 1.0; Some(Item::IronIngot) }
            None if n.buf_ore >= 1.0 => { n.buf_ore -= 1.0; Some(Item::IronOre) }
            None if n.buf_ingot >= 1.0 => { n.buf_ingot -= 1.0; Some(Item::IronIngot) }
            _ => None,
        }
    }

    fn accept(n: &mut Node, item: Item) -> bool {
        match (n.kind, item) {
            (BuildingKind::Smelter, Item::IronOre) if n.in_ore + 1.0 <= NODE_BUFFER => { n.in_ore += 1.0; true }
            (BuildingKind::Box, Item::IronOre) => { n.store_ore += 1.0; true }
            (BuildingKind::Box, Item::IronIngot) => { n.store_ingot += 1.0; true }
            (BuildingKind::Splitter, Item::IronOre) if n.buf_ore + 1.0 <= NODE_BUFFER => { n.buf_ore += 1.0; true }
            (BuildingKind::Splitter, Item::IronIngot) if n.buf_ingot + 1.0 <= NODE_BUFFER => { n.buf_ingot += 1.0; true }
            _ => false,
        }
    }

    fn try_board_belt(n: &mut Node, item: Item, lane: usize) -> bool {
        if !n.kind.is_belt() { return false; }
        let cap = n.capacity_per_lane();
        if n.lanes[lane].items.len() >= cap { return false; }
        if n.lanes[lane].items.iter().any(|it| it.dist < BELT_ITEM_SPACING) { return false; }
        n.lanes[lane].items.push(BeltItem { item, dist: 0.0 });
        true
    }

    fn dock_transfer(&mut self) {
        let docks = self.find_docks();
        // Count outbound docks from each (node, out_port) for splitter fairness.
        let mut out_deg: HashMap<(u32, usize), usize> = HashMap::new();
        for &(a, ai, _, _) in &docks { *out_deg.entry((a, ai)).or_insert(0) += 1; }

        for (a, ai, b, _bi) in docks {
            let from_is_belt = self.nodes.get(&a).map(|n| n.kind.is_belt()).unwrap_or(false);
            let to_is_belt = self.nodes.get(&b).map(|n| n.kind.is_belt()).unwrap_or(false);
            let prefer = self.nodes.get(&a).and_then(|n| n.ports.get(ai)).and_then(|p| match p.kind {
                PortKind::ItemOut(i) => Some(i), _ => None,
            });

            if from_is_belt {
                // Pop finished items from belt end onto next.
                let len = self.nodes.get(&a).map(|n| n.belt_length()).unwrap_or(1.0);
                for lane in 0..2 {
                    let item = {
                        let Some(n) = self.nodes.get_mut(&a) else { continue };
                        let Some(idx) = n.lanes[lane].items.iter().position(|it| it.dist >= len - 0.01) else { continue };
                        let it = n.lanes[lane].items.remove(idx);
                        it.item
                    };
                    let delivered = if to_is_belt {
                        let lane_to = lane;
                        if let Some(n) = self.nodes.get_mut(&b) { Self::try_board_belt(n, item, lane_to) } else { false }
                    } else if let Some(n) = self.nodes.get_mut(&b) {
                        Self::accept(n, item)
                    } else { false };
                    if !delivered {
                        if let Some(n) = self.nodes.get_mut(&a) {
                            n.lanes[lane].items.push(BeltItem { item, dist: len });
                        }
                    }
                }
            } else if to_is_belt {
                // Machine/splitter buffer → belt
                let lane = {
                    let Some(n) = self.nodes.get(&b) else { continue };
                    if n.lanes[0].items.len() <= n.lanes[1].items.len() { 0 } else { 1 }
                };
                if let Some(n) = self.nodes.get_mut(&a) {
                    if let Some(item) = Self::take_item(n, prefer) {
                        let ok = if let Some(bn) = self.nodes.get_mut(&b) {
                            Self::try_board_belt(bn, item, lane)
                        } else { false };
                        if !ok {
                            // refund
                            if let Some(n) = self.nodes.get_mut(&a) {
                                match item {
                                    Item::IronOre => { if n.kind == BuildingKind::OreNode { n.out_ore += 1.0; } else { n.buf_ore += 1.0; } }
                                    Item::IronIngot => { if n.kind == BuildingKind::Smelter { n.out_ingot += 1.0; } else { n.buf_ingot += 1.0; } }
                                }
                            }
                        }
                    }
                }
            } else {
                // Direct machine→machine short dock (adjacent)
                let _ = out_deg;
                if let Some(n) = self.nodes.get_mut(&a) {
                    if let Some(item) = Self::take_item(n, prefer) {
                        let ok = if let Some(m) = self.nodes.get_mut(&b) { Self::accept(m, item) } else { false };
                        if !ok {
                            if let Some(n) = self.nodes.get_mut(&a) {
                                match item {
                                    Item::IronOre => n.out_ore += 1.0,
                                    Item::IronIngot => n.out_ingot += 1.0,
                                }
                            }
                        }
                    }
                }
            }
        }

        // Splitter buffer → its own belt lanes (acts as belt segment too)
        let ids: Vec<u32> = self.nodes.keys().copied().collect();
        for id in ids {
            let Some(n) = self.nodes.get_mut(&id) else { continue };
            if n.kind != BuildingKind::Splitter { continue; }
            while n.buf_ore >= 1.0 {
                let lane = if n.lanes[0].items.len() <= n.lanes[1].items.len() { 0 } else { 1 };
                if Self::try_board_belt(n, Item::IronOre, lane) { n.buf_ore -= 1.0; } else { break; }
            }
            while n.buf_ingot >= 1.0 {
                let lane = if n.lanes[0].items.len() <= n.lanes[1].items.len() { 0 } else { 1 };
                if Self::try_board_belt(n, Item::IronIngot, lane) { n.buf_ingot -= 1.0; } else { break; }
            }
        }
    }
}

/// World position of an item on a conveyor segment (lane 0 left / 1 right).
pub fn belt_item_world(n: &Node, lane: usize, dist: f32) -> (f32, f32) {
    let len = n.belt_length();
    let t = (dist / len).clamp(0.0, 1.0);
    let back = match n.facing {
        Facing::E => Facing::W, Facing::S => Facing::N, Facing::W => Facing::E, Facing::N => Facing::S,
    };
    let (ix, iy) = n.port_world(0).unwrap_or(n.center());
    // For conveyor port0=in port1=out; splitter port0=in
    let (ox, oy) = if n.kind == BuildingKind::Conveyor {
        n.port_world(1).unwrap_or(n.center())
    } else {
        // approximate along facing through center
        let (cx, cy) = n.center();
        match n.facing {
            Facing::E => (n.x + n.w(), cy), Facing::W => (n.x, cy),
            Facing::S => (cx, n.y + n.h()), Facing::N => (cx, n.y),
        }
    };
    let _ = (back, ix, iy);
    let px = ix + (ox - ix) * t;
    let py = iy + (oy - iy) * t;
    let (tx, ty) = (ox - ix, oy - iy);
    let llen = (tx * tx + ty * ty).sqrt().max(1e-3);
    let (nx, ny) = (-ty / llen, tx / llen);
    let sep = 6.0;
    let s = if lane == 0 { -sep } else { sep };
    (px + nx * s, py + ny * s)
}
