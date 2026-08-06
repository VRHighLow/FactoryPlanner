//! Living resource vents — Factorio-oil-style yield %, organic underground fields.
//!
//! - Impassable surface crack + faint gas ooze when revealed; field outline only while placing a drill.
//! - Yield is a raw percent (can be 80%…1800%+). Mining depletes toward ~20% of peak.
//! - Farther vents start richer. Storm clear opens them; fog chokes throughput.
//! - Multiple drills share pressure (soft √ split).

use crate::sim::Item;
use macroquad::prelude::Color;
use macroquad::rand::gen_range;

pub const VEIN_OUTLINE_VERTS: usize = 40;

/// Patch resources players expand for. Five core types (uranium/water later).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ResourceKind {
    Iron,
    Copper,
    Stone,
    Coal,
    Oil,
}

impl ResourceKind {
    pub fn item(self) -> Item {
        match self {
            // Era 1 remap: Iron→ferrite, Copper→conductive, Coal→carbon, Stone→silicate, Oil→hydrocarbon.
            Self::Iron => Item::IronOre,
            Self::Copper => Item::CopperOre,
            Self::Stone => Item::Stone,
            Self::Coal => Item::Coal,
            Self::Oil => Item::CrudeOil,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Iron => "Ferrite Gas Vent",
            Self::Copper => "Conductive Gas Vent",
            Self::Stone => "Silicate Gas Vent",
            Self::Coal => "Carbon Gas Vent",
            Self::Oil => "Hydrocarbon Gas Vent",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Self::Iron => "Ferrite Gas",
            Self::Copper => "Conductive Gas",
            Self::Stone => "Silicate Gas",
            Self::Coal => "Carbon Gas",
            Self::Oil => "Hydrocarbon Gas",
        }
    }

    /// Base vein purity before noise (0..100).
    pub fn base_purity(self) -> f32 {
        match self {
            Self::Iron => 55.0,
            Self::Copper => 48.0,
            Self::Stone => 70.0,
            Self::Coal => 60.0,
            Self::Oil => 40.0,
        }
    }

    pub fn as_u8(self) -> u8 {
        match self {
            Self::Iron => 0,
            Self::Copper => 1,
            Self::Stone => 2,
            Self::Coal => 3,
            Self::Oil => 4,
        }
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Iron),
            1 => Some(Self::Copper),
            2 => Some(Self::Stone),
            3 => Some(Self::Coal),
            4 => Some(Self::Oil),
            _ => None,
        }
    }

    pub fn color(self) -> Color {
        match self {
            // Soft gas tints (used for field outline + ooze).
            Self::Iron => Color::from_rgba(110, 190, 210, 255),
            Self::Copper => Color::from_rgba(220, 150, 80, 255),
            Self::Stone => Color::from_rgba(180, 175, 165, 255),
            Self::Coal => Color::from_rgba(120, 100, 150, 255),
            Self::Oil => Color::from_rgba(70, 190, 140, 255),
        }
    }

    /// Pixel gas ooze tint (RGB + suggested peak alpha).
    pub fn gas_rgba(self) -> (u8, u8, u8, u8) {
        match self {
            // Slightly punchier hues so faint smoke still IDs the vent type.
            Self::Iron => (100, 210, 235, 90),
            Self::Copper => (255, 150, 70, 85),
            Self::Stone => (210, 205, 195, 70),
            Self::Coal => (160, 115, 220, 80),
            Self::Oil => (60, 230, 160, 85),
        }
    }
}

/// A living underground vein. Drills share its yield pressure.
#[derive(Clone, Debug)]
pub struct Vein {
    pub id: u32,
    pub kind: ResourceKind,
    pub x: f32,
    pub y: f32,
    /// Mean field radius (outline wobbles around this).
    pub radius: f32,
    /// Current yield percent (Factorio oil style — can be ≫100).
    pub yield_pct: f32,
    /// Peak / starting yield percent.
    pub yield_max: f32,
    /// Deterministic shape + marker seed.
    pub seed: u32,
    /// Updated each tick: 1 = fully clear, ~0.12 = choked by storm.
    pub clear_factor: f32,
    /// Updated each tick: drills currently tapping this vein.
    pub taps: u32,
    /// Ore purity 0..100 — dirty vs clean lines.
    pub purity: f32,
    /// Geological stability 0..1 — low stability hurts sustained yield.
    pub stability: f32,
}

impl Vein {
    /// Floor yield (~20% of peak, Factorio oil style).
    pub fn yield_floor(&self) -> f32 {
        (self.yield_max * 0.20).max(20.0)
    }

    /// Display / UI — current raw yield percent.
    pub fn yield_display(&self) -> f32 {
        self.yield_pct.max(0.0)
    }

