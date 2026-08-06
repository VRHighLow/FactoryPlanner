//! Data-driven crafting: items, recipes, and topo helpers for the recipe-tree debugger.
//! Era 0–1 content lives here; later eras append to the same tables.

use crate::sim::Item;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MachineKind {
    Smelt,
    Assemble,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScienceEra {
    Ember = 0,
    Clearfoundry = 1,
}

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub id: u16,
    pub name: &'static str,
    pub machine: MachineKind,
    pub era: ScienceEra,
    pub inputs: &'static [(Item, u32)],
    pub outputs: &'static [(Item, u32)],
    /// Seconds of machine work per craft.
    pub craft_time: f32,
    pub power: f32,
}

pub fn item_label(item: Item) -> &'static str {
    if let Some(c) = crate::content::try_content() {
        // Leak-free: names live in the registry for the process lifetime.
        return c.label(item);
    }
    match item.as_u16() {
        0 => "Ferrite Gas",
        1 => "Ferrite Plate",
        2 => "Conductive Gas",
        3 => "Silicate Gas",
        4 => "Carbon Gas",
        5 => "Hydrocarbon Gas",
        6 => "Conductive Plate",
        7 => "Stone Dust",
        8 => "Carbon Powder",
        9 => "Structural Frame",
        10 => "Conductive Wire",
        11 => "Fastener Set",
        12 => "Silicate Brick",
        13 => "Pipe Section",
        14 => "Machine Frame",
        15 => "Circuit Shard",
        16 => "Belt Link",
        17 => "Pole Kit",
        18 => "Solar Cell",
        19 => "Ammo Casing",
        20 => "Charge Cell",
        21 => "Totem Core",
        22 => "Engineering Data",
        23 => "Chemical Data",
        _ => "Item",
    }
}


