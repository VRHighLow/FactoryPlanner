//! Player inventory + building/belt placement recipes (DSP-like spend-on-place).

use crate::sim::{BuildingKind, Item};

pub const INV_COLS: usize = 10;
pub const INV_ROWS: usize = 4;
pub const INV_SLOTS: usize = INV_COLS * INV_ROWS;
pub const MAX_STACK: u32 = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct InvSlot {
    pub item: Option<Item>,
    pub count: u32,
}

/// Factorio-style stack slots (ore / ingot for now; empty slots for future items).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory {
    pub slots: [InvSlot; INV_SLOTS],
}

impl Default for Inventory {
    fn default() -> Self {
        Self::starter()
    }
}

impl Inventory {
    pub fn empty() -> Self {
        Self {
            slots: [InvSlot::default(); INV_SLOTS],
        }
    }

    /// Generous starter kit so a new factory can bootstrap Era 0.
    pub fn starter() -> Self {
        let mut inv = Self::empty();
        let _ = inv.add(Item::IronOre, 240);
        let _ = inv.add(Item::CopperOre, 120);
        let _ = inv.add(Item::Stone, 80);
        let _ = inv.add(Item::Coal, 80);
        let _ = inv.add(Item::IronIngot, 160);
        let _ = inv.add(Item::CopperIngot, 80);
        let _ = inv.add(Item::Gear, 40);
        let _ = inv.add(Item::Wire, 40);
        let _ = inv.add(Item::Brick, 40);
        let _ = inv.add(Item::Frame, 12);
        let _ = inv.add(Item::BeltLink, 80);
        let _ = inv.add(Item::PoleKit, 20);
        let _ = inv.add(Item::SolarCell, 8);
        let _ = inv.add(Item::CircuitShard, 6);
        let _ = inv.add(Item::ShellCasing, 8);
        let _ = inv.add(Item::ChargeCell, 4);
        let _ = inv.add(Item::TotemCore, 2);
        // Bootstrap science so labs can push past the industrial spine.
        let _ = inv.add(Item::ScienceRed, 40);
        let _ = inv.add(Item::ScienceGreen, 20);
        inv
    }

    pub fn from_totals(ore: u32, ingot: u32) -> Self {
        let mut inv = Self::empty();
        let _ = inv.add(Item::IronOre, ore);
        let _ = inv.add(Item::IronIngot, ingot);
        inv
    }

    pub fn ore(&self) -> u32 {
        self.count(Item::IronOre)
    }

    pub fn ingot(&self) -> u32 {
        self.count(Item::IronIngot)
    }

    pub fn count(&self, item: Item) -> u32 {
        self.slots
            .iter()
            .filter(|s| s.item == Some(item))
            .map(|s| s.count)
            .sum()
    }

    /// Add items, stacking into existing then empty slots. Returns leftover that did not fit.
    pub fn add(&mut self, item: Item, mut n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        for slot in self.slots.iter_mut() {
            if slot.item == Some(item) && slot.count < MAX_STACK {
                let space = MAX_STACK - slot.count;
                let take = n.min(space);
                slot.count += take;
                n -= take;
                if n == 0 {
                    return 0;
                }
            }
        }
        for slot in self.slots.iter_mut() {
            if slot.item.is_none() {
                let take = n.min(MAX_STACK);
                *slot = InvSlot {
                    item: Some(item),
                    count: take,
                };
                n -= take;
                if n == 0 {
                    return 0;
                }
            }
        }
        n
    }

    pub fn can_afford(&self, costs: &[(Item, u32)]) -> bool {
        for &(item, need) in costs {
            if self.count(item) < need {
                return false;
            }
        }
        true
    }

    pub fn try_spend(&mut self, costs: &[(Item, u32)]) -> bool {
        if !self.can_afford(costs) {
            return false;
        }
        for &(item, need) in costs {
            self.remove(item, need);
        }
        true
    }

    fn remove(&mut self, item: Item, mut n: u32) {
        for slot in self.slots.iter_mut().rev() {
            if slot.item != Some(item) || n == 0 {
                continue;
            }
            let take = n.min(slot.count);
            slot.count -= take;
            n -= take;
            if slot.count == 0 {
                *slot = InvSlot::default();
            }
        }
    }

    pub fn refund(&mut self, costs: &[(Item, u32)]) {
        for &(item, n) in costs {
            let _ = self.add(item, n);
        }
    }

    pub fn missing_hint(&self, costs: &[(Item, u32)]) -> String {
        let mut parts = Vec::new();
        for &(item, need) in costs {
            let have = self.count(item);
            if have < need {
                parts.push(format!(
                    "{} {} (have {})",
                    need - have,
                    item_label(item),
                    have
                ));
            }
        }
        if parts.is_empty() {
            "Can't afford".into()
        } else {
            format!("Need {}", parts.join(", "))
        }
    }
}

pub fn item_label(item: Item) -> &'static str {
    crate::recipes::item_label(item)
}

/// Materials spent when placing a ground building (DSP-style).
pub fn building_recipe(kind: BuildingKind) -> &'static [(Item, u32)] {
    match kind {
        BuildingKind::PowerPole => &[(Item::PoleKit, 1)],
        BuildingKind::Solar => &[(Item::SolarCell, 1), (Item::Frame, 1)],
        BuildingKind::OreNode => &[(Item::IronIngot, 8), (Item::Gear, 4)],
        BuildingKind::Smelter => &[(Item::Frame, 1), (Item::Brick, 4)],
        BuildingKind::Assembler | BuildingKind::Machine => {
            &[(Item::Frame, 2), (Item::Gear, 4), (Item::CircuitShard, 1)]
        }
        BuildingKind::Lab => &[(Item::Frame, 2), (Item::CircuitShard, 2), (Item::Wire, 4)],
        BuildingKind::Box => &[(Item::IronIngot, 4)],
        BuildingKind::FluidTank => &[(Item::IronIngot, 6), (Item::Pipe, 4)],
        BuildingKind::Splitter => &[(Item::BeltLink, 2), (Item::Gear, 2)],
        BuildingKind::Pipe => &[(Item::Pipe, 1)],
        BuildingKind::Totem => &[(Item::TotemCore, 1), (Item::Frame, 2)],
        BuildingKind::Wall => &[(Item::Brick, 2)],
        BuildingKind::ReinforcedWall => &[(Item::Brick, 4), (Item::IronIngot, 2)],
        BuildingKind::BallisticTurret => {
            &[(Item::Frame, 1), (Item::ShellCasing, 4), (Item::Gear, 2)]
        }
        BuildingKind::Turret => &[(Item::Frame, 1), (Item::ShellCasing, 2), (Item::ChargeCell, 1)],
        BuildingKind::LaserTurret => {
            &[(Item::Frame, 2), (Item::CircuitShard, 2), (Item::ChargeCell, 2)]
        }
        BuildingKind::NexusSite => &[(Item::Frame, 8), (Item::Gear, 8), (Item::CircuitShard, 4)],
        BuildingKind::Nexus => &[(Item::Frame, 20), (Item::TotemCore, 4)],
        // Tools — no inventory cost (wire connects; belts cost per tile; debug free).
        BuildingKind::PowerWire
        | BuildingKind::Conveyor
        | BuildingKind::SpawnAssault
        | BuildingKind::SpawnHunter
        | BuildingKind::SpawnSaboteur
        | BuildingKind::SpawnFogcaller
        | BuildingKind::SpawnNest => &[],
    }
}

pub fn belt_recipe() -> &'static [(Item, u32)] {
    &[(Item::BeltLink, 1)]
}
