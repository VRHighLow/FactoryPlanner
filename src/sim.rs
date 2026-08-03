//! Buildings, power wires, and port-to-port belt links (no conveyor tiles).

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
}

impl BuildCategory {
    pub const ALL: [BuildCategory; 5] = [
        Self::Energy,
        Self::Resource,
        Self::Processing,
        Self::Storage,
        Self::Transport,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Energy => "Energy",
            Self::Resource => "Resource",
            Self::Processing => "Processing",
            Self::Storage => "Storage",
            Self::Transport => "Transport",
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
}

impl BuildingKind {
    pub fn category(self) -> BuildCategory {
        match self {
            Self::Solar | Self::PowerPole => BuildCategory::Energy,
            Self::OreNode => BuildCategory::Resource,
            Self::Smelter => BuildCategory::Processing,
            Self::Box => BuildCategory::Storage,
            Self::Splitter => BuildCategory::Transport,
        }
    }
    pub fn in_category(cat: BuildCategory) -> Vec<BuildingKind> {
        [
            Self::Solar,
            Self::PowerPole,
            Self::OreNode,
            Self::Smelter,
            Self::Box,
            Self::Splitter,
        ]
        .into_iter()
        .filter(|k| k.category() == cat)
        .collect()
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Solar => "Solar Panel",
            Self::PowerPole => "Power Pole",
            Self::OreNode => "Iron Ore Node",
            Self::Smelter => "Smelter",
            Self::Box => "Storage Box",
            Self::Splitter => "Splitter",
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
        }
    }
    pub fn size(self) -> (f32, f32) {
        match self {
            Self::PowerPole => (100.0, 120.0),
            Self::Splitter => (130.0, 100.0),
            _ => (200.0, 128.0),
        }
    }
    pub fn needs_power(self) -> bool {
        matches!(self, Self::OreNode | Self::Smelter)
    }
    pub fn can_rotate(self) -> bool {
        !matches!(self, Self::PowerPole)
    }
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Solar => 0,
            Self::PowerPole => 1,
            Self::OreNode => 2,
            Self::Smelter => 3,
            Self::Box => 4,
            Self::Splitter => 5,
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
    pub fn is_item_out(self) -> bool {
        matches!(self, Self::ItemOut(_) | Self::AnyOut)
    }
}

#[derive(Clone, Debug)]
pub struct Port {
    pub kind: PortKind,
    pub ox: f32,
    pub oy: f32,
}

#[derive(Clone, Debug)]
pub struct BeltItem {
    pub item: Item,
    pub dist: f32,
}

#[derive(Clone, Debug, Default)]
pub struct BeltLane {
    pub items: Vec<BeltItem>,
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
    pub working: bool,
    pub powered: bool,
    pub ports: Vec<Port>,
}

impl Node {
    pub fn new(kind: BuildingKind, x: f32, y: f32, facing: Facing) -> Self {
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
            working: false,
            powered: false,
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
            let i = edge(w, h, back, m);
            let o0 = edge(w, h, facing, 0.28);
            let o1 = edge(w, h, facing, 0.72);
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
    }
}

#[derive(Clone, Debug)]
pub struct Link {
    pub from_node: u32,
    pub from_port: usize,
    pub to_node: u32,
    pub to_port: usize,
}

#[derive(Clone, Debug)]
pub struct BeltLink {
    pub from_node: u32,
    pub from_port: usize,
    pub to_node: u32,
    pub to_port: usize,
    pub lanes: [BeltLane; 2],
}

impl BeltLink {
    pub fn new(from_node: u32, from_port: usize, to_node: u32, to_port: usize) -> Self {
        Self {
            from_node,
            from_port,
            to_node,
            to_port,
            lanes: [BeltLane::default(), BeltLane::default()],
        }
    }

    pub fn length(&self, world: &World) -> f32 {
        let Some(a) = world.nodes.get(&self.from_node) else {
            return 1.0;
        };
        let Some(b) = world.nodes.get(&self.to_node) else {
            return 1.0;
        };
        let Some((ax, ay)) = a.port_world(self.from_port) else {
            return 1.0;
        };
        let Some((bx, by)) = b.port_world(self.to_port) else {
            return 1.0;
        };
        ((bx - ax).hypot(by - ay)).max(40.0)
    }

    pub fn capacity(&self, world: &World) -> usize {
        ((self.length(world) / BELT_ITEM_SPACING).floor() as usize).clamp(1, 48)
    }
}

pub struct World {
    pub nodes: HashMap<u32, Node>,
    pub links: Vec<Link>,
    pub belts: Vec<BeltLink>,
    pub next_id: u32,
    pub network_energy: HashMap<u32, f32>,
    pub energy_prod: f32,
    pub energy_use: f32,
}

