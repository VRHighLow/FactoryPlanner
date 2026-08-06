//! Runtime art loaded from `assets/` (WebP/PNG with alpha).

use macroquad::prelude::*;

pub const CRACK_VARIANTS: usize = 3;

/// CPU alpha mask for precise crack collision (matches drawn silhouette).
#[derive(Clone, Debug)]
pub struct CrackMask {
    pub w: u32,
    pub h: u32,
    /// Row-major alpha 0..255.
    pub alpha: Vec<u8>,
}

impl CrackMask {
    fn solid_fallback() -> Self {
        Self {
            w: 2,
            h: 2,
            alpha: vec![255, 255, 255, 255],
        }
    }

    #[inline]
    fn alpha_at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return 0;
        }
        self.alpha[(y as u32 * self.w + x as u32) as usize]
    }

    /// Soft coverage 0..1 with optional UV pad (max of neighborhood).
    pub fn coverage_uv(&self, u: f32, v: f32, pad_uv: f32, threshold: u8) -> f32 {
        let fx = u * self.w as f32 - 0.5;
        let fy = v * self.h as f32 - 0.5;
        let rad = (pad_uv * self.w.max(self.h) as f32).ceil().max(0.0) as i32;
        let cx = fx.round() as i32;
        let cy = fy.round() as i32;
        let mut best = 0u8;
        for dy in -rad..=rad {
            for dx in -rad..=rad {
                if rad > 0 && dx * dx + dy * dy > rad * rad {
                    continue;
                }
                best = best.max(self.alpha_at(cx + dx, cy + dy));
            }
        }
        if best < threshold {
            0.0
        } else {
            best as f32 / 255.0
        }
    }

    /// True if UV in 0..1 is opaque (optionally dilated by `pad_uv` in UV units).
    pub fn hits_uv(&self, u: f32, v: f32, pad_uv: f32, threshold: u8) -> bool {
        self.coverage_uv(u, v, pad_uv, threshold) > 0.0
    }
}

pub struct Art {
    pub turret_base: Texture2D,
    pub turret_gun: Texture2D,
    /// Black alpha silhouettes — voids that sit on any terrain.
    pub cracks: [Texture2D; CRACK_VARIANTS],
    /// CPU masks matching `cracks` (for impassable collision).
    pub crack_masks: [CrackMask; CRACK_VARIANTS],
    /// Grayscale ore lump — tinted per resource type.
    pub ore: Texture2D,
    /// Grayscale ingot — tinted per metal type.
    pub ingot: Texture2D,
    /// Corner dock icons (`assets/icons/`).
    pub icon_hammer: Option<Texture2D>,
    pub icon_map: Option<Texture2D>,
    pub icon_tech: Option<Texture2D>,
}

impl Art {
    pub async fn load() -> Self {
        let turret_base = load_tex("assets/buildings/turret_base.webp")
            .or_else(|| load_tex("assets/buildings/turret_base.png"))
            .unwrap_or_else(|| solid_fallback(Color::from_rgba(210, 214, 220, 255)));
        let turret_gun = load_tex("assets/buildings/turret_gun.webp")
            .or_else(|| load_tex("assets/buildings/turret_gun.png"))
            .unwrap_or_else(|| solid_fallback(Color::from_rgba(70, 74, 84, 255)));

        let crack_paths = [
            "assets/environment/Crack1.webp",
            "assets/environment/Crack2.webp",
            "assets/environment/Crack3.webp",
        ];
        let mut cracks = [
            solid_fallback(Color::from_rgba(0, 0, 0, 255)),
            solid_fallback(Color::from_rgba(0, 0, 0, 255)),
            solid_fallback(Color::from_rgba(0, 0, 0, 255)),
        ];
        let mut crack_masks = [
            CrackMask::solid_fallback(),
            CrackMask::solid_fallback(),
            CrackMask::solid_fallback(),
        ];
        for (i, path) in crack_paths.iter().enumerate() {
            if let Some((void, mask)) = load_crack_void_and_mask(path) {
                cracks[i] = void;
                crack_masks[i] = mask;
            }
        }

        let ore = load_tex("assets/items/ore.webp")
            .or_else(|| load_tex("assets/items/ore.png"))
            .unwrap_or_else(|| solid_fallback(Color::from_rgba(160, 160, 160, 255)));
        let ingot = load_tex("assets/items/ingot.webp")
            .or_else(|| load_tex("assets/items/ingot.png"))
            .unwrap_or_else(|| solid_fallback(Color::from_rgba(200, 200, 200, 255)));

        let icon_hammer = load_tex("assets/icons/Hammer.webp")
            .or_else(|| load_tex("assets/icons/Hammer.png"));
        let icon_map =
            load_tex("assets/icons/Map.webp").or_else(|| load_tex("assets/icons/Map.png"));
        let icon_tech =
            load_tex("assets/icons/Tech.webp").or_else(|| load_tex("assets/icons/Tech.png"));

        for t in &cracks {
            t.set_filter(FilterMode::Linear);
        }
        for t in [&turret_base, &turret_gun, &ore, &ingot] {
            t.set_filter(FilterMode::Linear);
        }
        for t in [&icon_hammer, &icon_map, &icon_tech].into_iter().flatten() {
            t.set_filter(FilterMode::Linear);
        }
        Self {
            turret_base,
            turret_gun,
            cracks,
            crack_masks,
            ore,
            ingot,
            icon_hammer,
            icon_map,
            icon_tech,
        }
    }
}

