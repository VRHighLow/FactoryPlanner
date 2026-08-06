//! Research state — unlocked techs, science buffers, lab progress.

use super::registry::content;
use crate::sim::Item;
use std::collections::{HashMap, HashSet};

/// Player / world technology progression for Era 1.
#[derive(Clone, Debug)]
pub struct TechState {
    pub researched: HashSet<String>,
    /// Science packs stored at the colony (labs pull from here or local stocks).
    pub science: HashMap<String, f32>,
    /// Active research tech id.
    pub active: Option<String>,
    /// Seconds invested into `active`.
    pub progress_t: f32,
    /// Era 2 unlocked via Nexus / era_transition tech.
    pub era2_unlocked: bool,
    /// Nexus construction progress 0..1.
    pub nexus_progress: f32,
    pub nexus_complete: bool,
}

impl Default for TechState {
    fn default() -> Self {
        let mut researched = HashSet::new();
        // T001 always. Industrial spine unlocked so Phase 1–2 factory loops are playable;
        // military / laser / nexus remain gated for Phase 3–6 progression.
        for id in [
            "era1_tech_basic_recovery",
            "era1_tech_basic_extraction",
            "era1_tech_material_processing",
            "era1_tech_basic_metallurgy",
            "era1_tech_fluid_engineering",
            "era1_tech_research_infrastructure",
            "era1_tech_industrial_automation",
            "era1_tech_structural_engineering",
            "era1_tech_waste_recovery",
        ] {
            researched.insert(id.into());
        }
        Self {
            researched,
            science: HashMap::new(),
            active: None,
            progress_t: 0.0,
            era2_unlocked: false,
            nexus_progress: 0.0,
            nexus_complete: false,
        }
    }
}

impl TechState {
    pub fn is_researched(&self, tech_id: &str) -> bool {
        self.researched.contains(tech_id)
    }

    pub fn recipe_unlocked(&self, tech_unlock: &str) -> bool {
        if tech_unlock.is_empty() || tech_unlock == "era1_tech_basic_recovery" {
            return true;
        }
        // Debug uncapped mode: if basic extraction researched, allow all for playtest? No —
        // gate properly. Starter unlocks T001 only.
        self.is_researched(tech_unlock)
    }

    pub fn machine_unlocked(&self, tech_unlock: &str) -> bool {
        self.recipe_unlocked(tech_unlock)
    }

    pub fn can_start(&self, tech_id: &str) -> bool {
        if self.is_researched(tech_id) {
            return false;
        }
        let Some(t) = content().tech_by_str(tech_id) else {
            return false;
        };
        t.prerequisites.iter().all(|p| self.is_researched(p))
    }

    pub fn start_research(&mut self, tech_id: &str) -> bool {
        if !self.can_start(tech_id) {
            return false;
        }
        self.active = Some(tech_id.to_string());
        self.progress_t = 0.0;
        true
    }

    /// Consume science from a lab node's stocks (item indices) and advance research.
    /// Returns true if a tech completed this tick.
    pub fn tick_lab(
        &mut self,
        dt: f32,
        try_take: &mut dyn FnMut(Item, f32) -> bool,
    ) -> Option<String> {
        let Some(tid) = self.active.clone() else {
            return None;
        };
        let Some(tech) = content().tech_by_str(&tid) else {
            self.active = None;
            return None;
        };
        if tech.research_seconds <= 0.0 {
            self.researched.insert(tid.clone());
            self.active = None;
            self.progress_t = 0.0;
            self.on_researched(&tid);
            return Some(tid);
        }

        // Drain a proportional share of science cost over research_time.
        // Prefer lab stocks (`try_take`); fall back to colony science bank.
        let frac = dt / tech.research_seconds;
        for (key, &need) in &tech.science_cost {
            let amt = need as f32 * frac;
            if amt <= 0.0 {
                continue;
            }
            let Some(item_idx) = content().science_item_for_key(key) else {
                continue;
            };
            if try_take(Item::from_u16(item_idx), amt) {
                continue;
            }
            let bank = self.science.entry(key.clone()).or_insert(0.0);
            if *bank + 1e-4 < amt {
                return None; // stalled — missing packs
            }
            *bank -= amt;
        }

        self.progress_t += dt;
        if self.progress_t >= tech.research_seconds {
            self.researched.insert(tid.clone());
            self.active = None;
            self.progress_t = 0.0;
            self.on_researched(&tid);
            return Some(tid);
        }
        None
    }

    fn on_researched(&mut self, tech_id: &str) {
        if tech_id == "era1_tech_era_transition" {
            self.era2_unlocked = true;
        }
    }

    pub fn research_progress01(&self) -> f32 {
        let Some(tid) = self.active.as_deref() else {
            return 0.0;
        };
        let Some(t) = content().tech_by_str(tid) else {
            return 0.0;
        };
        if t.research_seconds <= 0.0 {
            return 1.0;
        }
        (self.progress_t / t.research_seconds).clamp(0.0, 1.0)
    }

    /// Deposit science packs into the colony bank (labs can also feed via `try_take`).
    pub fn deposit_science(&mut self, key: &str, amt: f32) {
        if amt <= 0.0 {
            return;
        }
        *self.science.entry(key.to_string()).or_insert(0.0) += amt;
    }

    /// Debug: unlock a tech and all its prerequisites.
    pub fn debug_unlock(&mut self, tech_id: &str) {
        if let Some(t) = content().tech_by_str(tech_id) {
            let prereqs = t.prerequisites.clone();
            for p in prereqs {
                self.debug_unlock(&p);
            }
        }
        self.researched.insert(tech_id.to_string());
        self.on_researched(tech_id);
    }

    /// Debug: unlock everything for playtests.
    pub fn debug_unlock_all(&mut self) {
        for t in &content().techs {
            self.researched.insert(t.id.clone());
        }
        self.era2_unlocked = true;
    }
}