    /// How depleted toward the floor (1 = fresh, 0 = at floor).
    pub fn freshness01(&self) -> f32 {
        let floor = self.yield_floor();
        let span = (self.yield_max - floor).max(1.0);
        ((self.yield_pct - floor) / span).clamp(0.0, 1.0)
    }

    /// Organic radius along a world-space angle — lumpy cavern, not a circle.
    pub fn radius_at_angle(&self, ang: f32) -> f32 {
        self.radius * self.shape_mul(ang)
    }

    /// Deterministic crack art rotation (radians) — shared by draw + collision.
    pub fn crack_rotation(seed: u32) -> f32 {
        let h = seed.wrapping_mul(2654435761).wrapping_add(1013904223);
        ((h >> 8) as f32 / 16_777_215.0) * std::f32::consts::TAU
    }

    /// Shape multiplier around the rim (ellipse + lobes + notches). Independent of size.
    fn shape_mul(&self, ang: f32) -> f32 {
        let s = self.seed as f32;
        let u = ang / std::f32::consts::TAU;

        // Stretched ellipse so the field isn't radially symmetric.
        let twist = s * 0.0013;
        let local = ang - twist;
        let major = 1.0 + ((s * 0.011).sin()) * 0.42;
        let minor = 1.0 - ((s * 0.011).sin()) * 0.28;
        let ellipse = 1.0
            / ((local.cos() / major).powi(2) + (local.sin() / minor).powi(2))
                .sqrt()
                .max(0.35);

        // Low-frequency lobes (big bites taken out of the rim).
        let lobes = (u * 2.0 + s * 0.0021).sin() * 0.20
            + (u * 3.0 - s * 0.0034).sin() * 0.26
            + (u * 4.0 + s * 0.0016).cos() * 0.14;

        // Higher-frequency lumps / fjords.
        let lumps = (u * 6.0 - s * 0.0042).sin() * 0.14
            + (u * 9.0 + s * 0.0028).cos() * 0.10
            + (u * 13.0 - s * 0.0019).sin() * 0.07;

        // Occasional sharp notches.
        let notch = ((u * 5.0 + s * 0.006).sin()).abs().powf(2.4) * -0.22
            + ((u * 7.0 - s * 0.005).cos()).abs().powf(3.0) * -0.12;

        (ellipse * (0.78 + lobes + lumps) + notch).clamp(0.30, 1.72)
    }

    pub fn contains_point(&self, wx: f32, wy: f32) -> bool {
        let dx = wx - self.x;
        let dy = wy - self.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 1e-3 {
            return true;
        }
        let ang = dy.atan2(dx);
        dist <= self.radius_at_angle(ang)
    }

    /// True if the building footprint overlaps the organic field.
    pub fn overlaps_rect(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        // Sample center + corners + edge mids — enough for drill footprints.
        let pts = [
            (x + w * 0.5, y + h * 0.5),
            (x, y),
            (x + w, y),
            (x, y + h),
            (x + w, y + h),
            (x + w * 0.5, y),
            (x + w * 0.5, y + h),
            (x, y + h * 0.5),
            (x + w, y + h * 0.5),
        ];
        pts.iter().any(|&(px, py)| self.contains_point(px, py))
    }

    /// World-space outline polygon (organic cavern rim).
    pub fn outline_world(&self) -> [(f32, f32); VEIN_OUTLINE_VERTS] {
        let mut pts = [(0.0_f32, 0.0_f32); VEIN_OUTLINE_VERTS];
        for i in 0..VEIN_OUTLINE_VERTS {
            let ang = i as f32 / VEIN_OUTLINE_VERTS as f32 * std::f32::consts::TAU;
            let rr = self.radius_at_angle(ang);
            pts[i] = (self.x + ang.cos() * rr, self.y + ang.sin() * rr);
        }
        pts
    }

    /// Visual size of the surface crack marker (not the underground field).
    pub fn crack_world_size(&self) -> f32 {
        // Slightly larger vents on richer fields, but capped so cracks stay readable.
        (150.0 + self.radius * 0.06).clamp(170.0, 300.0)
    }

    /// Items/sec one drill gets when sharing this vein.
    pub fn rate_per_tap(&self, base_rate: f32) -> f32 {
        let taps = self.taps.max(1) as f32;
        let share = 1.0 / taps.sqrt();
        let yield_mult = (self.yield_pct / 100.0).max(0.05);
        let stab = self.stability.clamp(0.25, 1.0);
        base_rate * yield_mult * self.clear_factor * share * stab
    }

    /// Produce `amount` items and decay yield toward the floor.
    pub fn extract(&mut self, amount: f32) -> f32 {
        if amount <= 0.0 {
            return 0.0;
        }
        let floor = self.yield_floor();
        // Higher wells bleed yield faster (Factorio-ish feel).
        let decay = amount * (0.008 + self.yield_pct * 0.00002);
        self.yield_pct = (self.yield_pct - decay).max(floor);
        amount
    }
}