/// Decode WebP/PNG via the `image` crate — macroquad's built-in loader has no WebP.
pub fn try_load_tex(path: &str) -> Option<Texture2D> {
    load_tex(path)
}

fn load_tex(path: &str) -> Option<Texture2D> {
    let data = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&data).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 || w > u16::MAX as u32 || h > u16::MAX as u32 {
        return None;
    }
    Some(Texture2D::from_rgba8(w as u16, h as u16, img.as_raw()))
}

/// Crack mask: black+alpha void texture + CPU alpha for collision.
fn load_crack_void_and_mask(path: &str) -> Option<(Texture2D, CrackMask)> {
    let data = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&data).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 || w > u16::MAX as u32 || h > u16::MAX as u32 {
        return None;
    }
    let mut void_bytes = img.as_raw().to_vec();
    let mut alpha = Vec::with_capacity((w * h) as usize);
    for px in void_bytes.chunks_exact_mut(4) {
        alpha.push(px[3]);
        if px[3] > 0 {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
        }
    }
    let void = Texture2D::from_rgba8(w as u16, h as u16, &void_bytes);
    let mask = CrackMask { w, h, alpha };
    Some((void, mask))
}

fn solid_fallback(c: Color) -> Texture2D {
    let r = (c.r * 255.0) as u8;
    let g = (c.g * 255.0) as u8;
    let b = (c.b * 255.0) as u8;
    let a = (c.a * 255.0) as u8;
    let mut bytes = Vec::with_capacity(16);
    for _ in 0..4 {
        bytes.extend_from_slice(&[r, g, b, a]);
    }
    Texture2D::from_rgba8(2, 2, &bytes)
}

/// Draw a square dock icon centered on `(cx, cy)`.
pub fn draw_dock_icon(tex: &Texture2D, cx: f32, cy: f32, size: f32, tint: Color) {
    let half = size * 0.5;
    draw_texture_ex(
        tex,
        cx - half,
        cy - half,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            ..Default::default()
        },
    );
}

/// Draw base (fixed) then gun (rotated). Sprites are square and fill `w×h`.
pub fn draw_turret(
    art: &Art,
    sx: f32,
    sy: f32,
    w: f32,
    h: f32,
    gun_angle: f32,
    tint: Color,
) {
    let size = w.min(h);
    let ox = sx + (w - size) * 0.5;
    let oy = sy + (h - size) * 0.5;
    let cx = ox + size * 0.5;
    let cy = oy + size * 0.5;
    draw_texture_ex(
        &art.turret_base,
        ox,
        oy,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            ..Default::default()
        },
    );
    draw_texture_ex(
        &art.turret_gun,
        ox,
        oy,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            rotation: gun_angle,
            pivot: Some(vec2(cx, cy)),
            ..Default::default()
        },
    );
}

/// Stable per-vein crack index (0..2) from seed.
pub fn crack_variant(seed: u32) -> usize {
    (seed.wrapping_mul(0x9E3779B9) % CRACK_VARIANTS as u32) as usize
}

/// Stable random rotation in radians from seed.
pub fn crack_rotation(seed: u32) -> f32 {
    crate::deposits::Vein::crack_rotation(seed)
}

/// World size used when drawing a crack (must match collision).
pub fn crack_draw_world_size(v: &crate::deposits::Vein) -> f32 {
    v.crack_world_size() * (0.96 + v.freshness01() * 0.06)
}