impl World {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            links: Vec::new(),
            belts: Vec::new(),
            next_id: 1,
            network_energy: HashMap::new(),
            energy_prod: 0.0,
            energy_use: 0.0,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn set_id_namespace(&mut self, player_id: u8) {
        let base = (player_id as u32 + 1) * 1_000_000;
        if self.next_id < base {
            self.next_id = base;
        }
    }

    pub fn place_node(&mut self, kind: BuildingKind, x: f32, y: f32, facing: Facing) -> Option<u32> {
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
        let probe = Node::new(kind, x, y, facing);
        if id >= self.next_id {
            self.next_id = id + 1;
        }
        // Remote sync always wins so peers converge even after local collisions.
        self.nodes.insert(id, probe);
        true
    }

    pub fn try_move_node(&mut self, id: u32, x: f32, y: f32) -> bool {
        let (w, h) = match self.nodes.get(&id) {
            Some(n) => (n.w(), n.h()),
            None => return false,
        };
        if self.collides(x, y, w, h, Some(id)) {
            return false;
        }
        if let Some(n) = self.nodes.get_mut(&id) {
            n.x = x;
            n.y = y;
            true
        } else {
            false
        }
    }

    pub fn force_move_node(&mut self, id: u32, x: f32, y: f32) {
        if let Some(n) = self.nodes.get_mut(&id) {
            n.x = x;
            n.y = y;
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
            true
        } else {
            false
        }
    }

    pub fn force_set_facing(&mut self, id: u32, facing: Facing) {
        if let Some(n) = self.nodes.get_mut(&id) {
            n.set_facing(facing);
        }
    }

    pub fn collides(&self, x: f32, y: f32, w: f32, h: f32, ignore: Option<u32>) -> bool {
        self.nodes
            .iter()
            .any(|(&id, n)| Some(id) != ignore && n.overlaps_rect(x, y, w, h))
    }

    pub fn remove_node(&mut self, id: u32) {
        self.nodes.remove(&id);
        self.links
            .retain(|l| l.from_node != id && l.to_node != id);
        self.belts
            .retain(|l| l.from_node != id && l.to_node != id);
    }

    pub fn hit_node(&self, wx: f32, wy: f32) -> Option<u32> {
        let mut best = None;
        for (&id, n) in &self.nodes {
            if n.contains(wx, wy) {
                best = Some(id);
            }
        }
        best
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

    pub fn can_connect_power(&self, from: (u32, usize), to: (u32, usize)) -> bool {
        if from.0 == to.0 {
            return false;
        }
        let Some(pa) = self.nodes.get(&from.0).and_then(|n| n.ports.get(from.1)) else {
            return false;
        };
        let Some(pb) = self.nodes.get(&to.0).and_then(|n| n.ports.get(to.1)) else {
            return false;
        };
        matches!(
            (pa.kind, pb.kind),
            (PortKind::EnergyOut, PortKind::EnergyAny)
                | (PortKind::EnergyAny, PortKind::EnergyAny)
        )
    }

    pub fn connect_power(&mut self, from: (u32, usize), to: (u32, usize)) -> bool {
        let ordered = if self.can_connect_power(from, to) {
            Some((from, to))
        } else if self.can_connect_power(to, from) {
            Some((to, from))
        } else {
            None
        };
        let Some((from, to)) = ordered else {
            return false;
        };
        if self.links.iter().any(|l| {
            (l.from_node, l.from_port, l.to_node, l.to_port)
                == (from.0, from.1, to.0, to.1)
                || (l.from_node, l.from_port, l.to_node, l.to_port)
                    == (to.0, to.1, from.0, from.1)
        }) {
            return false;
        }
        self.links.push(Link {
            from_node: from.0,
            from_port: from.1,
            to_node: to.0,
            to_port: to.1,
        });
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

    pub fn can_connect_belt(&self, from: (u32, usize), to: (u32, usize)) -> bool {
        if from.0 == to.0 {
            return false;
        }
        let Some(pa) = self.nodes.get(&from.0).and_then(|n| n.ports.get(from.1)) else {
            return false;
        };
        let Some(pb) = self.nodes.get(&to.0).and_then(|n| n.ports.get(to.1)) else {
            return false;
        };
        if !Self::compatible(pa.kind, pb.kind) {
            return false;
        }
        // One belt out per output port (ore/smelter/splitter outs).
        if self
            .belts
            .iter()
            .any(|l| l.from_node == from.0 && l.from_port == from.1)
        {
            return false;
        }
        true
    }

    pub fn connect_belt(&mut self, from: (u32, usize), to: (u32, usize)) -> bool {
        let ordered = if self.can_connect_belt(from, to) {
            Some((from, to))
        } else if self.can_connect_belt(to, from) {
            Some((to, from))
        } else {
            None
        };
        let Some((from, to)) = ordered else {
            return false;
        };
        if self.belts.iter().any(|l| {
            (l.from_node, l.from_port, l.to_node, l.to_port) == (from.0, from.1, to.0, to.1)
        }) {
            return false;
        }
        self.belts
            .push(BeltLink::new(from.0, from.1, to.0, to.1));
        true
    }

    /// Connect either power or belt depending on port kinds.
    /// Returns (is_power, from, to) with ordered endpoints.
    pub fn connect_ports(
        &mut self,
        from: (u32, usize),
        to: (u32, usize),
    ) -> Option<(bool, (u32, usize), (u32, usize))> {
        let ordered_power = if self.can_connect_power(from, to) {
            Some((from, to))
        } else if self.can_connect_power(to, from) {
            Some((to, from))
        } else {
            None
        };
        if let Some((a, b)) = ordered_power {
            if self.connect_power(a, b) {
                return Some((true, a, b));
            }
        }
        let ordered_belt = if self.can_connect_belt(from, to) {
            Some((from, to))
        } else if self.can_connect_belt(to, from) {
            Some((to, from))
        } else {
            None
        };
        if let Some((a, b)) = ordered_belt {
            if self.connect_belt(a, b) {
                return Some((false, a, b));
            }
        }
        None
    }

    pub fn tick(&mut self, dt: f32) {
        let (node_net, gen_by_net, powered_poles) = self.power_step(dt);
        self.machine_step(dt, &node_net, &gen_by_net, &powered_poles);
        self.belt_advance(dt);
        self.belt_transfer();
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
                _ => {}
            }
        }
        self.energy_use = energy_draw;
    }

