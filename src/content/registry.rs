//! Load / intern / validate Era 1 JSON packs.

use super::types::*;
use crate::sim::Item;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::OnceLock;

static CONTENT: OnceLock<ContentRegistry> = OnceLock::new();

pub fn content() -> &'static ContentRegistry {
    CONTENT
        .get()
        .expect("content not initialized — call init_content() at boot")
}

pub fn try_content() -> Option<&'static ContentRegistry> {
    CONTENT.get()
}

pub fn init_content() -> Result<&'static ContentRegistry, String> {
    if let Some(c) = CONTENT.get() {
        return Ok(c);
    }
    let reg = ContentRegistry::load_default()?;
    let _ = CONTENT.set(reg);
    Ok(content())
}

#[derive(Clone, Copy, Debug)]
pub struct ContentStats {
    pub items: usize,
    pub fluids: usize,
    pub recipes: usize,
    pub machines: usize,
    pub technologies: usize,
}

pub struct ContentRegistry {
    pub items: Vec<RuntimeItem>,
    pub recipes: Vec<RuntimeRecipe>,
    pub machines: Vec<RuntimeMachine>,
    pub techs: Vec<RuntimeTech>,
    id_to_item: HashMap<String, u16>,
    id_to_recipe: HashMap<String, u16>,
    id_to_machine: HashMap<String, u16>,
    id_to_tech: HashMap<String, u16>,
    /// machine id → recipe indices craftable in that machine.
    recipes_by_machine: HashMap<String, Vec<u16>>,
    /// category → recipe indices.
    recipes_by_category: HashMap<String, Vec<u16>>,
    pub stats: ContentStats,
    pub validation_errors: Vec<String>,
}

impl ContentRegistry {
    pub fn load_default() -> Result<Self, String> {
        let root = Path::new("assets/data/era1");
        Self::load_from(root)
    }