struct RingSpec {
    kind: ResourceKind,
    count: usize,
    dist_lo: f32,
    dist_hi: f32,
    radius_lo: f32,
    radius_hi: f32,
    /// Starting yield % band (farther rings use higher bands).
    yield_lo: f32,
    yield_hi: f32,
}

/// Seed veins around the storm pocket. Iron + coal always inside the clear zone.
pub fn seed_veins(cx: f32, cy: f32, safe_r: f32) -> Vec<Vein> {
    let hard = (safe_r * 0.72).max(400.0);
    let mut placed: Vec<(f32, f32, f32)> = Vec::new();
    let mut out: Vec<Vein> = Vec::new();
    let mut next_id = 1u32;

    let mut push = |kind: ResourceKind,
                    dist_lo: f32,
                    dist_hi: f32,
                    rad_lo: f32,
                    rad_hi: f32,
                    yield_lo: f32,
                    yield_hi: f32,
                    min_sep: f32| {
        let radius = gen_range(rad_lo, rad_hi);
        let yield_max = gen_range(yield_lo, yield_hi);
        let sep = min_sep.max(radius * 1.15);
        for _ in 0..56 {
            let ang = gen_range(0.0, std::f32::consts::TAU);
            let dist = gen_range(dist_lo.min(dist_hi), dist_lo.max(dist_hi));
            let x = cx + ang.cos() * dist;
            let y = cy + ang.sin() * dist;
            if overlaps(&placed, x, y, radius, sep) {
                continue;
            }
            placed.push((x, y, radius));
            let id = next_id;
            next_id += 1;
            let purity = (kind.base_purity() + gen_range(-12.0, 18.0)).clamp(15.0, 95.0);
            let stability = gen_range(0.55, 1.0);
            out.push(Vein {
                id,
                kind,
                x,
                y,
                radius,
                yield_pct: yield_max,
                yield_max,
                seed: hash_u32(x.to_bits(), y.to_bits(), id),
                clear_factor: 1.0,
                taps: 0,
                purity,
                stability,
            });
            return;
        }
        let ang = gen_range(0.0, std::f32::consts::TAU);
        let dist = gen_range(dist_lo.min(dist_hi), dist_lo.max(dist_hi));
        let x = cx + ang.cos() * dist;
        let y = cy + ang.sin() * dist;
        placed.push((x, y, radius));
        let id = next_id;
        next_id += 1;
        let purity = (kind.base_purity() + gen_range(-12.0, 18.0)).clamp(15.0, 95.0);
        let stability = gen_range(0.55, 1.0);
        out.push(Vein {
            id,
            kind,
            x,
            y,
            radius,
            yield_pct: yield_max,
            yield_max,
            seed: hash_u32(x.to_bits(), y.to_bits(), id),
            clear_factor: 1.0,
            taps: 0,
            purity,
            stability,
        });
    };

    // Guaranteed starters inside the clear pocket — fully random pose/size/yield.
    // Only constraint: at least one iron + one coal somewhere in the hard clear.
    let starter_kinds = [ResourceKind::Iron, ResourceKind::Coal];
    for &kind in &starter_kinds {
        let radius = gen_range(900.0, 1600.0);
        let yield_max = gen_range(60.0, 280.0);
        // Anywhere in the pocket except right on spawn (keep a small keep-out).
        let dist_lo = hard * gen_range(0.12, 0.28);
        let dist_hi = hard * gen_range(0.55, 0.72);
        push(
            kind,
            dist_lo.min(dist_hi),
            dist_lo.max(dist_hi),
            radius * 0.92, // push() re-rolls radius inside [lo,hi]
            radius * 1.08,
            yield_max * 0.85,
            yield_max * 1.15,
            hard * gen_range(0.35, 0.55),
        );
    }
    // 0–2 bonus starter patches (iron / coal / stone) for map variety.
    let bonus = (gen_range(0.0_f32, 3.0_f32).floor() as i32).clamp(0, 2);
    for _ in 0..bonus {
        let kind = match (gen_range(0.0_f32, 3.0_f32).floor() as i32).clamp(0, 2) {
            0 => ResourceKind::Iron,
            1 => ResourceKind::Coal,
            _ => ResourceKind::Stone,
        };
        let radius = gen_range(800.0, 1400.0);
        let yield_max = gen_range(70.0, 320.0);
        let dist_lo = hard * gen_range(0.20, 0.40);
        let dist_hi = hard * gen_range(0.58, 0.78);
        push(
            kind,
            dist_lo.min(dist_hi),
            dist_lo.max(dist_hi),
            radius * 0.9,
            radius * 1.12,
            yield_max * 0.8,
            yield_max * 1.2,
            hard * gen_range(0.30, 0.50),
        );
    }

    let rings: &[RingSpec] = &[
        RingSpec {
            kind: ResourceKind::Copper,
            count: 2,
            dist_lo: 1.1,
            dist_hi: 1.9,
            radius_lo: 1000.0,
            radius_hi: 1400.0,
            yield_lo: 180.0,
            yield_hi: 350.0,
        },
        RingSpec {
            kind: ResourceKind::Copper,
            count: 2,
            dist_lo: 2.1,
            dist_hi: 3.3,
            radius_lo: 1200.0,
            radius_hi: 1700.0,
            yield_lo: 400.0,
            yield_hi: 900.0,
        },
        RingSpec {
            kind: ResourceKind::Stone,
            count: 2,
            dist_lo: 1.2,
            dist_hi: 2.2,
            radius_lo: 1050.0,
            radius_hi: 1500.0,
            yield_lo: 160.0,
            yield_hi: 320.0,
        },
        RingSpec {
            kind: ResourceKind::Stone,
            count: 2,
            dist_lo: 2.3,
            dist_hi: 3.5,
            radius_lo: 1250.0,
            radius_hi: 1800.0,
            yield_lo: 380.0,
            yield_hi: 850.0,
        },
        RingSpec {
            kind: ResourceKind::Iron,
            count: 3,
            dist_lo: 1.7,
            dist_hi: 2.9,
            radius_lo: 1300.0,
            radius_hi: 1900.0,
            yield_lo: 350.0,
            yield_hi: 800.0,
        },
        RingSpec {
            kind: ResourceKind::Iron,
            count: 2,
            dist_lo: 2.9,
            dist_hi: 4.1,
            radius_lo: 1500.0,
            radius_hi: 2200.0,
            yield_lo: 700.0,
            yield_hi: 1600.0,
        },
        RingSpec {
            kind: ResourceKind::Coal,
            count: 2,
            dist_lo: 1.8,
            dist_hi: 3.0,
            radius_lo: 1100.0,
            radius_hi: 1600.0,
            yield_lo: 300.0,
            yield_hi: 700.0,
        },
        RingSpec {
            kind: ResourceKind::Coal,
            count: 2,
            dist_lo: 3.0,
            dist_hi: 4.2,
            radius_lo: 1250.0,
            radius_hi: 1850.0,
            yield_lo: 650.0,
            yield_hi: 1400.0,
        },
        RingSpec {
            kind: ResourceKind::Oil,
            count: 2,
            dist_lo: 2.5,
            dist_hi: 3.7,
            radius_lo: 900.0,
            radius_hi: 1300.0,
            yield_lo: 500.0,
            yield_hi: 1200.0,
        },
        RingSpec {
            kind: ResourceKind::Oil,
            count: 2,
            dist_lo: 3.5,
            dist_hi: 4.7,
            radius_lo: 1000.0,
            radius_hi: 1450.0,
            yield_lo: 900.0,
            yield_hi: 2000.0,
        },
    ];

    for spec in rings {
        for _ in 0..spec.count {
            push(
                spec.kind,
                hard * spec.dist_lo,
                hard * spec.dist_hi,
                spec.radius_lo,
                spec.radius_hi,
                spec.yield_lo,
                spec.yield_hi,
                hard * 0.42,
            );
        }
    }

    out
}