    fn belt_advance(&mut self, dt: f32) {
        let lens: Vec<f32> = self.belts.iter().map(|b| b.length(self)).collect();
        for (i, belt) in self.belts.iter_mut().enumerate() {
            let len = lens[i];
            let speed = BELT_SPEED * dt;
            for lane in 0..2 {
                belt.lanes[lane]
                    .items
                    .sort_by(|a, b| b.dist.partial_cmp(&a.dist).unwrap_or(std::cmp::Ordering::Equal));
                let mut prev = f32::INFINITY;
                for it in &mut belt.lanes[lane].items {
                    let mut nd = it.dist + speed;
                    if prev.is_finite() {
                        nd = nd.min(prev - BELT_ITEM_SPACING);
                    }
                    it.dist = nd.max(it.dist).clamp(0.0, len);
                    prev = it.dist;
                }
            }
        }
    }

    fn take_item(n: &mut Node, prefer: Option<Item>) -> Option<Item> {
        match prefer {
            Some(Item::IronOre) if n.out_ore >= 1.0 => {
                n.out_ore -= 1.0;
                Some(Item::IronOre)
            }
            Some(Item::IronIngot) if n.out_ingot >= 1.0 => {
                n.out_ingot -= 1.0;
                Some(Item::IronIngot)
            }
            None if n.out_ore >= 1.0 => {
                n.out_ore -= 1.0;
                Some(Item::IronOre)
            }
            None if n.out_ingot >= 1.0 => {
                n.out_ingot -= 1.0;
                Some(Item::IronIngot)
            }
            None if n.buf_ore >= 1.0 => {
                n.buf_ore -= 1.0;
                Some(Item::IronOre)
            }
            None if n.buf_ingot >= 1.0 => {
                n.buf_ingot -= 1.0;
                Some(Item::IronIngot)
            }
            _ => None,
        }
    }

    fn accept(n: &mut Node, item: Item) -> bool {
        match (n.kind, item) {
            (BuildingKind::Smelter, Item::IronOre) if n.in_ore + 1.0 <= NODE_BUFFER => {
                n.in_ore += 1.0;
                true
            }
            (BuildingKind::Box, Item::IronOre) => {
                n.store_ore += 1.0;
                true
            }
            (BuildingKind::Box, Item::IronIngot) => {
                n.store_ingot += 1.0;
                true
            }
            (BuildingKind::Splitter, Item::IronOre) if n.buf_ore + 1.0 <= NODE_BUFFER => {
                n.buf_ore += 1.0;
                true
            }
            (BuildingKind::Splitter, Item::IronIngot) if n.buf_ingot + 1.0 <= NODE_BUFFER => {
                n.buf_ingot += 1.0;
                true
            }
            _ => false,
        }
    }