    pub fn load_from(root: &Path) -> Result<Self, String> {
        let items_raw: Vec<ItemDef> = read_json(&root.join("items.json"))?;
        let fluids_raw: Vec<ItemDef> = read_json(&root.join("fluids.json"))?;
        let recipes_raw: Vec<RecipeDef> = read_json(&root.join("recipes.json"))?;
        let machines_raw: Vec<MachineDef> = read_json(&root.join("machines.json"))?;
        let techs_raw: Vec<TechDef> = read_json(&root.join("technologies.json"))?;

        let mut reg = Self {
            items: Vec::new(),
            recipes: Vec::new(),
            machines: Vec::new(),
            techs: Vec::new(),
            id_to_item: HashMap::new(),
            id_to_recipe: HashMap::new(),
            id_to_machine: HashMap::new(),
            id_to_tech: HashMap::new(),
            recipes_by_machine: HashMap::new(),
            recipes_by_category: HashMap::new(),
            stats: ContentStats {
                items: 0,
                fluids: 0,
                recipes: 0,
                machines: 0,
                technologies: 0,
            },
            validation_errors: Vec::new(),
        };

        // 1) Reserve legacy Item slots 0..23 so save/net stay stable, aliased to Era 1 ids.
        for (legacy, era_id, name, kind) in LEGACY_ALIASES {
            reg.push_item(RuntimeItem {
                id: era_id.to_string(),
                name: name.to_string(),
                kind: *kind,
                era: 1,
                category: "legacy".into(),
                description: String::new(),
                state: None,
                stack_size: 200,
                purity_supported: matches!(
                    kind,
                    ItemKind::Item | ItemKind::Waste
                ) && (era_id.contains("raw_")
                    || era_id.contains("material_")
                    || era_id.contains("waste_")),
                grade_supported: era_id.contains("material_") || era_id.contains("military_"),
                family: "legacy".into(),
                index: 0, // set in push
            });
            // Also map legacy-style names if they differ — already using era id as canonical.
            let _ = legacy; // documented in LEGACY_ALIASES for Item::* constants
        }

        // 2) Pack items + fluids
        for def in items_raw.into_iter().chain(fluids_raw.into_iter()) {
            if reg.id_to_item.contains_key(&def.id) {
                // Update display name from pack if alias already reserved.
                if let Some(&idx) = reg.id_to_item.get(&def.id) {
                    let it = &mut reg.items[idx as usize];
                    if !def.name.is_empty() {
                        it.name = def.name;
                    }
                    it.era = def.era;
                    it.category = def.category;
                    it.description = def.description;
                    it.state = def.state;
                    it.purity_supported = def.purity_supported;
                    it.grade_supported = def.grade_supported;
                    it.stack_size = def.stack_size;
                    it.family = def.family;
                }
                continue;
            }
            let kind = ItemKind::from_str(&def.kind);
            reg.push_item(RuntimeItem {
                id: def.id,
                name: def.name,
                kind,
                era: def.era,
                category: def.category,
                description: def.description,
                state: def.state,
                stack_size: def.stack_size,
                purity_supported: def.purity_supported,
                grade_supported: def.grade_supported,
                family: def.family,
                index: 0,
            });
        }

        // 3) Machines
        for def in machines_raw {
            if reg.id_to_machine.contains_key(&def.id) {
                continue;
            }
            let idx = reg.machines.len() as u16;
            reg.id_to_machine.insert(def.id.clone(), idx);
            let uses_fluids = !def.fluid_ports.is_empty();
            reg.machines.push(RuntimeMachine {
                id: def.id,
                name: def.name,
                category: def.category,
                tier: def.tier,
                description: def.description,
                function: def.function,
                purity_behavior: def.purity_behavior,
                size_tiles: def.size,
                power_kw: def.power_kw,
                power_type: def.power_type,
                recipe_categories: def.recipe_categories,
                fluid_ports: def.fluid_ports,
                technology_unlock: def.technology_unlock,
                placeable: def.placeable,
                index: idx,
                uses_fluids,
            });
        }

        // 4) Recipes — resolve IO, auto-create missing item stubs
        for def in recipes_raw {
            if reg.id_to_recipe.contains_key(&def.id) {
                continue;
            }
            let inputs = reg.resolve_ios(&def.inputs);
            let outputs = reg.resolve_ios(&def.outputs);
            let waste = reg.resolve_ios(&def.waste_outputs);
            let machine_idx = reg.id_to_machine.get(&def.machine).copied();
            let power_kw = def
                .power_consumption
                .values()
                .copied()
                .fold(0.0_f32, f32::max)
                .max(1.0);
            // Convert kW-ish numbers in docs to sim energy units (~old smelter used ~4).
            let power_sim = (power_kw / 50.0).max(1.0);
            let idx = (reg.recipes.len() as u16).saturating_add(1); // 1-based; 0 = idle
            let mut tech_unlock = if def.technology_unlock.is_empty() {
                "era1_tech_basic_recovery".into()
            } else {
                def.technology_unlock
            };
            // Alias typos / older bible IDs → canonical pack techs.
            if tech_unlock == "era1_tech_waste_management" {
                tech_unlock = "era1_tech_waste_recovery".into();
            }
            let rt = RuntimeRecipe {
                id: def.id.clone(),
                name: def.name,
                category: def.category.clone(),
                description: def.description,
                machine: def.machine.clone(),
                machine_idx,
                inputs,
                outputs,
                waste,
                processing_time: def.processing_time.max(0.1),
                power_kw: power_sim,
                purity_effect: def.purity_effect,
                grade_effect: def.grade_effect,
                technology_unlock: tech_unlock,
                extracts: def.extracts,
                index: idx,
            };
            reg.id_to_recipe.insert(def.id, idx);
            reg.recipes_by_machine
                .entry(rt.machine.clone())
                .or_default()
                .push(idx);
            reg.recipes_by_category
                .entry(rt.category.clone())
                .or_default()
                .push(idx);
            if let Some(mi) = machine_idx {
                if !rt.inputs.is_empty() || !rt.outputs.is_empty() {
                    let has_fluid = rt
                        .inputs
                        .iter()
                        .chain(rt.outputs.iter())
                        .chain(rt.waste.iter())
                        .any(|io| reg.items[io.item as usize].kind.is_fluid());
                    if has_fluid {
                        reg.machines[mi as usize].uses_fluids = true;
                    }
                }
            }
            reg.recipes.push(rt);
        }

        // 5) Techs
        for def in techs_raw {
            if reg.id_to_tech.contains_key(&def.id) {
                continue;
            }
            let idx = reg.techs.len() as u16;
            reg.id_to_tech.insert(def.id.clone(), idx);
            let secs = parse_research_seconds(&def.research_time);
            reg.techs.push(RuntimeTech {
                id: def.id,
                name: def.name,
                tier: def.tier,
                era: def.era,
                description: def.description,
                purpose: def.purpose,
                science_cost: def.science_cost,
                prerequisites: def.prerequisites,
                unlocks: def.unlocks,
                research_seconds: secs,
                index: idx,
            });
        }

        reg.validate();
        let fluid_n = reg.items.iter().filter(|i| i.kind.is_fluid()).count();
        reg.stats = ContentStats {
            items: reg.items.len() - fluid_n,
            fluids: fluid_n,
            recipes: reg.recipes.len(),
            machines: reg.machines.len(),
            technologies: reg.techs.len(),
        };
        if let Ok(manifest) = read_json::<Manifest>(&root.join("manifest.json")) {
            let c = &manifest.counts;
            let checks = [
                ("items", c.items, reg.stats.items),
                ("fluids", c.fluids, reg.stats.fluids),
                ("recipes", c.recipes, reg.stats.recipes),
                ("machines", c.machines, reg.stats.machines),
                ("technologies", c.technologies, reg.stats.technologies),
            ];
            for (label, expect, got) in checks {
                // Loaded item count includes legacy aliases — allow >= manifest.
                if got < expect {
                    reg.validation_errors.push(format!(
                        "manifest {label}: pack has {got}, expected at least {expect} ({})",
                        manifest.name
                    ));
                }
            }
            let _ = manifest.era;
        }

        if !reg.validation_errors.is_empty() {
            eprintln!(
                "[era1] content validation: {} issue(s)",
                reg.validation_errors.len()
            );
            for e in reg.validation_errors.iter().take(20) {
                eprintln!("  - {e}");
            }
            if reg.validation_errors.len() > 20 {
                eprintln!("  … +{} more", reg.validation_errors.len() - 20);
            }
        }

        eprintln!(
            "[era1] loaded items={} fluids={} recipes={} machines={} techs={}",
            reg.stats.items,
            reg.stats.fluids,
            reg.stats.recipes,
            reg.stats.machines,
            reg.stats.technologies
        );

        Ok(reg)
    }