pub static RECIPES: &[Recipe] = &[
    Recipe {
        id: 1,
        name: "Smelt Iron Plate",
        machine: MachineKind::Smelt,
        era: ScienceEra::Ember,
        inputs: &[(Item::IronOre, 1)],
        outputs: &[(Item::IronIngot, 1), (Item::Slag, 1)],
        craft_time: 1.0,
        power: 4.0,
    },
    Recipe {
        id: 2,
        name: "Smelt Copper Plate",
        machine: MachineKind::Smelt,
        era: ScienceEra::Ember,
        inputs: &[(Item::CopperOre, 1)],
        outputs: &[(Item::CopperIngot, 1), (Item::Slag, 1)],
        craft_time: 1.0,
        power: 4.0,
    },
    Recipe {
        id: 3,
        name: "Bake Brick",
        machine: MachineKind::Smelt,
        era: ScienceEra::Ember,
        inputs: &[(Item::Stone, 2)],
        outputs: &[(Item::Brick, 1)],
        craft_time: 1.2,
        power: 3.0,
    },
    Recipe {
        id: 4,
        name: "Coke Coal",
        machine: MachineKind::Smelt,
        era: ScienceEra::Ember,
        inputs: &[(Item::Coal, 2)],
        outputs: &[(Item::Coke, 1)],
        craft_time: 1.5,
        power: 3.0,
    },
    Recipe {
        id: 10,
        name: "Iron Gear",
        machine: MachineKind::Assemble,
        era: ScienceEra::Ember,
        inputs: &[(Item::IronIngot, 2)],
        outputs: &[(Item::Gear, 1)],
        craft_time: 0.8,
        power: 2.5,
    },
    Recipe {
        id: 11,
        name: "Copper Wire",
        machine: MachineKind::Assemble,
        era: ScienceEra::Ember,
        inputs: &[(Item::CopperIngot, 1)],
        outputs: &[(Item::Wire, 2)],
        craft_time: 0.5,
        power: 2.0,
    },
    Recipe {
        id: 12,
        name: "Rivet",
        machine: MachineKind::Assemble,
        era: ScienceEra::Ember,
        inputs: &[(Item::IronIngot, 1)],
        outputs: &[(Item::Rivet, 2)],
        craft_time: 0.4,
        power: 2.0,
    },
    Recipe {
        id: 13,
        name: "Pipe Section",
        machine: MachineKind::Assemble,
        era: ScienceEra::Ember,
        inputs: &[(Item::IronIngot, 1)],
        outputs: &[(Item::Pipe, 1)],
        craft_time: 0.6,
        power: 2.0,
    },
    Recipe {
        id: 14,
        name: "Machine Frame",
        machine: MachineKind::Assemble,
        era: ScienceEra::Ember,
        inputs: &[(Item::IronIngot, 4), (Item::Gear, 2), (Item::Rivet, 4)],
        outputs: &[(Item::Frame, 1)],
        craft_time: 2.0,
        power: 5.0,
    },
    Recipe {
        id: 15,
        name: "Circuit Shard",
        machine: MachineKind::Assemble,
        era: ScienceEra::Clearfoundry,
        inputs: &[(Item::CopperIngot, 1), (Item::Wire, 3), (Item::Coke, 1)],
        outputs: &[(Item::CircuitShard, 1)],
        craft_time: 1.5,
        power: 4.0,
    },
    Recipe {
        id: 16,
        name: "Belt Link",
        machine: MachineKind::Assemble,
        era: ScienceEra::Ember,
        inputs: &[(Item::IronIngot, 1), (Item::Gear, 1)],
        outputs: &[(Item::BeltLink, 2)],
        craft_time: 0.7,
        power: 2.0,
    },
    Recipe {
        id: 17,
        name: "Pole Kit",
        machine: MachineKind::Assemble,
        era: ScienceEra::Ember,
        inputs: &[(Item::IronIngot, 2), (Item::Wire, 2)],
        outputs: &[(Item::PoleKit, 1)],
        craft_time: 1.0,
        power: 2.5,
    },
    Recipe {
        id: 18,
        name: "Solar Cell",
        machine: MachineKind::Assemble,
        era: ScienceEra::Ember,
        inputs: &[(Item::CopperIngot, 3), (Item::Wire, 4), (Item::Brick, 2)],
        outputs: &[(Item::SolarCell, 1)],
        craft_time: 2.5,
        power: 3.0,
    },
    Recipe {
        id: 19,
        name: "Shell Casing",
        machine: MachineKind::Assemble,
        era: ScienceEra::Ember,
        inputs: &[(Item::IronIngot, 2)],
        outputs: &[(Item::ShellCasing, 1)],
        craft_time: 0.9,
        power: 2.5,
    },
    Recipe {
        id: 20,
        name: "Charge Cell",
        machine: MachineKind::Assemble,
        era: ScienceEra::Ember,
        inputs: &[(Item::CopperIngot, 2), (Item::Wire, 2), (Item::Coke, 1)],
        outputs: &[(Item::ChargeCell, 1)],
        craft_time: 1.8,
        power: 4.0,
    },
    Recipe {
        id: 21,
        name: "Totem Core",
        machine: MachineKind::Assemble,
        era: ScienceEra::Ember,
        inputs: &[
            (Item::Frame, 1),
            (Item::CircuitShard, 2),
            (Item::ChargeCell, 1),
            (Item::Brick, 4),
        ],
        outputs: &[(Item::TotemCore, 1)],
        craft_time: 4.0,
        power: 8.0,
    },
    Recipe {
        id: 30,
        name: "Ember Science",
        machine: MachineKind::Assemble,
        era: ScienceEra::Ember,
        inputs: &[(Item::IronIngot, 2), (Item::CopperIngot, 1), (Item::Coke, 1)],
        outputs: &[(Item::ScienceRed, 1)],
        craft_time: 2.0,
        power: 3.0,
    },
    Recipe {
        id: 31,
        name: "Circuit Science",
        machine: MachineKind::Assemble,
        era: ScienceEra::Clearfoundry,
        inputs: &[
            (Item::ScienceRed, 1),
            (Item::Gear, 2),
            (Item::CircuitShard, 1),
            (Item::Pipe, 1),
        ],
        outputs: &[(Item::ScienceGreen, 1)],
        craft_time: 3.0,
        power: 5.0,
    },
];

pub fn recipes_for(machine: MachineKind) -> impl Iterator<Item = &'static Recipe> {
    RECIPES.iter().filter(move |r| r.machine == machine)
}

pub fn recipe_by_id(id: u16) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.id == id)
}

pub fn item_is_machine_output(machine: MachineKind, item: Item) -> bool {
    recipes_for(machine).any(|r| r.outputs.iter().any(|(i, _)| *i == item))
}