    fn try_board(belt: &mut BeltLink, item: Item, lane: usize, cap: usize) -> bool {
        if belt.lanes[lane].items.len() >= cap {
            return false;
        }
        if belt.lanes[lane]
            .items
            .iter()
            .any(|it| it.dist < BELT_ITEM_SPACING)
        {
            return false;
        }
        belt.lanes[lane].items.push(BeltItem { item, dist: 0.0 });
        true
    }

    fn belt_transfer(&mut self) {
        // Deliver finished items into destination buildings.
        let lens: Vec<f32> = self.belts.iter().map(|b| b.length(self)).collect();
        for bi in 0..self.belts.len() {
            let len = lens[bi];
            let (to_node, _to_port) = {
                let b = &self.belts[bi];
                (b.to_node, b.to_port)
            };
            for lane in 0..2 {
                loop {
                    let item = {
                        let belt = &mut self.belts[bi];
                        let Some(idx) = belt.lanes[lane]
                            .items
                            .iter()
                            .position(|it| it.dist >= len - 0.01)
                        else {
                            break;
                        };
                        belt.lanes[lane].items.remove(idx).item
                    };
                    let delivered = if let Some(n) = self.nodes.get_mut(&to_node) {
                        Self::accept(n, item)
                    } else {
                        false
                    };
                    if !delivered {
                        self.belts[bi]
                            .lanes[lane]
                            .items
                            .push(BeltItem { item, dist: len });
                        break;
                    }
                }
            }
        }

        // Emit from source buildings onto belts leaving their out ports.
        let caps: Vec<usize> = self.belts.iter().map(|b| b.capacity(self)).collect();
        for bi in 0..self.belts.len() {
            let (from_node, from_port) = {
                let b = &self.belts[bi];
                (b.from_node, b.from_port)
            };
            let prefer = self
                .nodes
                .get(&from_node)
                .and_then(|n| n.ports.get(from_port))
                .and_then(|p| match p.kind {
                    PortKind::ItemOut(i) => Some(i),
                    _ => None,
                });
            let lane = {
                let b = &self.belts[bi];
                if b.lanes[0].items.len() <= b.lanes[1].items.len() {
                    0
                } else {
                    1
                }
            };
            let cap = caps[bi];
            if let Some(n) = self.nodes.get_mut(&from_node) {
                if let Some(item) = Self::take_item(n, prefer) {
                    let ok = Self::try_board(&mut self.belts[bi], item, lane, cap);
                    if !ok {
                        if let Some(n) = self.nodes.get_mut(&from_node) {
                            match item {
                                Item::IronOre => {
                                    if n.kind == BuildingKind::OreNode {
                                        n.out_ore += 1.0;
                                    } else {
                                        n.buf_ore += 1.0;
                                    }
                                }
                                Item::IronIngot => {
                                    if n.kind == BuildingKind::Smelter {
                                        n.out_ingot += 1.0;
                                    } else {
                                        n.buf_ingot += 1.0;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Mark splitters working if belts have cargo leaving them.
        for belt in &self.belts {
            if self.nodes.get(&belt.from_node).map(|n| n.kind) == Some(BuildingKind::Splitter) {
                if belt.lanes.iter().any(|l| !l.items.is_empty()) {
                    if let Some(n) = self.nodes.get_mut(&belt.from_node) {
                        n.working = true;
                    }
                }
            }
        }
    }
}

pub fn belt_item_world(world: &World, belt: &BeltLink, lane: usize, dist: f32) -> (f32, f32) {
    let len = belt.length(world);
    let t = (dist / len).clamp(0.0, 1.0);
    let (ix, iy) = world
        .nodes
        .get(&belt.from_node)
        .and_then(|n| n.port_world(belt.from_port))
        .unwrap_or((0.0, 0.0));
    let (ox, oy) = world
        .nodes
        .get(&belt.to_node)
        .and_then(|n| n.port_world(belt.to_port))
        .unwrap_or((0.0, 0.0));
    // Manhattan path mid-point matching draw_power_manhattan (H→V→H).
    let mx = (ix + ox) * 0.5;
    let (px, py) = if t < 0.33 {
        let u = t / 0.33;
        (ix + (mx - ix) * u, iy)
    } else if t < 0.66 {
        let u = (t - 0.33) / 0.33;
        (mx, iy + (oy - iy) * u)
    } else {
        let u = (t - 0.66) / 0.34;
        (mx + (ox - mx) * u, oy)
    };
    let (tx, ty) = if t < 0.33 {
        (mx - ix, 0.0)
    } else if t < 0.66 {
        (0.0, oy - iy)
    } else {
        (ox - mx, 0.0)
    };
    let llen = (tx * tx + ty * ty).sqrt().max(1e-3);
    let (nx, ny) = (-ty / llen, tx / llen);
    let sep = 6.0;
    let s = if lane == 0 { -sep } else { sep };
    (px + nx * s, py + ny * s)
}