    fn push_item(&mut self, mut item: RuntimeItem) {
        let idx = self.items.len() as u16;
        item.index = idx;
        self.id_to_item.insert(item.id.clone(), idx);
        self.items.push(item);
    }

    fn resolve_ios(&mut self, raw: &[IoAmount]) -> Vec<ResolvedIo> {
        let mut out = Vec::with_capacity(raw.len());
        for io in raw {
            let item = self.ensure_item(&io.id);
            out.push(ResolvedIo {
                item,
                amount: io.amount,
            });
        }
        out
    }

    fn ensure_item(&mut self, id: &str) -> u16 {
        if let Some(&idx) = self.id_to_item.get(id) {
            return idx;
        }
        let kind = if id.starts_with("era1_fluid_") {
            ItemKind::Fluid
        } else if id.starts_with("era1_gas_") {
            ItemKind::Gas
        } else if id.starts_with("era1_waste_") {
            ItemKind::Waste
        } else {
            ItemKind::Item
        };
        let name = id
            .rsplit('_')
            .next()
            .unwrap_or(id)
            .replace('_', " ");
        self.push_item(RuntimeItem {
            id: id.to_string(),
            name,
            kind,
            era: 1,
            category: "auto".into(),
            description: String::new(),
            state: None,
            stack_size: 100,
            purity_supported: true,
            grade_supported: false,
            family: "auto".into(),
            index: 0,
        });
        *self.id_to_item.get(id).unwrap()
    }

