//! Serde shapes for `assets/data/era1/*.json`.

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub era: u8,
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub category: String,
    #[serde(default = "default_stack")]
    pub stack_size: u32,
    #[serde(default)]
    pub purity_supported: bool,
    #[serde(default)]
    pub grade_supported: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub state: Option<String>,
}

fn default_stack() -> u32 {
    100
}

#[derive(Clone, Debug, Deserialize)]
pub struct IoAmount {
    pub id: String,
    pub amount: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RecipeDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub machine: String,
    #[serde(default)]
    pub inputs: Vec<IoAmount>,
    #[serde(default)]
    pub outputs: Vec<IoAmount>,
    #[serde(default)]
    pub waste_outputs: Vec<IoAmount>,
    #[serde(default = "default_time")]
    pub processing_time: f32,
    #[serde(default)]
    pub power_consumption: HashMap<String, f32>,
    #[serde(default)]
    pub purity_effect: f32,
    #[serde(default)]
    pub grade_effect: String,
    #[serde(default)]
    pub technology_unlock: String,
    #[serde(default)]
    pub description: String,
    /// Optional mining recipe → vein family key.
    #[serde(default)]
    pub extracts: Option<String>,
}

fn default_time() -> f32 {
    1.0
}

#[derive(Clone, Debug, Deserialize)]
pub struct MachineDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tier: u8,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub function: String,
    #[serde(default = "default_size")]
    pub size: [u32; 2],
    #[serde(default = "default_power")]
    pub power_kw: f32,
    #[serde(default)]
    pub power_type: String,
    #[serde(default)]
    pub recipe_categories: Vec<String>,
    #[serde(default)]
    pub fluid_ports: Vec<String>,
    #[serde(default)]
    pub technology_unlock: String,
    #[serde(default)]
    pub purity_behavior: String,
    #[serde(default)]
    pub placeable: bool,
}

fn default_size() -> [u32; 2] {
    [3, 3]
}
fn default_power() -> f32 {
    100.0
}

#[derive(Clone, Debug, Deserialize)]
pub struct TechDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tier: u8,
    #[serde(default = "default_era")]
    pub era: u8,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub science_cost: HashMap<String, u32>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub unlocks: Vec<String>,
    #[serde(default)]
    pub research_time: String,
}

fn default_era() -> u8 {
    1
}

#[derive(Clone, Debug, Deserialize)]
pub struct Manifest {
    pub era: u8,
    pub name: String,
    pub counts: ManifestCounts,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ManifestCounts {
    pub items: usize,
    pub fluids: usize,
    pub recipes: usize,
    pub machines: usize,
    pub technologies: usize,
}

/// Resolved IO using interned item indices.
#[derive(Clone, Copy, Debug)]
pub struct ResolvedIo {
    pub item: u16,
    pub amount: f32,
}

/// Runtime recipe with resolved item indices.
#[derive(Clone, Debug)]
pub struct RuntimeRecipe {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub machine: String,
    pub machine_idx: Option<u16>,
    pub inputs: Vec<ResolvedIo>,
    pub outputs: Vec<ResolvedIo>,
    pub waste: Vec<ResolvedIo>,
    pub processing_time: f32,
    /// Approximate electrical kW draw for sim energy.
    pub power_kw: f32,
    pub purity_effect: f32,
    pub grade_effect: String,
    pub technology_unlock: String,
    pub extracts: Option<String>,
    /// Dense recipe index (for Node.craft_recipe; 0 = idle).
    pub index: u16,
}

impl RuntimeRecipe {
    pub fn all_outputs(&self) -> impl Iterator<Item = &ResolvedIo> {
        self.outputs.iter().chain(self.waste.iter())
    }
}

/// Runtime machine.
#[derive(Clone, Debug)]
pub struct RuntimeMachine {
    pub id: String,
    pub name: String,
    pub category: String,
    pub tier: u8,
    pub description: String,
    pub function: String,
    pub purity_behavior: String,
    pub size_tiles: [u32; 2],
    pub power_kw: f32,
    pub power_type: String,
    pub recipe_categories: Vec<String>,
    pub fluid_ports: Vec<String>,
    pub technology_unlock: String,
    pub placeable: bool,
    pub index: u16,
    /// True if any recipe has fluid IO or machine declares fluid ports.
    pub uses_fluids: bool,
}

#[derive(Clone, Debug)]
pub struct RuntimeItem {
    pub id: String,
    pub name: String,
    pub kind: ItemKind,
    pub era: u8,
    pub category: String,
    pub description: String,
    pub state: Option<String>,
    pub stack_size: u32,
    pub purity_supported: bool,
    pub grade_supported: bool,
    pub family: String,
    pub index: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemKind {
    Item,
    Fluid,
    Gas,
    Waste,
}

impl ItemKind {
    pub fn from_str(s: &str) -> Self {
        match s {
            "fluid" => Self::Fluid,
            "gas" => Self::Gas,
            "waste" => Self::Waste,
            _ => Self::Item,
        }
    }

    pub fn is_fluid(self) -> bool {
        matches!(self, Self::Fluid | Self::Gas)
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeTech {
    pub id: String,
    pub name: String,
    pub tier: u8,
    pub era: u8,
    pub description: String,
    pub purpose: String,
    pub science_cost: HashMap<String, u32>,
    pub prerequisites: Vec<String>,
    pub unlocks: Vec<String>,
    pub research_seconds: f32,
    pub index: u16,
}
