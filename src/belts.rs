//! Factorio-style grid belts: placeable tiles, dual lanes, turns, and sideloading.
//!
//! Lane balance tricks work the same way as Factorio: U-turn / side-feed loops
//! compress items onto one lane; rejoining from the other side fills the other.

use crate::sim::{BuildingKind, Facing, Item, Node, PortKind, World, NODE_BUFFER};
use std::collections::HashMap;

/// World units per belt / building grid cell (matches visual GRID_MINOR).
pub const TILE_SIZE: f32 = 40.0;
/// How many tiles an item crosses per second (≈ yellow-belt pace).
pub const BELT_TILES_PER_SEC: f32 = 3.0;
/// Minimum progress gap between items on one lane (~4–5 items / tile).
pub const LANE_GAP: f32 = 0.22;

#[derive(Clone, Debug)]
pub struct BeltItem {
    pub item: Item,
    /// 0 = back (entry), 1 = front (exit).
    pub progress: f32,
    /// Carried purity 0..100 (Era 1 signature system).
    pub purity: f32,
}

impl BeltItem {
    pub fn with_purity(item: Item, progress: f32, purity: f32) -> Self {
        Self {
            item,
            progress,
            purity,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BeltLane {
    pub items: Vec<BeltItem>,
}

#[derive(Clone, Debug)]
pub struct BeltTile {
    pub dir: Facing,
    pub lanes: [BeltLane; 2],
}

impl BeltTile {
    pub fn new(dir: Facing) -> Self {
        Self {
            dir,
            lanes: [BeltLane::default(), BeltLane::default()],
        }
    }

    pub fn item_count(&self) -> usize {
        self.lanes[0].items.len() + self.lanes[1].items.len()
    }
}

pub type BeltGrid = HashMap<(i32, i32), BeltTile>;

pub fn world_to_tile(x: f32, y: f32) -> (i32, i32) {
    (
        (x / TILE_SIZE).floor() as i32,
        (y / TILE_SIZE).floor() as i32,
    )
}

pub fn tile_origin(tx: i32, ty: i32) -> (f32, f32) {
    (tx as f32 * TILE_SIZE, ty as f32 * TILE_SIZE)
}

pub fn tile_center(tx: i32, ty: i32) -> (f32, f32) {
    (
        (tx as f32 + 0.5) * TILE_SIZE,
        (ty as f32 + 0.5) * TILE_SIZE,
    )
}

/// Snap a building's top-left so its footprint sits on the tile grid.
/// `size` is the unrotated footprint (Era machines pass explicit size).
pub fn snap_building_xy_size(size: (f32, f32), facing: Facing, cx: f32, cy: f32) -> (f32, f32) {
    let (bw, bh) = match facing {
        Facing::N | Facing::S => (size.1, size.0),
        _ => size,
    };
    let tw = (bw / TILE_SIZE).round().max(1.0) as i32;
    let th = (bh / TILE_SIZE).round().max(1.0) as i32;
    let tcx = (cx / TILE_SIZE).floor() as i32;
    let tcy = (cy / TILE_SIZE).floor() as i32;
    // Center the footprint on the cursor tile.
    let tx = tcx - tw / 2;
    let ty = tcy - th / 2;
    tile_origin(tx, ty)
}

pub fn facing_delta(f: Facing) -> (i32, i32) {
    match f {
        Facing::E => (1, 0),
        Facing::W => (-1, 0),
        Facing::S => (0, 1),
        Facing::N => (0, -1),
    }
}

pub fn facing_opposite(f: Facing) -> Facing {
    f.rotate_cw().rotate_cw()
}

pub fn facing_left(f: Facing) -> Facing {
    // Counter-clockwise.
    match f {
        Facing::E => Facing::N,
        Facing::N => Facing::W,
        Facing::W => Facing::S,
        Facing::S => Facing::E,
    }
}

pub fn facing_right(f: Facing) -> Facing {
    f.rotate_cw()
}

fn neighbor(tx: i32, ty: i32, dir: Facing) -> (i32, i32) {
    let (dx, dy) = facing_delta(dir);
    (tx + dx, ty + dy)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BeltShape {
    Straight,
    /// Incoming from the left (relative to exit dir) — counter-clockwise bend.
    CornerLeft,
    /// Incoming from the right (relative to exit dir) — clockwise bend.
    CornerRight,
}

/// How this belt tile bends, based on which neighbor feeds into its back/sides.
pub fn belt_shape(grid: &BeltGrid, tx: i32, ty: i32, dir: Facing) -> BeltShape {
    let behind = neighbor(tx, ty, facing_opposite(dir));
    if grid
        .get(&behind)
        .map(|b| b.dir == dir)
        .unwrap_or(false)
    {
        return BeltShape::Straight;
    }
    // Neighbor on our left outputting into us (faces toward this tile).
    let left_c = neighbor(tx, ty, facing_left(dir));
    if grid
        .get(&left_c)
        .map(|b| b.dir == facing_right(dir))
        .unwrap_or(false)
    {
        return BeltShape::CornerLeft;
    }
    let right_c = neighbor(tx, ty, facing_right(dir));
    if grid
        .get(&right_c)
        .map(|b| b.dir == facing_left(dir))
        .unwrap_or(false)
    {
        return BeltShape::CornerRight;
    }
    BeltShape::Straight
}

/// Factorio lane rule on a 90° bend: traveler left/right stays the same index
/// (left track stays left track). Physical edges follow the curved belt.
pub fn turn_lane(_from_dir: Facing, _to_dir: Facing, lane: usize) -> usize {
    lane.min(1)
}

/// Lane closest to an approach from `from_dir` into a belt facing `dest_dir`.
/// Lane 0 = left, 1 = right when looking in the belt's travel direction.
fn near_lane(dest_dir: Facing, from_dir: Facing) -> usize {
    let approach_side = facing_opposite(from_dir);
    if approach_side == facing_left(dest_dir) {
        0
    } else {
        1
    }
}

/// Which belt lane a source at `(from_x, from_y)` maps to on a belt facing `dir`.
/// Lane 0 = left, 1 = right when looking in the belt's travel direction (Factorio).
pub fn lane_from_side(dir: Facing, belt_tx: i32, belt_ty: i32, from_x: f32, from_y: f32) -> usize {
    let (bcx, bcy) = tile_center(belt_tx, belt_ty);
    let (rx, ry) = facing_right(dir).into_delta_f();
    let side = (from_x - bcx) * rx + (from_y - bcy) * ry;
    if side >= 0.0 {
        1
    } else {
        0
    }
}

fn lane_can_accept(lane: &BeltLane, at_progress: f32) -> bool {
    !lane
        .items
        .iter()
        .any(|it| (it.progress - at_progress).abs() < LANE_GAP)
}

fn try_enter(tile: &mut BeltTile, lane: usize, item: Item, progress: f32) -> bool {
    try_enter_purity(tile, lane, item, progress, 50.0)
}

fn try_enter_purity(
    tile: &mut BeltTile,
    lane: usize,
    item: Item,
    progress: f32,
    purity: f32,
) -> bool {
    let lane = lane.min(1);
    if !lane_can_accept(&tile.lanes[lane], progress) {
        return false;
    }
    tile.lanes[lane]
        .items
        .push(BeltItem::with_purity(item, progress, purity));
    true
}

fn building_covers_tile(n: &Node, tx: i32, ty: i32) -> bool {
    let (x0, y0) = tile_origin(tx, ty);
    let x1 = x0 + TILE_SIZE;
    let y1 = y0 + TILE_SIZE;
    n.x < x1 && n.x + n.w() > x0 && n.y < y1 && n.y + n.h() > y0
}

fn find_building_at(world: &World, tx: i32, ty: i32) -> Option<u32> {
    for (&id, n) in &world.nodes {
        if n.kind.is_cable() || n.kind == BuildingKind::Conveyor {
            continue;
        }
        if building_covers_tile(n, tx, ty) {
            return Some(id);
        }
    }
    None
}

/// World position of an item on a belt tile (straight or curved).
pub fn item_world_pos_shaped(
    tx: i32,
    ty: i32,
    dir: Facing,
    shape: BeltShape,
    lane: usize,
    progress: f32,
) -> (f32, f32) {
    let (cx, cy) = tile_center(tx, ty);
    let sep = TILE_SIZE * 0.22;
    let t = progress.clamp(0.0, 1.0);
    let lane = lane.min(1);

    match shape {
        BeltShape::Straight => {
            let (dx, dy) = facing_delta(dir);
            let along = (t - 0.5) * TILE_SIZE;
            let (lx, ly) = facing_left(dir).into_delta_f();
            let lane_side = if lane == 0 { 1.0 } else { -1.0 };
            (
                cx + dx as f32 * along + lx * sep * lane_side,
                cy + dy as f32 * along + ly * sep * lane_side,
            )
        }
        BeltShape::CornerLeft | BeltShape::CornerRight => {
            // Quarter-circle from entry-edge slot → exit-edge slot, pivoted on the
            // empty inner corner so both lanes hug the bend (not a chord that
            // misses the join on the next belt).
            let entry_edge = match shape {
                BeltShape::CornerLeft => facing_left(dir),
                BeltShape::CornerRight => facing_right(dir),
                BeltShape::Straight => unreachable!(),
            };
            let inward = facing_opposite(entry_edge);
            let (x0, y0) = edge_lane_point(cx, cy, entry_edge, inward, lane, sep);
            let (x1, y1) = edge_lane_point(cx, cy, dir, dir, lane, sep);

            // Pivot = empty corner of the tile (inner elbow).
            let (ex, ey) = facing_delta(facing_opposite(entry_edge));
            let (xx, xy) = facing_delta(facing_opposite(dir));
            let px = cx + (ex + xx) as f32 * TILE_SIZE * 0.5;
            let py = cy + (ey + xy) as f32 * TILE_SIZE * 0.5;

            let a0 = (y0 - py).atan2(x0 - px);
            let a1 = (y1 - py).atan2(x1 - px);
            // Shortest signed delta matching the bend direction.
            let mut da = a1 - a0;
            let ccw = matches!(shape, BeltShape::CornerLeft);
            if ccw {
                while da <= 0.0 {
                    da += std::f32::consts::TAU;
                }
                while da > std::f32::consts::TAU {
                    da -= std::f32::consts::TAU;
                }
            } else {
                while da >= 0.0 {
                    da -= std::f32::consts::TAU;
                }
                while da < -std::f32::consts::TAU {
                    da += std::f32::consts::TAU;
                }
            }
            let r0 = ((x0 - px).hypot(y0 - py)).max(1.0);
            let r1 = ((x1 - px).hypot(y1 - py)).max(1.0);
            let r = r0 + (r1 - r0) * t;
            let a = a0 + da * t;
            (px + a.cos() * r, py + a.sin() * r)
        }
    }
}

/// Point on the `edge_out` side of a tile, offset into `travel`'s left/right lane.
fn edge_lane_point(
    cx: f32,
    cy: f32,
    edge_out: Facing,
    travel: Facing,
    lane: usize,
    sep: f32,
) -> (f32, f32) {
    let (ox, oy) = facing_delta(edge_out);
    let (lx, ly) = facing_left(travel).into_delta_f();
    let lane_side = if lane == 0 { 1.0 } else { -1.0 };
    (
        cx + ox as f32 * TILE_SIZE * 0.5 + lx * sep * lane_side,
        cy + oy as f32 * TILE_SIZE * 0.5 + ly * sep * lane_side,
    )
}

trait FacingExt {
    fn into_delta_f(self) -> (f32, f32);
}

impl FacingExt for Facing {
    fn into_delta_f(self) -> (f32, f32) {
        let (dx, dy) = facing_delta(self);
        (dx as f32, dy as f32)
    }
}

impl World {
    pub fn remove_belt_at(&mut self, tx: i32, ty: i32) -> bool {
        self.belt_tiles.remove(&(tx, ty)).is_some()
    }

    pub fn belt_at(&self, tx: i32, ty: i32) -> Option<&BeltTile> {
        self.belt_tiles.get(&(tx, ty))
    }

    /// Drag-paint / click place. Overwrites direction if a belt already exists.
    pub fn paint_belt(&mut self, tx: i32, ty: i32, dir: Facing) -> bool {
        if find_building_at(self, tx, ty).is_some() {
            return false;
        }
        if let Some(existing) = self.belt_tiles.get_mut(&(tx, ty)) {
            existing.dir = dir;
            return true;
        }
        self.belt_tiles.insert((tx, ty), BeltTile::new(dir));
        true
    }

    pub fn tile_blocked_by_belt(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let tx0 = (x / TILE_SIZE).floor() as i32;
        let ty0 = (y / TILE_SIZE).floor() as i32;
        let tx1 = ((x + w - 0.01) / TILE_SIZE).floor() as i32;
        let ty1 = ((y + h - 0.01) / TILE_SIZE).floor() as i32;
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                if self.belt_tiles.contains_key(&(tx, ty)) {
                    return true;
                }
            }
        }
        false
    }

    /// Advance belts, transfer across tiles (incl. sideload / corners), and
    /// exchange items with adjacent machine ports.
    pub fn belt_grid_step(&mut self, dt: f32) {
        self.belt_deliver_to_machines();
        self.belt_advance_tiles(dt);
        self.belt_transfer_tiles();
        self.belt_emit_from_machines();
    }

    fn belt_advance_tiles(&mut self, dt: f32) {
        let step = BELT_TILES_PER_SEC * dt;
        for tile in self.belt_tiles.values_mut() {
            for lane in &mut tile.lanes {
                lane.items.sort_by(|a, b| b.progress.partial_cmp(&a.progress).unwrap());
                let mut prev = 2.0_f32; // virtual blocker past the front
                for it in &mut lane.items {
                    let mut np = (it.progress + step).min(1.0);
                    if np > prev - LANE_GAP {
                        np = (prev - LANE_GAP).max(it.progress);
                    }
                    it.progress = np.clamp(0.0, 1.0);
                    prev = it.progress;
                }
            }
        }
    }

    fn belt_transfer_tiles(&mut self) {
        // Snapshot keys so we can mutate one tile at a time.
        let mut keys: Vec<(i32, i32)> = self.belt_tiles.keys().copied().collect();
        keys.sort_unstable();

        // Process downstream-first-ish by facing groups to reduce double-steps.
        for &(tx, ty) in &keys {
            let Some(dir) = self.belt_tiles.get(&(tx, ty)).map(|t| t.dir) else {
                continue;
            };
            for lane in 0..2 {
                loop {
                    let ready = self
                        .belt_tiles
                        .get(&(tx, ty))
                        .and_then(|t| {
                            t.lanes[lane]
                                .items
                                .iter()
                                .enumerate()
                                .filter(|(_, it)| it.progress >= 1.0 - 1e-3)
                                .max_by(|a, b| a.1.progress.partial_cmp(&b.1.progress).unwrap())
                                .map(|(i, it)| (i, it.item))
                        });
                    let Some((idx, item)) = ready else {
                        break;
                    };
                    if self.try_leave_belt(tx, ty, dir, lane, item) {
                        if let Some(t) = self.belt_tiles.get_mut(&(tx, ty)) {
                            t.lanes[lane].items.remove(idx);
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn try_leave_belt(&mut self, tx: i32, ty: i32, dir: Facing, lane: usize, item: Item) -> bool {
        let (nx, ny) = neighbor(tx, ty, dir);
        if let Some(next_dir) = self.belt_tiles.get(&(nx, ny)).map(|t| t.dir) {
            return self.try_enter_from(tx, ty, dir, nx, ny, next_dir, lane, item);
        }
        // No belt ahead — leave item jammed at front (delivery handled separately).
        false
    }

    fn try_enter_from(
        &mut self,
        from_tx: i32,
        from_ty: i32,
        from_dir: Facing,
        nx: i32,
        ny: i32,
        next_dir: Facing,
        lane: usize,
        item: Item,
    ) -> bool {
        if next_dir == facing_opposite(from_dir) {
            // Head-on collision — jam.
            return false;
        }

        let behind = neighbor(nx, ny, facing_opposite(next_dir));
        let has_backfeed = self
            .belt_tiles
            .get(&behind)
            .map(|b| b.dir == next_dir)
            .unwrap_or(false);

        let (target_lane, entry_prog) = if next_dir == from_dir {
            // Straight junction — stay on the same track.
            (lane, 0.0)
        } else if !has_backfeed {
            // Curved corner: keep the same left/right track through the bend.
            (turn_lane(from_dir, next_dir, lane), 0.0)
        } else {
            // Sideload into a belt that already has rearward flow → near lane.
            // Progress must match where the item actually crosses the shared edge
            // (corner exits mid/high on that edge; spawning at 0.05 made the ore
            // line miss the join on the destination belt).
            let from_shape = belt_shape(&self.belt_tiles, from_tx, from_ty, from_dir);
            let (ix, iy) =
                item_world_pos_shaped(from_tx, from_ty, from_dir, from_shape, lane, 1.0);
            let (ncx, ncy) = tile_center(nx, ny);
            let (ndx, ndy) = facing_delta(next_dir);
            let along =
                (ix - ncx) * ndx as f32 + (iy - ncy) * ndy as f32;
            let progress = (along / TILE_SIZE + 0.5).clamp(0.02, 0.98);
            (near_lane(next_dir, from_dir), progress)
        };

        let Some(tile) = self.belt_tiles.get_mut(&(nx, ny)) else {
            return false;
        };
        try_enter(tile, target_lane, item, entry_prog)
    }

    fn belt_deliver_to_machines(&mut self) {
        let keys: Vec<(i32, i32)> = self.belt_tiles.keys().copied().collect();
        for (tx, ty) in keys {
            let Some(dir) = self.belt_tiles.get(&(tx, ty)).map(|t| t.dir) else {
                continue;
            };
            let (fx, fy) = neighbor(tx, ty, dir);
            let Some(bid) = find_building_at(self, fx, fy) else {
                continue;
            };
            // Only deliver if the building has an input port facing this belt.
            if !self.building_accepts_from(bid, tx, ty, dir) {
                continue;
            }
            for lane in 0..2 {
                loop {
                    let item = {
                        let Some(tile) = self.belt_tiles.get(&(tx, ty)) else {
                            break;
                        };
                        let Some((idx, it)) = tile.lanes[lane]
                            .items
                            .iter()
                            .enumerate()
                            .filter(|(_, it)| it.progress >= 0.85)
                            .max_by(|a, b| a.1.progress.partial_cmp(&b.1.progress).unwrap())
                        else {
                            break;
                        };
                        (idx, it.item)
                    };
                    if self.accept_item(bid, item.1, Some(lane)) {
                        if let Some(tile) = self.belt_tiles.get_mut(&(tx, ty)) {
                            tile.lanes[lane].items.remove(item.0);
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn building_accepts_from(&self, bid: u32, btx: i32, bty: i32, belt_dir: Facing) -> bool {
        let Some(n) = self.nodes.get(&bid) else {
            return false;
        };
        if n.held {
            return false;
        }
        let has_input = n
            .ports
            .iter()
            .any(|p| matches!(p.kind, PortKind::ItemIn(_) | PortKind::AnyIn));
        if !has_input {
            return false;
        }
        for (pi, p) in n.ports.iter().enumerate() {
            if !matches!(p.kind, PortKind::ItemIn(_) | PortKind::AnyIn) {
                continue;
            }
            let Some((px, py)) = n.port_world(pi) else {
                continue;
            };
            let outward = port_outward_dir(n, p);
            // Port faces the belt that is driving into the building.
            if outward != facing_opposite(belt_dir) {
                continue;
            }
            let (ox, oy) = facing_delta(outward);
            let ax = px + ox as f32 * TILE_SIZE * 0.4;
            let ay = py + oy as f32 * TILE_SIZE * 0.4;
            let (tx, ty) = world_to_tile(ax, ay);
            if (tx, ty) == (btx, bty) {
                return true;
            }
        }
        // Fallback: belt front cell is on the building and it has any item input.
        true
    }

    fn accept_item(&mut self, id: u32, item: Item, from_lane: Option<usize>) -> bool {
        let Some(n) = self.nodes.get_mut(&id) else {
            return false;
        };
        if n.held {
            return false;
        }
        // Filtered item inputs (e.g. ballistic ammo port).
        let item_ok = n.ports.iter().any(|p| match p.kind {
            PortKind::ItemIn(want) => want == item,
            PortKind::AnyIn => true,
            _ => false,
        });
        let has_item_in = n
            .ports
            .iter()
            .any(|p| matches!(p.kind, PortKind::ItemIn(_) | PortKind::AnyIn));
        if has_item_in && !item_ok && matches!(n.kind, BuildingKind::BallisticTurret) {
            return false;
        }
        if let Some(f) = n.fluid_filter {
            if item.is_fluid() && item != f {
                return false;
            }
        }
        match n.kind {
            BuildingKind::Smelter
            | BuildingKind::Assembler
            | BuildingKind::Machine
            | BuildingKind::Lab
            | BuildingKind::Box
            | BuildingKind::NexusSite
            | BuildingKind::BallisticTurret
            | BuildingKind::FluidTank => {
                if item.is_fluid() && !matches!(n.kind, BuildingKind::FluidTank | BuildingKind::Machine | BuildingKind::Lab | BuildingKind::Smelter | BuildingKind::Assembler) {
                    return false;
                }
                if n.stock(item) + 1.0 <= NODE_BUFFER {
                    if n.fluid_filter.is_none() && item.is_fluid() {
                        n.fluid_filter = Some(item);
                    }
                    n.add_stock(item, 1.0);
                    true
                } else {
                    false
                }
            }
            BuildingKind::Splitter => match item {
                Item::IronOre if n.buf_ore + 1.0 <= NODE_BUFFER => {
                    let lane = from_lane.unwrap_or(0).min(1);
                    n.split_ore[lane] = n.split_ore[lane].saturating_add(1);
                    n.buf_ore = (n.split_ore[0] + n.split_ore[1]) as f32;
                    true
                }
                Item::IronIngot if n.buf_ingot + 1.0 <= NODE_BUFFER => {
                    let lane = from_lane.unwrap_or(0).min(1);
                    n.split_ingot[lane] = n.split_ingot[lane].saturating_add(1);
                    n.buf_ingot = (n.split_ingot[0] + n.split_ingot[1]) as f32;
                    true
                }
                // Any other solid — park in generic stocks (single-type-per-lane soft).
                other if !other.is_fluid() && n.stock(other) + 1.0 <= NODE_BUFFER => {
                    n.add_stock(other, 1.0);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn belt_emit_from_machines(&mut self) {
        let ids: Vec<u32> = self.nodes.keys().copied().collect();
        for id in ids {
            let Some(n) = self.nodes.get(&id) else {
                continue;
            };
            if n.kind.is_cable() {
                continue;
            }
            if n.held {
                continue;
            }
            // Splitter: alternate evenly between the two front outputs.
            if n.kind == BuildingKind::Splitter {
                self.belt_emit_splitter(id);
                continue;
            }
            let outs: Vec<(usize, PortKind, f32, f32, Facing)> = n
                .ports
                .iter()
                .enumerate()
                .filter(|(_, p)| p.kind.is_output() && !p.kind.is_energy())
                .filter_map(|(i, p)| {
                    let (px, py) = n.port_world(i)?;
                    let dir = port_outward_dir(n, p);
                    Some((i, p.kind, px, py, dir))
                })
                .collect();
            let _ = n;

            for (_pi, kind, px, py, outward) in outs {
                let (ox, oy) = facing_delta(outward);
                let ax = px + ox as f32 * TILE_SIZE * 0.4;
                let ay = py + oy as f32 * TILE_SIZE * 0.4;
                let (tx, ty) = world_to_tile(ax, ay);
                let Some(tile_dir) = self.belt_tiles.get(&(tx, ty)).map(|t| t.dir) else {
                    continue;
                };

                let prefer = match kind {
                    PortKind::ItemOut(i) => Some(i),
                    _ => None,
                };
                let Some(item) = self.take_output_item(id, prefer) else {
                    continue;
                };

                // Factorio lane rules:
                // - Side feed → near (approach) lane only
                // - From behind (same travel dir) or along the belt → lane matching
                //   which side of the belt the machine/port sits on, and stay there
                let primary_lane = if outward == facing_left(tile_dir)
                    || outward == facing_right(tile_dir)
                {
                    near_lane(tile_dir, outward)
                } else {
                    lane_from_side(tile_dir, tx, ty, px, py)
                };

                let placed = {
                    let Some(tile) = self.belt_tiles.get_mut(&(tx, ty)) else {
                        self.refund_item(id, item);
                        continue;
                    };
                    // Prefer the correct side; only spill to the other lane if jammed
                    // and we're feeding from behind (miner-in-front style).
                    if try_enter(tile, primary_lane, item, 0.0) {
                        true
                    } else if outward == tile_dir || outward == facing_opposite(tile_dir) {
                        try_enter(tile, 1 - primary_lane, item, 0.0)
                    } else {
                        false
                    }
                };
                if !placed {
                    self.refund_item(id, item);
                }
            }
        }
    }

    /// Factorio-style: preserve input lane on both outputs; alternate sides per lane.
    fn belt_emit_splitter(&mut self, id: u32) {
        let Some(n) = self.nodes.get(&id) else {
            return;
        };
        let outs: Vec<(f32, f32, Facing)> = n
            .ports
            .iter()
            .enumerate()
            .filter(|(_, p)| p.kind.is_output() && !p.kind.is_energy())
            .filter_map(|(i, p)| {
                let (px, py) = n.port_world(i)?;
                Some((px, py, port_outward_dir(n, p)))
            })
            .collect();
        if outs.len() < 2 {
            return;
        }
        // Migrate pre-lane backlog (or desynced totals) onto left lane.
        if let Some(n) = self.nodes.get_mut(&id) {
            let ore = n.split_ore[0] as u32 + n.split_ore[1] as u32;
            let ing = n.split_ingot[0] as u32 + n.split_ingot[1] as u32;
            if ore == 0 && n.buf_ore >= 1.0 {
                n.split_ore[0] = n.buf_ore.floor() as u16;
            }
            if ing == 0 && n.buf_ingot >= 1.0 {
                n.split_ingot[0] = n.buf_ingot.floor() as u16;
            }
            n.buf_ore = (n.split_ore[0] + n.split_ore[1]) as f32;
            n.buf_ingot = (n.split_ingot[0] + n.split_ingot[1]) as f32;
        }

        // Each input lane is independent: left stays left, right stays right.
        for lane in 0..2 {
            let prefer = {
                let Some(n) = self.nodes.get(&id) else {
                    return;
                };
                n.split_side[lane].min(1) as usize
            };
            let order = [prefer, 1 - prefer];
            let Some(item) = self.take_splitter_lane(id, lane) else {
                continue;
            };

            let mut placed_side: Option<usize> = None;
            for &side in &order {
                let (px, py, outward) = outs[side];
                let (ox, oy) = facing_delta(outward);
                let ax = px + ox as f32 * TILE_SIZE * 0.4;
                let ay = py + oy as f32 * TILE_SIZE * 0.4;
                let (tx, ty) = world_to_tile(ax, ay);
                if self.belt_tiles.get(&(tx, ty)).is_none() {
                    continue;
                }
                let ok = {
                    let Some(tile) = self.belt_tiles.get_mut(&(tx, ty)) else {
                        continue;
                    };
                    // Never spill to the other lane — empty side stays empty.
                    try_enter(tile, lane, item, 0.0)
                };
                if ok {
                    placed_side = Some(side);
                    break;
                }
            }

            if let Some(side) = placed_side {
                if side == prefer {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.split_side[lane] = if prefer == 0 { 1 } else { 0 };
                    }
                }
            } else {
                self.refund_splitter_lane(id, lane, item);
            }
        }
    }

    fn take_splitter_lane(&mut self, id: u32, lane: usize) -> Option<Item> {
        let n = self.nodes.get_mut(&id)?;
        let lane = lane.min(1);
        if n.split_ore[lane] >= 1 {
            n.split_ore[lane] -= 1;
            n.buf_ore = (n.split_ore[0] + n.split_ore[1]) as f32;
            Some(Item::IronOre)
        } else if n.split_ingot[lane] >= 1 {
            n.split_ingot[lane] -= 1;
            n.buf_ingot = (n.split_ingot[0] + n.split_ingot[1]) as f32;
            Some(Item::IronIngot)
        } else {
            None
        }
    }

    fn refund_splitter_lane(&mut self, id: u32, lane: usize, item: Item) {
        let Some(n) = self.nodes.get_mut(&id) else {
            return;
        };
        let lane = lane.min(1);
        match item {
            Item::IronOre => {
                n.split_ore[lane] = n.split_ore[lane].saturating_add(1);
                n.buf_ore = (n.split_ore[0] + n.split_ore[1]) as f32;
            }
            Item::IronIngot => {
                n.split_ingot[lane] = n.split_ingot[lane].saturating_add(1);
                n.buf_ingot = (n.split_ingot[0] + n.split_ingot[1]) as f32;
            }
            // Splitter only balances iron for now.
            _ => {}
        }
    }

    fn take_output_item(&mut self, id: u32, prefer: Option<Item>) -> Option<Item> {
        let n = self.nodes.get_mut(&id)?;
        if n.kind == BuildingKind::OreNode {
            if n.out_ore >= 1.0 {
                let item = n.mine_item.unwrap_or(Item::IronOre);
                if prefer.is_none() || prefer == Some(item) {
                    n.out_ore -= 1.0;
                    return Some(item);
                }
            }
            return None;
        }
        if matches!(
            n.kind,
            BuildingKind::Smelter
                | BuildingKind::Assembler
                | BuildingKind::Machine
                | BuildingKind::Lab
                | BuildingKind::Box
                | BuildingKind::NexusSite
        ) {
            if n.era_craft {
                if let Some(want) = prefer {
                    if n.stock(want) >= 1.0 {
                        let _ = n.try_take_stock(want, 1.0);
                        return Some(want);
                    }
                }
                if n.craft_recipe != 0 {
                    if let Some(r) = crate::content::content().recipe(n.craft_recipe) {
                        for io in r.all_outputs() {
                            let item = Item::from_u16(io.item);
                            if !item.is_fluid() && n.stock(item) >= 1.0 {
                                let _ = n.try_take_stock(item, 1.0);
                                return Some(item);
                            }
                        }
                    }
                }
                // Any solid stock
                for (i, &v) in n.stocks.iter().enumerate() {
                    if v >= 1.0 {
                        let item = Item::from_u16(i as u16);
                        if !item.is_fluid() {
                            let _ = n.try_take_stock(item, 1.0);
                            return Some(item);
                        }
                    }
                }
                return None;
            }
            if matches!(n.kind, BuildingKind::Smelter | BuildingKind::Assembler) {
                let machine = if n.kind == BuildingKind::Smelter {
                    crate::recipes::MachineKind::Smelt
                } else {
                    crate::recipes::MachineKind::Assemble
                };
                let craft_recipe = n.craft_recipe;
                if let Some(want) = prefer {
                    if crate::recipes::item_is_machine_output(machine, want) && n.stock(want) >= 1.0 {
                        let _ = n.try_take_stock(want, 1.0);
                        return Some(want);
                    }
                }
                if craft_recipe != 0 {
                    if let Some(r) = crate::recipes::recipe_by_id(craft_recipe) {
                        for &(item, _) in r.outputs {
                            if n.stock(item) >= 1.0 {
                                let _ = n.try_take_stock(item, 1.0);
                                return Some(item);
                            }
                        }
                    }
                }
                for r in crate::recipes::recipes_for(machine) {
                    for &(item, _) in r.outputs {
                        if n.stock(item) >= 1.0 {
                            let _ = n.try_take_stock(item, 1.0);
                            return Some(item);
                        }
                    }
                }
            }
            // Generic stock pull for box / machine
            if let Some(want) = prefer {
                if n.stock(want) >= 1.0 {
                    let _ = n.try_take_stock(want, 1.0);
                    return Some(want);
                }
            }
            for (i, &v) in n.stocks.iter().enumerate() {
                if v >= 1.0 {
                    let item = Item::from_u16(i as u16);
                    if !item.is_fluid() {
                        let _ = n.try_take_stock(item, 1.0);
                        return Some(item);
                    }
                }
            }
            return None;
        }
        if let Some(want) = prefer {
            match want {
                Item::IronOre if n.buf_ore >= 1.0 => {
                    n.buf_ore -= 1.0;
                    return Some(Item::IronOre);
                }
                Item::IronIngot if n.buf_ingot >= 1.0 => {
                    n.buf_ingot -= 1.0;
                    return Some(Item::IronIngot);
                }
                Item::IronOre if n.out_ore >= 1.0 => {
                    n.out_ore -= 1.0;
                    return Some(Item::IronOre);
                }
                Item::IronIngot if n.out_ingot >= 1.0 => {
                    n.out_ingot -= 1.0;
                    return Some(Item::IronIngot);
                }
                _ => {}
            }
        }
        if n.buf_ore >= 1.0 {
            n.buf_ore -= 1.0;
            Some(Item::IronOre)
        } else if n.buf_ingot >= 1.0 {
            n.buf_ingot -= 1.0;
            Some(Item::IronIngot)
        } else if n.out_ore >= 1.0 {
            n.out_ore -= 1.0;
            Some(Item::IronOre)
        } else if n.out_ingot >= 1.0 {
            n.out_ingot -= 1.0;
            Some(Item::IronIngot)
        } else {
            None
        }
    }

    fn refund_item(&mut self, id: u32, item: Item) {
        let Some(n) = self.nodes.get_mut(&id) else {
            return;
        };
        match n.kind {
            BuildingKind::OreNode => n.out_ore += 1.0,
            BuildingKind::Smelter
            | BuildingKind::Assembler
            | BuildingKind::Machine
            | BuildingKind::Lab
            | BuildingKind::Box
            | BuildingKind::NexusSite
            | BuildingKind::BallisticTurret => {
                n.add_stock(item, 1.0);
            }
            BuildingKind::Splitter => match item {
                Item::IronOre => {
                    n.split_ore[0] = n.split_ore[0].saturating_add(1);
                    n.buf_ore = (n.split_ore[0] + n.split_ore[1]) as f32;
                }
                Item::IronIngot => {
                    n.split_ingot[0] = n.split_ingot[0].saturating_add(1);
                    n.buf_ingot = (n.split_ingot[0] + n.split_ingot[1]) as f32;
                }
                _ => {}
            },
            _ => match item {
                Item::IronOre => n.buf_ore += 1.0,
                Item::IronIngot => n.buf_ingot += 1.0,
                _ => n.add_stock(item, 1.0),
            },
        }
    }
}

fn port_outward_dir(n: &Node, p: &crate::sim::Port) -> Facing {
    // Prefer the AABB edge the port sits on — off-center ports on a short
    // edge (splitter outs) must face through that edge, not toward center.
    let eps = 1.5;
    let on_w = p.ox <= eps;
    let on_e = p.ox >= n.w() - eps;
    let on_n = p.oy <= eps;
    let on_s = p.oy >= n.h() - eps;
    match (on_w, on_e, on_n, on_s) {
        (true, false, false, false) => Facing::W,
        (false, true, false, false) => Facing::E,
        (false, false, true, false) => Facing::N,
        (false, false, false, true) => Facing::S,
        _ => {
            let dx = p.ox - n.w() * 0.5;
            let dy = p.oy - n.h() * 0.5;
            if dx.abs() >= dy.abs() {
                if dx >= 0.0 {
                    Facing::E
                } else {
                    Facing::W
                }
            } else if dy >= 0.0 {
                Facing::S
            } else {
                Facing::N
            }
        }
    }
}