    fn validate(&mut self) {
        let mut errs = Vec::new();
        let tech_ids: HashSet<_> = self.techs.iter().map(|t| t.id.as_str()).collect();
        for r in &self.recipes {
            if r.machine_idx.is_none() && !r.machine.is_empty() {
                errs.push(format!("recipe {} missing machine {}", r.id, r.machine));
            }
            if !r.technology_unlock.is_empty() && !tech_ids.contains(r.technology_unlock.as_str()) {
                // Soft: tech may be bridge — warn only
                errs.push(format!(
                    "recipe {} tech unlock {} not in pack",
                    r.id, r.technology_unlock
                ));
            }
            for io in r.inputs.iter().chain(r.outputs.iter()).chain(r.waste.iter()) {
                if io.item as usize >= self.items.len() {
                    errs.push(format!("recipe {} bad item index {}", r.id, io.item));
                }
            }
        }
        for t in &self.techs {
            for p in &t.prerequisites {
                if !tech_ids.contains(p.as_str()) {
                    errs.push(format!("tech {} missing prereq {}", t.id, p));
                }
            }
        }
        // Hard fail only on broken item refs — tech warnings are soft in release.
        let hard: Vec<_> = errs
            .iter()
            .filter(|e| e.contains("bad item") || e.contains("missing machine"))
            .cloned()
            .collect();
        self.validation_errors = errs;
        if cfg!(debug_assertions) && !hard.is_empty() {
            // Keep going but loud — full packs have intentional gaps during authoring.
            eprintln!("[era1] DEBUG hard issues: {}", hard.len());
        }
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn item(&self, id: u16) -> Option<&RuntimeItem> {
        self.items.get(id as usize)
    }

    pub fn item_by_str(&self, id: &str) -> Option<&RuntimeItem> {
        self.id_to_item.get(id).and_then(|&i| self.item(i))
    }

    pub fn item_index(&self, id: &str) -> Option<u16> {
        self.id_to_item.get(id).copied()
    }

    pub fn label(&self, item: Item) -> &str {
        self.items
            .get(item.as_u16() as usize)
            .map(|i| i.name.as_str())
            .unwrap_or("?")
    }

    pub fn short(&self, item: Item) -> String {
        self.items
            .get(item.as_u16() as usize)
            .map(|i| {
                if i.name.len() <= 10 {
                    i.name.clone()
                } else if let Some(tail) = i.id.rsplit('_').next() {
                    if !tail.is_empty() && tail.len() <= 12 {
                        tail.to_string()
                    } else {
                        format!("{}…", &i.name[..9])
                    }
                } else {
                    format!("{}…", &i.name[..9.min(i.name.len())])
                }
            })
            .unwrap_or_else(|| "?".into())
    }

    pub fn is_fluid(&self, item: Item) -> bool {
        self.items
            .get(item.as_u16() as usize)
            .map(|i| i.kind.is_fluid())
            .unwrap_or(false)
    }

    pub fn recipe(&self, index: u16) -> Option<&RuntimeRecipe> {
        if index == 0 {
            return None;
        }
        self.recipes.get((index as usize) - 1)
    }

    pub fn recipe_by_str(&self, id: &str) -> Option<&RuntimeRecipe> {
        self.id_to_recipe.get(id).and_then(|&i| self.recipe(i))
    }

    pub fn machine(&self, index: u16) -> Option<&RuntimeMachine> {
        self.machines.get(index as usize)
    }

    pub fn machine_by_str(&self, id: &str) -> Option<&RuntimeMachine> {
        self.id_to_machine.get(id).and_then(|&i| self.machine(i))
    }

    pub fn machine_index(&self, id: &str) -> Option<u16> {
        self.id_to_machine.get(id).copied()
    }

    pub fn tech(&self, index: u16) -> Option<&RuntimeTech> {
        self.techs.get(index as usize)
    }

    pub fn tech_by_str(&self, id: &str) -> Option<&RuntimeTech> {
        self.id_to_tech.get(id).and_then(|&i| self.tech(i))
    }

    pub fn recipes_for_machine(&self, machine_id: &str) -> &[u16] {
        self.recipes_by_machine
            .get(machine_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn recipes_for_categories(&self, cats: &[String]) -> Vec<u16> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for c in cats {
            if let Some(list) = self.recipes_by_category.get(c) {
                for &r in list {
                    if seen.insert(r) {
                        out.push(r);
                    }
                }
            }
        }
        out
    }

    /// Science pack item id for a cost key like `engineering_data`.
    pub fn science_item_for_key(&self, key: &str) -> Option<u16> {
        let id = match key {
            "engineering_data" => "era1_science_engineering_data",
            "chemical_data" => "era1_science_chemical_data",
            "computational_data" => "era1_science_computational_data",
            "defense_data" => "era1_science_defense_data",
            _ => key,
        };
        self.item_by_str(id).map(|i| i.index)
    }

    /// Include every craftable recipe in the full tree viewer.
    pub fn is_viewer_recipe(&self, recipe: &RuntimeRecipe) -> bool {
        !recipe.outputs.is_empty()
            && (!recipe.inputs.is_empty() || recipe.extracts.is_some())
    }

    /// Topological craft depth per **recipe** (0 = extract / no inputs).
    pub fn recipe_depths(&self) -> Vec<(u16, u32)> {
        let n = self.recipes.len();
        if n == 0 {
            return Vec::new();
        }
        // item → recipes that produce it
        let mut producers: HashMap<u16, Vec<u16>> = HashMap::new();
        for r in &self.recipes {
            if !self.is_viewer_recipe(r) {
                continue;
            }
            for io in r.outputs.iter().chain(r.waste.iter()) {
                producers.entry(io.item).or_default().push(r.index);
            }
        }

        let mut depth = vec![99u32; n + 1]; // recipe.index is 1-based
        let mut known = vec![false; n + 1];
        for r in &self.recipes {
            if !self.is_viewer_recipe(r) {
                continue;
            }
            if r.inputs.is_empty() || r.extracts.is_some() {
                depth[r.index as usize] = 0;
                known[r.index as usize] = true;
            }
        }

        for _ in 0..(n + 8).min(96) {
            let mut changed = false;
            for r in &self.recipes {
                if !self.is_viewer_recipe(r) || r.inputs.is_empty() {
                    continue;
                }
                let mut max_in = 0u32;
                let mut ok = true;
                for io in &r.inputs {
                    // Depth of an input item = min depth among recipes that produce it
                    // (0 if no producer = raw world item).
                    let Some(prods) = producers.get(&io.item) else {
                        // Raw / unproduced — treat as depth 0 source.
                        continue;
                    };
                    let mut best = None;
                    for &pid in prods {
                        if known[pid as usize] {
                            best = Some(best.map_or(depth[pid as usize], |b: u32| {
                                b.min(depth[pid as usize])
                            }));
                        }
                    }
                    match best {
                        Some(d) => max_in = max_in.max(d),
                        None => {
                            // Producers exist but unknown yet.
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    continue;
                }
                let d = max_in.saturating_add(1).min(40);
                let i = r.index as usize;
                if !known[i] || d < depth[i] {
                    depth[i] = d;
                    known[i] = true;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        // Anything still unknown — park at end so it still shows.
        for r in &self.recipes {
            if !self.is_viewer_recipe(r) {
                continue;
            }
            let i = r.index as usize;
            if !known[i] {
                depth[i] = 30;
                known[i] = true;
            }
        }

        self.recipes
            .iter()
            .filter(|r| self.is_viewer_recipe(r) && known[r.index as usize])
            .map(|r| (r.index, depth[r.index as usize]))
            .collect()
    }

    /// Edges recipe→recipe when an output (or waste) of `from` feeds an input of `to`.
    pub fn recipe_dependency_edges(&self) -> Vec<(u16, u16)> {
        let mut producers: HashMap<u16, Vec<u16>> = HashMap::new();
        for r in &self.recipes {
            if !self.is_viewer_recipe(r) {
                continue;
            }
            for io in r.outputs.iter().chain(r.waste.iter()) {
                producers.entry(io.item).or_default().push(r.index);
            }
        }
        let mut edges = Vec::new();
        let mut seen = HashSet::new();
        for r in &self.recipes {
            if !self.is_viewer_recipe(r) {
                continue;
            }
            for io in &r.inputs {
                if let Some(prods) = producers.get(&io.item) {
                    for &pid in prods {
                        if pid == r.index {
                            continue;
                        }
                        let key = (pid, r.index);
                        if seen.insert(key) {
                            edges.push(key);
                        }
                    }
                }
            }
        }
        edges
    }

    /// Prefer a "real" craft recipe for an item (not recovery loops / waste side-products).
    pub fn best_recipe_for_output(&self, item: u16) -> Option<&RuntimeRecipe> {
        let mut best: Option<&RuntimeRecipe> = None;
        let mut best_score = i32::MIN;
        for r in &self.recipes {
            if !self.is_viewer_recipe(r) {
                continue;
            }
            let in_out = r.outputs.iter().any(|o| o.item == item);
            let in_waste = r.waste.iter().any(|o| o.item == item);
            if !in_out && !in_waste {
                continue;
            }
            let mut score = 0i32;
            if in_out {
                score += 100;
            }
            if in_waste {
                score -= 40;
            }
            if r.id.contains("bridge_") {
                score -= 20;
            }
            if r.id.contains("recovery_") {
                score -= 50;
            }
            if r.extracts.is_some() {
                score += 30;
            }
            // Prefer shorter input lists slightly (simpler steps).
            score -= r.inputs.len() as i32;
            if score > best_score {
                best_score = score;
                best = Some(r);
            }
        }
        best
    }


    /// Build a Helmod-style nested production plan rooted at `root_item`.
    pub fn production_tree(&self, root_item: u16, max_rows: usize) -> Vec<ProductionTreeRow> {
        let mut rows = Vec::new();
        let mut path = HashSet::new();
        self.walk_production_tree(root_item, 0, true, &[], &mut path, &mut rows, max_rows);
        rows
    }

    fn walk_production_tree(
        &self,
        item: u16,
        depth: u32,
        is_last: bool,
        ancestor_open: &[bool],
        path: &mut HashSet<u16>,
        rows: &mut Vec<ProductionTreeRow>,
        max_rows: usize,
    ) {
        if rows.len() >= max_rows || depth > 24 {
            return;
        }
        if !path.insert(item) {
            // Cycle — show as leaf.
            rows.push(ProductionTreeRow {
                item,
                recipe: None,
                depth,
                is_last,
                ancestor_open: ancestor_open.to_vec(),
                cyclic: true,
            });
            return;
        }
        let recipe = self.best_recipe_for_output(item);
        rows.push(ProductionTreeRow {
            item,
            recipe: recipe.map(|r| r.index),
            depth,
            is_last,
            ancestor_open: ancestor_open.to_vec(),
            cyclic: false,
        });
        if let Some(r) = recipe {
            let mut child_open = ancestor_open.to_vec();
            child_open.push(!is_last);
            let inputs: Vec<u16> = r.inputs.iter().map(|io| io.item).collect();
            let n = inputs.len();
            for (i, child) in inputs.into_iter().enumerate() {
                self.walk_production_tree(
                    child,
                    depth + 1,
                    i + 1 == n,
                    &child_open,
                    path,
                    rows,
                    max_rows,
                );
            }
        }
        path.remove(&item);
    }
}

/// One row in a Helmod-style nested production tree.
#[derive(Clone, Debug)]
pub struct ProductionTreeRow {
    pub item: u16,
    pub recipe: Option<u16>,
    pub depth: u32,
    pub is_last: bool,
    /// For each ancestor depth, whether a vertical continuation line should be drawn.
    pub ancestor_open: Vec<bool>,
    pub cyclic: bool,
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn parse_research_seconds(s: &str) -> f32 {
    let s = s.trim().to_ascii_lowercase();
    if s.is_empty() || s == "0s" {
        return 0.0;
    }
    if let Some(num) = s.strip_suffix('s') {
        return num.parse().unwrap_or(60.0);
    }
    if let Some(num) = s.strip_suffix('m') {
        return num.parse::<f32>().unwrap_or(1.0) * 60.0;
    }
    s.parse().unwrap_or(60.0)
}

/// Slot order must match `Item::*` constants / legacy `as_u8` values.
const LEGACY_ALIASES: &[(&str, &str, &str, ItemKind)] = &[
    ("IronOre", "era1_raw_ferrite_ore", "Ferrite Gas", ItemKind::Item),
    (
        "IronIngot",
        "era1_material_ferrite_plate",
        "Ferrite Plate",
        ItemKind::Item,
    ),
    (
        "CopperOre",
        "era1_raw_conductive_ore",
        "Conductive Gas",
        ItemKind::Item,
    ),
    (
        "Stone",
        "era1_raw_silicate_rock",
        "Silicate Gas",
        ItemKind::Item,
    ),
    (
        "Coal",
        "era1_raw_carbon_deposit",
        "Carbon Gas",
        ItemKind::Item,
    ),
    (
        "CrudeOil",
        "era1_fluid_raw_hydrocarbon",
        "Hydrocarbon Gas",
        ItemKind::Fluid,
    ),
    (
        "CopperIngot",
        "era1_material_conductive_plate",
        "Conductive Plate",
        ItemKind::Item,
    ),
    ("Slag", "era1_waste_stone_dust", "Stone Dust", ItemKind::Waste),
    (
        "Coke",
        "era1_material_carbon_powder",
        "Carbon Powder",
        ItemKind::Item,
    ),
    (
        "Gear",
        "era1_component_structural_frame",
        "Structural Frame",
        ItemKind::Item,
    ),
    (
        "Wire",
        "era1_component_conductive_wire",
        "Conductive Wire",
        ItemKind::Item,
    ),
    (
        "Rivet",
        "era1_component_fastener_set",
        "Fastener Set",
        ItemKind::Item,
    ),
    (
        "Brick",
        "era1_material_silicate_brick",
        "Silicate Brick",
        ItemKind::Item,
    ),
    (
        "Pipe",
        "era1_component_pipe_section",
        "Pipe Section",
        ItemKind::Item,
    ),
    (
        "Frame",
        "era1_component_machine_frame",
        "Machine Frame",
        ItemKind::Item,
    ),
    (
        "CircuitShard",
        "era1_component_circuit_shard",
        "Circuit Shard",
        ItemKind::Item,
    ),
    (
        "BeltLink",
        "era1_component_belt_link",
        "Belt Link",
        ItemKind::Item,
    ),
    (
        "PoleKit",
        "era1_component_pole_kit",
        "Pole Kit",
        ItemKind::Item,
    ),
    (
        "SolarCell",
        "era1_component_solar_cell",
        "Solar Cell",
        ItemKind::Item,
    ),
    (
        "ShellCasing",
        "era1_military_ammo_casing",
        "Ammo Casing",
        ItemKind::Item,
    ),
    (
        "ChargeCell",
        "era1_military_charge_cell",
        "Charge Cell",
        ItemKind::Item,
    ),
    (
        "TotemCore",
        "era1_component_totem_core",
        "Totem Core",
        ItemKind::Item,
    ),
    (
        "ScienceRed",
        "era1_science_engineering_data",
        "Engineering Data",
        ItemKind::Item,
    ),
    (
        "ScienceGreen",
        "era1_science_chemical_data",
        "Chemical Data",
        ItemKind::Item,
    ),
];