/// Map a world point into crack sprite UV (0..1). Returns None if far outside the quad.
pub fn world_to_crack_uv(
    v: &crate::deposits::Vein,
    wx: f32,
    wy: f32,
) -> Option<(usize, f32, f32, f32)> {
    let size = crack_draw_world_size(v);
    if size < 1.0 {
        return None;
    }
    let rot = crack_rotation(v.seed);
    let variant = crack_variant(v.seed);
    let dx = wx - v.x;
    let dy = wy - v.y;
    // Inverse of draw_texture_ex rotation around center (macroquad: CCW).
    let c = rot.cos();
    let s = rot.sin();
    let lx = dx * c + dy * s;
    let ly = -dx * s + dy * c;
    let u = lx / size + 0.5;
    let vv = ly / size + 0.5;
    Some((variant, u, vv, size))
}

/// True if the crack silhouette blocks this world point (with world-space pad for body radius).
pub fn crack_blocks_point(art: &Art, v: &crate::deposits::Vein, wx: f32, wy: f32, pad: f32) -> bool {
    let Some((variant, u, vv, size)) = world_to_crack_uv(v, wx, wy) else {
        return false;
    };
    let pad_uv = (pad / size).clamp(0.0, 0.35);
    art.crack_masks[variant % CRACK_VARIANTS].hits_uv(u, vv, pad_uv, 24)
}

/// Outward wall normal from free-space samples (points toward air, away from crack).
pub fn crack_wall_normal(
    art: &Art,
    v: &crate::deposits::Vein,
    wx: f32,
    wy: f32,
    pad: f32,
) -> (f32, f32) {
    let mut gx = 0.0;
    let mut gy = 0.0;
    let probe = (pad * 0.65).max(4.0);
    const DIRS: usize = 12;
    for i in 0..DIRS {
        let ang = i as f32 / DIRS as f32 * std::f32::consts::TAU;
        let px = wx + ang.cos() * probe;
        let py = wy + ang.sin() * probe;
        if !crack_blocks_point(art, v, px, py, pad) {
            gx += ang.cos();
            gy += ang.sin();
        }
    }
    let len = (gx * gx + gy * gy).sqrt();
    if len > 1e-4 {
        (gx / len, gy / len)
    } else {
        // Buried deep — fall back to radial from vent center.
        let dx = wx - v.x;
        let dy = wy - v.y;
        let d = (dx * dx + dy * dy).sqrt().max(1e-3);
        (dx / d, dy / d)
    }
}

/// Wall-style resolve: walk out along the surface normal in small steps (no teleport shove).
/// Returns (x, y, outward_nx, outward_ny).
pub fn crack_resolve_wall(
    art: &Art,
    v: &crate::deposits::Vein,
    mut wx: f32,
    mut wy: f32,
    pad: f32,
) -> (f32, f32, f32, f32) {
    if !crack_blocks_point(art, v, wx, wy, pad) {
        return (wx, wy, 0.0, 0.0);
    }
    let (mut nx, mut ny) = crack_wall_normal(art, v, wx, wy, pad);
    let step = 2.0;
    for _ in 0..24 {
        if !crack_blocks_point(art, v, wx, wy, pad) {
            break;
        }
        // Refresh normal occasionally so we slide around concave arms.
        let (nnx, nny) = crack_wall_normal(art, v, wx, wy, pad);
        nx = nnx;
        ny = nny;
        wx += nx * step;
        wy += ny * step;
    }
    // One extra nudge so we sit just outside (reduces sticky re-entry).
    if crack_blocks_point(art, v, wx, wy, pad) {
        wx += nx * step;
        wy += ny * step;
    } else {
        wx += nx * 1.0;
        wy += ny * 1.0;
    }
    (wx, wy, nx, ny)
}

/// Draw crack void only (gas ooze is drawn by the caller).
pub fn draw_crack(
    art: &Art,
    variant: usize,
    sx: f32,
    sy: f32,
    size: f32,
    rotation: f32,
    void_alpha: f32,
) {
    let i = variant % CRACK_VARIANTS;
    let cx = sx + size * 0.5;
    let cy = sy + size * 0.5;
    let va = (void_alpha.clamp(0.0, 1.0) * 255.0) as u8;
    draw_texture_ex(
        &art.cracks[i],
        sx,
        sy,
        Color::from_rgba(255, 255, 255, va),
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            rotation,
            pivot: Some(vec2(cx, cy)),
            ..Default::default()
        },
    );
}

/// Draw a grayscale item sprite tinted to `tint`, centered on `(cx, cy)`.
pub fn draw_tinted_item(tex: &Texture2D, cx: f32, cy: f32, size: f32, tint: Color) {
    let s = size.max(1.0);
    draw_texture_ex(
        tex,
        cx - s * 0.5,
        cy - s * 0.5,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(s, s)),
            ..Default::default()
        },
    );
}