/// Build a vein from a legacy circular deposit (save migration).
pub fn vein_from_legacy(
    id: u32,
    kind: ResourceKind,
    x: f32,
    y: f32,
    radius: f32,
    amount: f32,
) -> Vein {
    // Map old reserve ballpark → a plausible starting yield %.
    let yield_max = (80.0 + (amount / 200.0).clamp(0.0, 1500.0)).clamp(80.0, 1800.0);
    Vein {
        id,
        kind,
        x,
        y,
        radius: radius.max(700.0),
        yield_pct: yield_max,
        yield_max,
        seed: hash_u32(x.to_bits(), y.to_bits(), id),
        clear_factor: 1.0,
        taps: 0,
        purity: kind.base_purity(),
        stability: 0.85,
    }
}

fn overlaps(existing: &[(f32, f32, f32)], x: f32, y: f32, radius: f32, min_sep: f32) -> bool {
    for &(ex, ey, er) in existing {
        let dx = ex - x;
        let dy = ey - y;
        let need = (er + radius) * 0.5 + min_sep * 0.4;
        if dx * dx + dy * dy < need * need {
            return true;
        }
    }
    false
}

fn hash_u32(a: u32, b: u32, c: u32) -> u32 {
    let mut n = a
        .wrapping_mul(374761393)
        .wrapping_add(b.wrapping_mul(668265263))
        .wrapping_add(c.wrapping_mul(1274126177));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}
