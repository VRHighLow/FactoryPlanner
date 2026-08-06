#![cfg_attr(windows, windows_subsystem = "windows")]

mod art;
mod belts;
mod content;
mod deposits;
mod inventory;
mod player;
mod recipes;
mod sim;
mod net;
mod save;
mod perf_log;
mod ui_chrome;

use macroquad::prelude::*;
use macroquad::window::miniquad::*;
use inventory::{belt_recipe, building_recipe, item_label, Inventory, INV_COLS, INV_SLOTS};
use net::{NetCommand, NetEvent, NetHandle};
use player::{CamMode, Player};
use save::{
    apply_save, capture_save, delete_save, format_bytes, format_playtime, format_saved_at,
    list_saves, preview_path_for, read_save, write_autosave, write_manual_save, EffectQuality,
    GameMode, Settings, AUTOSAVE_INTERVAL_SECS,
};
use sim::*;
use std::collections::HashMap;
use std::time::Instant;
use ui_chrome::{ButtonStyle, UI_AMBER, UI_CYAN, UI_EDGE, UI_PANEL_INNER, UI_SLOT, UI_TEXT, UI_TEXT_DIM};

/// Match draw projection (fixes hitbox/visual drift when context size ≠ framebuffer).
#[inline]
fn screen_width() -> f32 {
    ui_chrome::ui_width()
}
#[inline]
fn screen_height() -> f32 {
    ui_chrome::ui_height()
}

const MIN_ZOOM: f32 = 0.35;
const MAX_ZOOM: f32 = 2.5;
/// Must match `belts::TILE_SIZE` / `sim::TILE_SIZE`.
const GRID_MINOR: f32 = 40.0;
const GRID_MAJOR_EVERY: i32 = 10;
const HOTBAR_SLOTS: usize = 9;
/// Fixed simulation rate (Factorio-style UPS). Independent of render FPS.
const TARGET_UPS: f64 = 60.0;
const FIXED_DT: f32 = 1.0 / TARGET_UPS as f32;
/// Cap catch-up steps so a hitch doesn't spiral the sim.
const MAX_SIM_STEPS: u32 = 12;
/// Max sim time debt kept when the step budget is exhausted (soft slowdown).
const MAX_SIM_DEBT: f32 = 0.25;

const BG: Color = Color::from_rgba(28, 30, 28, 255);
const GRID_MINOR_C: Color = Color::from_rgba(62, 70, 58, 55);
const NODE_BORDER: Color = Color::from_rgba(120, 140, 160, 180);
const CYAN: Color = Color::from_rgba(72, 220, 205, 255);
const BELT_YELLOW: Color = Color::from_rgba(210, 170, 55, 255);
const POWER_C: Color = Color::from_rgba(255, 190, 70, 255);
const POWER_DIM: Color = Color::from_rgba(255, 190, 70, 90);
const TEXT: Color = Color::from_rgba(228, 236, 244, 255);
const TEXT_DIM: Color = Color::from_rgba(148, 162, 178, 255);
const ACCENT: Color = Color::from_rgba(255, 168, 72, 255);
const ORE_C: Color = Color::from_rgba(140, 140, 150, 255);
const INGOT_C: Color = Color::from_rgba(190, 200, 220, 255);

/// Starting clear pocket radius (world units). Totems expand further.
const STORM_SAFE_RADIUS: f32 = 4320.0;
const STORM_MAX_TOTEMS: usize = 8;
const STORM_MAX_FLASHES: usize = 4;
const LIGHT_MAX: usize = 32;
/// Hard build border sits inside the visual fog coast (~0.78 in the shader).
const STORM_HARD_CLEAR_SCALE: f32 = 0.72;
/// World-space radius of cloud illumination around a strike.
const STORM_FLASH_RADIUS: f32 = 520.0;

const STORM_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying vec2 uv;
varying vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    // macroquad stores color0 as Byte4 (0..255), NOT normalized — must divide.
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

/// World-space point lights: darkens unlit floor, leaves lit areas clear (real 2D lighting).
const LIGHT_FRAGMENT: &str = r#"#version 100
precision highp float;

varying vec2 uv;
varying vec4 color;

uniform vec2 ScreenSize;
uniform vec2 CamPos;
uniform float CamZoom;
uniform float Ambient;
uniform vec4 Lights[32];

void main() {
    vec2 screen = vec2(gl_FragCoord.x, ScreenSize.y - gl_FragCoord.y);
    vec2 world = (screen - 0.5 * ScreenSize) / max(CamZoom, 0.001) + CamPos;

    float illum = Ambient;
    vec3 warm = vec3(0.0);
    for (int i = 0; i < 32; i++) {
        float inten = Lights[i].w;
        if (inten > 0.01) {
            vec2 d = world - Lights[i].xy;
            float rad = max(Lights[i].z, 8.0);
            float nd = length(d) / rad;
            // Smooth physical-ish falloff: bright core, soft edge, no hard disc.
            float atten = 1.0 / (1.0 + nd * nd * 2.75);
            atten *= 1.0 - smoothstep(0.72, 1.0, nd);
            illum += inten * atten;
            warm += vec3(1.0, 0.84, 0.62) * (inten * atten);
        }
    }
    illum = clamp(illum, 0.0, 1.45);
    warm = clamp(warm, 0.0, 1.0);

    // How much to darken the scene (0 = fully lit / transparent).
    float shade = 1.0 - smoothstep(0.22, 1.05, illum);
    shade = clamp(shade, 0.0, 1.0);

    // Cool shadow tint in unlit areas; tiny warm lift where lights actually reach.
    vec3 shadow = vec3(0.03, 0.04, 0.06);
    vec3 rgb = mix(shadow, warm * 0.18, clamp(illum - Ambient, 0.0, 1.0) * 0.45);
    float alpha = shade * 0.52;
    // Do NOT multiply by vertex `color` unless it's been /255'd in the vertex shader.
    // Multiplying by raw Byte4 (0..255) blows this fullscreen pass to opaque white.
    gl_FragColor = vec4(rgb, alpha);
}
"#;

/// Soft natural storm-earth — wet soil / mud / dead scrub under perpetual overcast.
const GROUND_FRAGMENT: &str = r#"#version 100
precision highp float;

varying vec2 uv;
varying vec4 color;

uniform vec2 ScreenSize;
uniform vec2 CamPos;
uniform float CamZoom;
uniform float Time;
uniform vec4 Totems[8];

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    // 3 octaves — enough dirt detail, far cheaper on integrated GPUs.
    for (int i = 0; i < 3; i++) {
        v += a * noise(p);
        p = p * 2.07 + vec2(1.7, 9.2);
        a *= 0.5;
    }
    return v;
}

float clear_amt(vec2 world) {
    float best = 0.0;
    for (int i = 0; i < 8; i++) {
        if (Totems[i].w > 0.5) {
            float r = max(Totems[i].z, 1.0);
            float d = length(world - Totems[i].xy) / r;
            best = max(best, 1.0 - smoothstep(0.28, 1.18, d));
        }
    }
    return clamp(best, 0.0, 1.0);
}

void main() {
    vec2 screen = vec2(gl_FragCoord.x, ScreenSize.y - gl_FragCoord.y);
    vec2 world = (screen - 0.5 * ScreenSize) / max(CamZoom, 0.001) + CamPos;
    float clear = clear_amt(world);

    // World-scale soil layers (no screen mosaic — stays natural while panning).
    vec2 p = world * 0.011;
    float soil = fbm(p);
    float mud = fbm(p * 1.85 + vec2(3.1, 8.4));
    // Reuse soil for grit cue — skips a third full fBm on every pixel.
    float grit = soil;

    // Slow storm-cloud shadows drifting across the land.
    float t = Time * 0.035;
    float cloud = fbm(world * 0.0028 + vec2(t * 0.55, -t * 0.22));
    float shadow = smoothstep(0.28, 0.72, cloud);

    // Palette: dark wet earth under perpetual overcast (readable, not crushed black).
    vec3 dry = vec3(0.22, 0.20, 0.17);      // ash clay
    vec3 damp = vec3(0.14, 0.15, 0.13);      // wet soil
    vec3 mudc = vec3(0.10, 0.11, 0.10);     // puddle mud
    vec3 grassc = vec3(0.21, 0.22, 0.155);  // lighter dead-grass patches
    vec3 storm = vec3(0.08, 0.09, 0.11);     // under deep fog

    // Light patches = grass ground (high soil, low mud). Dark = wet mud.
    float grass = smoothstep(0.42, 0.74, soil) * (1.0 - smoothstep(0.55, 0.85, mud));
    vec3 col = mix(damp, dry, smoothstep(0.35, 0.7, soil));
    col = mix(col, mudc, smoothstep(0.62, 0.88, mud) * 0.85);
    col = mix(col, grassc, grass * 0.75);

    // Soft wet sheen — slight highlight on raised soil.
    float sheen = smoothstep(0.45, 0.85, soil + grit * 0.25);
    col += vec3(0.035, 0.04, 0.045) * sheen * (0.35 + clear * 0.45);

    // Soft organic grit (no ridge/contour grass — that caused vertical striping).
    float grit_fleck = smoothstep(0.72, 0.95, grit) * 0.04;
    col += vec3(0.03, 0.032, 0.028) * grit_fleck;

    // Occasional soft pebble dots (hashed cells, not streaks).
    float pebble = step(0.955, hash(floor(world * 0.18)));
    col += vec3(0.045, 0.042, 0.035) * pebble;

    // Cloud shadow + clear-pocket lift.
    col *= mix(0.72, 1.0, 1.0 - shadow * 0.55);
    col = mix(storm, col, mix(0.55, 1.0, clear));
    // Cool wet rim where storm fog meets reclaimed ground.
    float coast = smoothstep(0.08, 0.45, clear) * (1.0 - smoothstep(0.45, 0.95, clear));
    col += vec3(0.02, 0.035, 0.05) * coast * 0.55;

    // Mild banding so it still reads in a pixel game without mosaic tiles.
    col = floor(col * 22.0 + 0.5) / 22.0;
    gl_FragColor = vec4(col, 1.0);
}
"#;

const STORM_FRAGMENT: &str = r#"#version 100
precision highp float;

varying vec2 uv;
varying vec4 color;

uniform vec2 ScreenSize;
uniform vec2 CamPos;
uniform float CamZoom;
uniform float Time;
uniform vec4 Totems[8];
uniform vec4 Flashes[4];
uniform float PixelSize;

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < 3; i++) {
        v += a * noise(p);
        p *= 2.03;
        a *= 0.5;
    }
    return v;
}

float clear_zone(vec2 world, vec2 center, float radius, float mist, float fine) {
    if (radius < 1.0) {
        return 0.0;
    }
    float rr = length(world - center) / radius;
    float coast = 0.78 + (mist - 0.5) * 0.85 + (fine - 0.5) * 0.25;
    return 1.0 - smoothstep(coast - 0.32, coast + 0.48, rr);
}

void main() {
    vec2 screen = vec2(gl_FragCoord.x, ScreenSize.y - gl_FragCoord.y);
    float pixel = max(PixelSize, 4.0);
    // Lock mosaic to the world while panning (screen-fixed grids crawl/jitter).
    // For fixed world W: screen + CamPos*CamZoom = W*CamZoom + half_screen.
    vec2 locked = screen + CamPos * CamZoom;
    locked = floor(locked / pixel) * pixel + pixel * 0.5;
    screen = locked - CamPos * CamZoom;
    vec2 world = (screen - 0.5 * ScreenSize) / max(CamZoom, 0.001) + CamPos;

    float t = Time * 0.028;
    vec2 q = world * 0.00155;
    vec2 warp = vec2(
        fbm(q + vec2(t, 0.0)),
        fbm(q + vec2(5.2, 1.3) - t)
    );
    vec2 p = q + (warp - 0.5) * 1.25;

    float lumps = fbm(p * 1.25);
    float detail = fbm(p * 3.4 + t * 0.45);
    float fine = fbm(p * 7.0 - t * 0.2);
    float mist = lumps * 0.55 + detail * 0.30 + fine * 0.15;

    float clear_amt = 0.0;
    for (int i = 0; i < 8; i++) {
        if (Totems[i].w > 0.5) {
            clear_amt = max(
                clear_amt,
                clear_zone(world, Totems[i].xy, Totems[i].z, mist, fine)
            );
        }
    }
    float outside = 1.0 - clear_amt;

    // Denser billows, fewer voids.
    float body = smoothstep(0.16, 0.52, mist);
    float holes = 1.0 - smoothstep(0.10, 0.32, detail) * 0.38;
    float density = clamp(outside * body * holes, 0.0, 1.0);

    vec3 col = mix(vec3(0.36, 0.34, 0.46), vec3(0.78, 0.80, 0.88), mist);
    col = mix(col, vec3(0.45, 0.52, 0.72), detail * 0.35);

    float rim = smoothstep(0.02, 0.20, density) * (1.0 - smoothstep(0.20, 0.50, density));
    col += vec3(0.40, 0.62, 0.88) * rim * 0.5;

    // Localized cloud flash — only lights fog near each strike (aerial storm view).
    float lit = 0.0;
    for (int i = 0; i < 4; i++) {
        float inten = Flashes[i].z;
        if (inten > 0.01) {
            vec2 d = world - Flashes[i].xy;
            float dist2 = dot(d, d);
            float rad = max(Flashes[i].w, 80.0);
            float fall = exp(-dist2 / (rad * rad));
            lit = max(lit, inten * fall);
        }
    }
    lit = clamp(lit, 0.0, 1.5) * density;
    float core_lit = lit * (0.55 + mist * 0.55);
    col = mix(col, vec3(0.78, 0.82, 0.95), core_lit * 0.55);
    col += vec3(0.65, 0.72, 1.0) * core_lit * 0.55;
    col += vec3(0.95, 0.97, 1.0) * lit * lit * 0.35;

    float alpha = density * (0.42 + 0.48 * mist) + outside * (0.10 + 0.10 * lumps);
    alpha = clamp(alpha + lit * 0.18, 0.0, 0.92);
    // Posterize so mosaic cells read as flat pixels, not soft gradients.
    col = floor(col * 14.0 + 0.5) / 14.0;
    alpha = floor(alpha * 12.0 + 0.5) / 12.0;
    if (alpha < 0.015) {
        discard;
    }
    gl_FragColor = vec4(col, alpha);
}
"#;

const CANNON_MAX_CHARGES: usize = 8;
const CANNON_MAX_BEAMS: usize = 8;
const GAS_MAX_VENTS: usize = 16;
/// Shared mosaic size for storm / cannon / lightning FX (screen pixels).
const FX_PIXEL: f32 = 8.0;
/// Finer mosaic for vent gas wisps — chunky 8px reads as chimneys on small cracks.
const GAS_FX_PIXEL: f32 = 4.0;

/// Faint organic smoke from crack silhouette sites — soft billows drifting screen NNE.
const GAS_FRAGMENT: &str = r#"#version 100
precision highp float;

varying vec2 uv;
varying vec4 color;

uniform vec2 ScreenSize;
uniform vec2 CamPos;
uniform float CamZoom;
uniform float Time;
uniform float PixelSize;
uniform vec4 Vents[16];
uniform vec4 VentColor[16];
uniform vec4 VentXform[16];
uniform sampler2D Crack0;
uniform sampler2D Crack1;
uniform sampler2D Crack2;

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < 5; i++) {
        v += a * noise(p);
        p *= 2.02;
        a *= 0.5;
    }
    return v;
}

float sample_crack(float variant, vec2 tuv) {
    if (tuv.x < 0.0 || tuv.y < 0.0 || tuv.x > 1.0 || tuv.y > 1.0) {
        return 0.0;
    }
    if (variant < 0.5) {
        return texture2D(Crack0, tuv).a;
    } else if (variant < 1.5) {
        return texture2D(Crack1, tuv).a;
    }
    return texture2D(Crack2, tuv).a;
}

float crack_solid(float variant, vec2 tuv) {
    return step(0.28, sample_crack(variant, tuv));
}

void main() {
    vec2 screen = vec2(gl_FragCoord.x, ScreenSize.y - gl_FragCoord.y);
    float pixel = max(PixelSize, 3.0);
    vec2 locked = screen + CamPos * CamZoom;
    locked = floor(locked / pixel) * pixel + pixel * 0.5;
    screen = locked - CamPos * CamZoom;
    vec2 world = (screen - 0.5 * ScreenSize) / max(CamZoom, 0.001) + CamPos;
    float t = Time;

    // Always drift NNE in screen/world space (top + a little right).
    vec2 drift = normalize(vec2(0.34, -1.0));
    vec2 perp = vec2(-drift.y, drift.x);

    vec3 col = vec3(0.0);
    float alpha = 0.0;

    for (int i = 0; i < 16; i++) {
        float inten = Vents[i].w;
        if (inten < 0.02) {
            continue;
        }
        vec2 c = Vents[i].xy;
        float size = max(Vents[i].z, 16.0);
        float cs = VentXform[i].x;
        float sn = VentXform[i].y;
        float variant = VentXform[i].z;
        float seed = VentXform[i].w;

        vec2 d0 = world - c;
        if (dot(d0, d0) > size * size * 2.6) {
            continue;
        }

        // Shared smoke noise field (organic billows).
        vec2 q = (world * 0.02) - drift * (t * 0.38) + vec2(seed * 0.07, seed * 0.03);
        float mist = fbm(q);
        float detail = fbm(q * 2.25 + vec2(t * 0.12, -t * 0.18));
        float body = smoothstep(0.30, 0.64, mist * 0.62 + detail * 0.38);

        float best = 0.0;
        // Trace upwind to crack pixels — smoke originates on the fissure arms, not the sprite center.
        for (int k = 0; k <= 12; k++) {
            float along = float(k) * (size * 0.045);
            float rise01 = float(k) / 12.0;
            // Soft lateral search so puffs have width without becoming a center blob.
            float sigma = size * (0.035 + rise01 * 0.07);

            for (int j = -2; j <= 2; j++) {
                float side = float(j) * (sigma * 0.42);
                vec2 src_world = world - drift * along - perp * side;
                vec2 sd = src_world - c;
                vec2 slocal = vec2(sd.x * cs + sd.y * sn, -sd.x * sn + sd.y * cs);
                vec2 src_uv = slocal / size + 0.5;

                float solid = crack_solid(variant, src_uv);
                if (solid < 0.5) {
                    continue;
                }

                // Random subset of crack cells (~35%) so it seeps from scattered arms.
                vec2 cell = floor(src_uv * 18.0 + vec2(seed * 0.21, seed * 0.09));
                if (hash(cell) < 0.65) {
                    continue;
                }

                // Soft puff around that crack site, drifting NNE.
                float sway = (detail - 0.5) * sigma * 0.55;
                float lat = side - sway;
                float blob = exp(-(lat * lat) / max(2.0 * sigma * sigma, 1.0));
                // k=0 is on the crack — keep a little seep; main smoke is just downwind.
                float profile = (k == 0)
                    ? 0.35
                    : (smoothstep(0.05, 0.22, rise01) * (1.0 - smoothstep(0.55, 1.0, rise01)));
                float dens = blob * body * profile;
                best = max(best, dens);
            }
        }

        if (best < 0.03) {
            continue;
        }

        best *= inten * 0.42;
        best = clamp(best, 0.0, 1.0);

        vec3 gas = VentColor[i].rgb;
        vec3 tip = mix(gas, vec3(1.0), 0.18);
        col += mix(gas, tip, detail * 0.35) * best;
        alpha = max(alpha, best * 0.58);
    }

    col = floor(col * 20.0 + 0.5) / 20.0;
    alpha = floor(alpha * 16.0 + 0.5) / 16.0;
    if (alpha < 0.03) {
        discard;
    }
    gl_FragColor = vec4(col, alpha);
}
"#;

/// Atmospheric plasma for charge orbs + bolt blooms — same noise language as the storm.
const CANNON_FRAGMENT: &str = r#"#version 100
precision highp float;

varying vec2 uv;
varying vec4 color;

uniform vec2 ScreenSize;
uniform vec2 CamPos;
uniform float CamZoom;
uniform float Time;
uniform float PixelSize;
uniform vec4 Charges[8];
uniform vec4 Beams[8];
uniform vec4 BeamLife[8];

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < 4; i++) {
        v += a * noise(p);
        p *= 2.07;
        a *= 0.5;
    }
    return v;
}

float sd_segment(vec2 p, vec2 a, vec2 b) {
    vec2 pa = p - a;
    vec2 ba = b - a;
    float h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-4), 0.0, 1.0);
    return length(pa - ba * h);
}

void main() {
    vec2 screen = vec2(gl_FragCoord.x, ScreenSize.y - gl_FragCoord.y);
    float pixel = max(PixelSize, 4.0);
    // Same world-locked mosaic as the storm fog.
    vec2 locked = screen + CamPos * CamZoom;
    locked = floor(locked / pixel) * pixel + pixel * 0.5;
    screen = locked - CamPos * CamZoom;
    vec2 world = (screen - 0.5 * ScreenSize) / max(CamZoom, 0.001) + CamPos;
    float t = Time;

    vec3 col = vec3(0.0);
    float alpha = 0.0;

    // --- Charge orbs: swirling plasma cores ---
    for (int i = 0; i < 8; i++) {
        float ch = Charges[i].w;
        if (ch < 0.02) {
            continue;
        }
        vec2 c = Charges[i].xy;
        float rad = max(Charges[i].z, 8.0);
        vec2 d = world - c;
        float dist = length(d);
        float nd = dist / rad;

        vec2 q = d * 0.08;
        float ang = atan(d.y, d.x);
        float swirl = fbm(q * 2.4 + vec2(ang * 0.4 + t * 2.0, t * 1.1));
        float filament = fbm(q * 5.0 - vec2(t * 2.6, ang));
        float plasma = swirl * 0.6 + filament * 0.4;

        float corona = exp(-nd * nd * 3.4) * (0.5 + plasma * 0.55);
        float core = exp(-nd * nd * 12.0) * (0.85 + filament * 0.4);
        float ring = smoothstep(0.62, 0.42, nd) * smoothstep(0.18, 0.4, nd);
        ring *= 0.5 + sin(ang * 5.0 + t * 5.5 + plasma * 6.0) * 0.35;

        float inten = ch * ch;
        // Warm charge look (polished version of the original orange orb).
        vec3 outer = vec3(1.0, 0.42, 0.12);
        vec3 mid = vec3(1.0, 0.68, 0.28);
        vec3 hot = vec3(1.0, 0.94, 0.78);
        col += outer * corona * inten * 0.7;
        col += mid * (corona * 0.45 + ring * 0.55) * inten;
        col += hot * core * inten * (0.75 + ch * 0.5);
        alpha = max(alpha, (corona * 0.5 + core * 0.8 + ring * 0.35) * inten);
    }

    // --- Beam blooms + traveling bolt heads ---
    for (int i = 0; i < 8; i++) {
        float life = BeamLife[i].x;
        if (life < 0.01) {
            continue;
        }
        vec2 a = Beams[i].xy;
        vec2 b = Beams[i].zw;
        float dist = sd_segment(world, a, b);
        float len = max(length(b - a), 1.0);
        vec2 dir = (b - a) / len;
        float along = clamp(dot(world - a, dir) / len, 0.0, 1.0);

        float age = 1.0 - life;
        float travel = clamp(age * 2.1, 0.0, 1.0);
        vec2 head = mix(a, b, travel);
        float head_d = length(world - head);

        float mist = fbm((world - a) * 0.015 + vec2(t * 2.0, along * 3.0));
        // Tight laser bloom — straight capsule, slight shimmer only.
        float glow = exp(-dist * dist / (18.0 * 18.0)) * (0.65 + mist * 0.25);
        float sheath = exp(-dist * dist / (7.0 * 7.0));
        float core = exp(-dist * dist / (2.2 * 2.2));
        float tip = exp(-head_d * head_d / (12.0 * 12.0));
        float tip_hot = exp(-head_d * head_d / (4.0 * 4.0));

        // Impact bloom near end once bolt arrives.
        float impact = smoothstep(0.55, 0.95, travel) * life;
        float impact_d = length(world - b);
        float shock = exp(-impact_d * impact_d / (32.0 * 32.0)) * impact;
        float shock_core = exp(-impact_d * impact_d / (10.0 * 10.0)) * impact;

        float fade = life * (0.55 + life * 0.45);
        vec3 outer = vec3(1.0, 0.38, 0.10);
        vec3 amber = vec3(1.0, 0.62, 0.28);
        vec3 white = vec3(1.0, 0.96, 0.88);

        col += outer * glow * 0.45 * fade;
        col += amber * sheath * 0.95 * fade;
        col += mix(amber, white, 0.7) * core * 1.35 * fade;
        col += amber * tip * 0.9 * fade;
        col += white * tip_hot * 1.35 * fade;
        col += outer * shock * 0.6;
        col += white * shock_core * 0.9;
        alpha = max(alpha, (glow * 0.28 + sheath * 0.6 + core * 0.85 + tip * 0.5 + shock * 0.45) * fade);
    }

    col = clamp(col, 0.0, 1.6);
    alpha = clamp(alpha, 0.0, 0.92);
    // Posterize to match storm pixel filter.
    col = floor(col * 6.0 + 0.5) / 6.0;
    alpha = floor(alpha * 12.0 + 0.5) / 12.0;
    if (alpha < 0.015) {
        discard;
    }
    gl_FragColor = vec4(col, alpha);
}
"#;

fn create_cannon_material() -> Option<Material> {
    let pipeline_params = PipelineParams {
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };
    load_material(
        ShaderSource::Glsl {
            vertex: STORM_VERTEX,
            fragment: CANNON_FRAGMENT,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("ScreenSize", UniformType::Float2),
                UniformDesc::new("CamPos", UniformType::Float2),
                UniformDesc::new("CamZoom", UniformType::Float1),
                UniformDesc::new("Time", UniformType::Float1),
                UniformDesc::new("PixelSize", UniformType::Float1),
                UniformDesc::array(
                    UniformDesc::new("Charges", UniformType::Float4),
                    CANNON_MAX_CHARGES,
                ),
                UniformDesc::array(
                    UniformDesc::new("Beams", UniformType::Float4),
                    CANNON_MAX_BEAMS,
                ),
                UniformDesc::array(
                    UniformDesc::new("BeamLife", UniformType::Float4),
                    CANNON_MAX_BEAMS,
                ),
            ],
            pipeline_params,
            ..Default::default()
        },
    )
    .map_err(|e| {
        eprintln!("cannon fx shader unavailable: {e:?}");
        e
    })
    .ok()
}

fn create_gas_material() -> Option<Material> {
    let pipeline_params = PipelineParams {
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };
    load_material(
        ShaderSource::Glsl {
            vertex: STORM_VERTEX,
            fragment: GAS_FRAGMENT,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("ScreenSize", UniformType::Float2),
                UniformDesc::new("CamPos", UniformType::Float2),
                UniformDesc::new("CamZoom", UniformType::Float1),
                UniformDesc::new("Time", UniformType::Float1),
                UniformDesc::new("PixelSize", UniformType::Float1),
                UniformDesc::array(UniformDesc::new("Vents", UniformType::Float4), GAS_MAX_VENTS),
                UniformDesc::array(
                    UniformDesc::new("VentColor", UniformType::Float4),
                    GAS_MAX_VENTS,
                ),
                UniformDesc::array(
                    UniformDesc::new("VentXform", UniformType::Float4),
                    GAS_MAX_VENTS,
                ),
            ],
            textures: vec![
                "Crack0".to_string(),
                "Crack1".to_string(),
                "Crack2".to_string(),
            ],
            pipeline_params,
            ..Default::default()
        },
    )
    .map_err(|e| {
        eprintln!("gas vent shader unavailable: {e:?}");
        e
    })
    .ok()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Main,
    SinglePlayer,
    Play,
    Multiplayer,
    HostSetup,
    HostLobby,
    JoinLobby,
    Game,
    Settings,
    LoadGame,
}

impl Screen {
    fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::SinglePlayer => "single",
            Self::Play => "new_game",
            Self::Multiplayer => "multi",
            Self::HostSetup => "host_setup",
            Self::HostLobby => "host_lobby",
            Self::JoinLobby => "join",
            Self::Game => "game",
            Self::Settings => "settings",
            Self::LoadGame => "load",
        }
    }
}

/// New Game / Load Game destination: solo vs hosting a session.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MenuPlayIntent {
    Solo,
    Host,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsCategory {
    Display,
    Audio,
    Keybinds,
}

impl SettingsCategory {
    const ALL: [SettingsCategory; 3] = [Self::Display, Self::Audio, Self::Keybinds];

    fn label(self) -> &'static str {
        match self {
            Self::Display => "Graphics",
            Self::Audio => "Audio",
            Self::Keybinds => "Keybinds",
        }
    }

    fn from_index(i: usize) -> Self {
        Self::ALL.get(i).copied().unwrap_or(Self::Display)
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|&c| c == self).unwrap_or(0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CornerTool {
    Build,
    Recipes,
    TechTree,
    Map,
    NodeChart,
}

impl CornerTool {
    const ALL: [CornerTool; 5] = [
        Self::Build,
        Self::Recipes,
        Self::TechTree,
        Self::Map,
        Self::NodeChart,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Build => "Build",
            Self::Recipes => "Recipes",
            Self::TechTree => "Tech",
            Self::Map => "Map",
            Self::NodeChart => "Nodes",
        }
    }
}

struct PeerPresence {
    id: u8,
    /// Mouse cursor (placement / aim).
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    /// Authoritative remote player (circle placeholder).
    drone: player::RemoteDrone,
    selected: Option<BuildingKind>,
    facing: Facing,
    last_sample_t: f32,
}

/// Build-menu / hotbar entry: core tool or Era machine from the data pack.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BuildEntry {
    Kind(BuildingKind),
    Era(u16),
}

/// Era pack machines that must place as a specific `BuildingKind` (combat / fluids / nexus).
fn building_kind_for_era_machine(mid: u16) -> BuildingKind {
    let id = content::content()
        .machine(mid)
        .map(|m| m.id.as_str())
        .unwrap_or("");
    match id {
        "era1_machine_wall" => BuildingKind::Wall,
        "era1_machine_reinforced_wall" => BuildingKind::ReinforcedWall,
        "era1_machine_ballistic_turret" => BuildingKind::BallisticTurret,
        "era1_machine_laser_turret" => BuildingKind::LaserTurret,
        "era1_machine_charge_cannon" => BuildingKind::Turret,
        "era1_machine_storage_tank" | "era1_machine_fluid_tank" => BuildingKind::FluidTank,
        "era1_machine_construction_site" => BuildingKind::NexusSite,
        "era1_machine_planetary_nexus" => BuildingKind::Nexus,
        "era1_machine_solar_panel_mk1" => BuildingKind::Solar,
        "era1_machine_research_laboratory" | "era1_machine_laboratory_module" => BuildingKind::Lab,
        "era1_machine_thermal_smelter_mk1" => BuildingKind::Smelter,
        "era1_machine_assembler_mk1" => BuildingKind::Assembler,
        _ => BuildingKind::Machine,
    }
}

impl BuildEntry {
    fn kind(self) -> BuildingKind {
        match self {
            Self::Kind(k) => k,
            Self::Era(id) => building_kind_for_era_machine(id),
        }
    }

    fn machine_id(self) -> Option<u16> {
        match self {
            Self::Kind(_) => None,
            Self::Era(id) => Some(id),
        }
    }

    fn label(self) -> String {
        match self {
            Self::Kind(k) => k.label().to_string(),
            Self::Era(id) => content::content()
                .machine(id)
                .map(|m| m.name.clone())
                .unwrap_or_else(|| format!("Machine#{id}")),
        }
    }

    fn short(self) -> String {
        match self {
            Self::Kind(k) => k.short().to_string(),
            Self::Era(id) => {
                let name = content::content()
                    .machine(id)
                    .map(|m| m.name.as_str())
                    .unwrap_or("?");
                if name.len() <= 8 {
                    name.to_string()
                } else {
                    format!("{}…", &name[..7])
                }
            }
        }
    }

    fn hint(self) -> String {
        match self {
            Self::Kind(k) => k.hint().to_string(),
            Self::Era(id) => content::content()
                .machine(id)
                .map(|m| {
                    let mut parts = Vec::new();
                    if !m.function.is_empty() {
                        parts.push(m.function.clone());
                    } else if !m.description.is_empty() {
                        let d = if m.description.len() > 72 {
                            format!("{}…", &m.description[..70])
                        } else {
                            m.description.clone()
                        };
                        parts.push(d);
                    }
                    parts.push(format!(
                        "T{} · {} · {:.0} kW",
                        m.tier,
                        if m.power_type.is_empty() {
                            "power"
                        } else {
                            m.power_type.as_str()
                        },
                        m.power_kw
                    ));
                    if !m.fluid_ports.is_empty() {
                        parts.push(format!("fluids: {}", m.fluid_ports.join("/")));
                    }
                    if !m.purity_behavior.is_empty() {
                        parts.push(m.purity_behavior.clone());
                    }
                    parts.join(" · ")
                })
                .unwrap_or_default(),
        }
    }

    fn tech_unlock(self) -> String {
        match self {
            Self::Kind(k) => k.tech_unlock().to_string(),
            Self::Era(id) => content::content()
                .machine(id)
                .map(|m| m.technology_unlock.clone())
                .unwrap_or_else(|| "era1_tech_basic_recovery".into()),
        }
    }

    fn category(self) -> BuildCategory {
        match self {
            Self::Kind(k) => k.category(),
            Self::Era(id) => {
                let cat = content::content()
                    .machine(id)
                    .map(|m| m.category.as_str())
                    .unwrap_or("");
                machine_category_to_build(cat)
            }
        }
    }

    fn matches_query(self, query: &str) -> bool {
        let q = query.trim();
        if q.is_empty() {
            return true;
        }
        let q = q.to_ascii_lowercase();
        if self.label().to_ascii_lowercase().contains(&q)
            || self.short().to_ascii_lowercase().contains(&q)
        {
            return true;
        }
        match self {
            Self::Era(id) => content::content()
                .machine(id)
                .map(|m| {
                    m.id.to_ascii_lowercase().contains(&q)
                        || m.category.to_ascii_lowercase().contains(&q)
                })
                .unwrap_or(false),
            Self::Kind(_) => false,
        }
    }

    fn swatch(self) -> Color {
        match self {
            Self::Kind(k) => kind_swatch(k),
            Self::Era(id) => {
                let cat = content::content()
                    .machine(id)
                    .map(|m| m.category.as_str())
                    .unwrap_or("");
                match cat {
                    "extraction" | "mining" => ORE_C,
                    "military" | "defense" => Color::from_rgba(200, 90, 90, 255),
                    "chemical" | "water" | "water_purification" => {
                        Color::from_rgba(70, 140, 200, 255)
                    }
                    "research" => Color::from_rgba(140, 100, 220, 255),
                    "energy" | "power" => POWER_C,
                    _ => Color::from_rgba(100, 150, 170, 255),
                }
            }
        }
    }
}

fn machine_category_to_build(cat: &str) -> BuildCategory {
    match cat {
        "extraction" | "mining" | "atmosphere" => BuildCategory::Resource,
        "energy" | "power" => BuildCategory::Energy,
        "military" | "defense" | "ammunition" => BuildCategory::Defense,
        "storage" => BuildCategory::Storage,
        "logistics" | "transport" => BuildCategory::Transport,
        "nexus" => BuildCategory::Processing,
        _ => BuildCategory::Processing,
    }
}

/// Infrastructure / tools not fully replaced by the Era machine pack.
fn core_build_kinds() -> &'static [BuildingKind] {
    &[
        BuildingKind::PowerPole,
        BuildingKind::OreNode,
        BuildingKind::Box,
        BuildingKind::Splitter,
        BuildingKind::Pipe,
        BuildingKind::Totem,
        BuildingKind::PowerWire,
        BuildingKind::Conveyor,
    ]
}

struct Cam {
    x: f32,
    y: f32,
    zoom: f32,
}

impl Cam {
    fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        (
            ((wx - self.x) * self.zoom + screen_width() * 0.5).round(),
            ((wy - self.y) * self.zoom + screen_height() * 0.5).round(),
        )
    }

    fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        (
            (sx - screen_width() * 0.5) / self.zoom + self.x,
            (sy - screen_height() * 0.5) / self.zoom + self.y,
        )
    }

    /// World-space AABB of the viewport, expanded by `margin` world units.
    fn view_world_aabb(&self, margin: f32) -> (f32, f32, f32, f32) {
        let (x0, y0) = self.screen_to_world(-margin, -margin);
        let (x1, y1) = self.screen_to_world(screen_width() + margin, screen_height() + margin);
        (x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1))
    }

    fn world_rect_visible(&self, x: f32, y: f32, w: f32, h: f32, margin: f32) -> bool {
        let (min_x, min_y, max_x, max_y) = self.view_world_aabb(margin);
        x + w >= min_x && x <= max_x && y + h >= min_y && y <= max_y
    }

    fn world_circle_visible(&self, cx: f32, cy: f32, r: f32, margin: f32) -> bool {
        let (min_x, min_y, max_x, max_y) = self.view_world_aabb(margin);
        cx + r >= min_x && cx - r <= max_x && cy + r >= min_y && cy - r <= max_y
    }

    /// Lock cam to the zoom lattice so grid lines / labels don't shimmer while following.
    fn quantize(&mut self) {
        let z = self.zoom.max(1e-4);
        self.x = (self.x * z).round() / z;
        self.y = (self.y * z).round() / z;
    }
}

/// Visual storm border — clear pocket at origin; fog outside. Gameplay later.
struct Storm {
    cx: f32,
    cy: f32,
    radius: f32,
    time: f32,
    /// Local cloud flashes: (x, y, intensity, radius).
    flashes: [(f32, f32, f32, f32); STORM_MAX_FLASHES],
    material: Option<Material>,
}

impl Storm {
    fn new(material: Option<Material>) -> Self {
        Self {
            cx: 0.0,
            cy: 0.0,
            radius: STORM_SAFE_RADIUS,
            time: 0.0,
            flashes: [(0.0, 0.0, 0.0, 0.0); STORM_MAX_FLASHES],
            material,
        }
    }

    fn tick(&mut self, dt: f32) {
        self.time += dt;
        for slot in &mut self.flashes {
            if slot.2 <= 0.0 {
                continue;
            }
            slot.2 = (slot.2 - dt * 4.5).max(0.0);
            if slot.2 > 0.15 && storm_hash01(self.time * 40.0 + slot.0 * 0.01) > 0.82 {
                slot.2 = (slot.2 * 0.55).max(0.08);
            }
            if slot.2 <= 0.01 {
                *slot = (0.0, 0.0, 0.0, 0.0);
            }
        }
    }

    fn trigger_flash(&mut self, x: f32, y: f32, intensity: f32) {
        let intensity = intensity.clamp(0.2, 1.5);
        // Reinforce a nearby existing flash, else take weakest/empty slot.
        let mut best_near = None;
        let mut best_near_d2 = (STORM_FLASH_RADIUS * 0.45).powi(2);
        let mut weakest = 0usize;
        let mut weakest_i = f32::MAX;
        for (i, slot) in self.flashes.iter().enumerate() {
            if slot.2 < weakest_i {
                weakest_i = slot.2;
                weakest = i;
            }
            if slot.2 > 0.01 {
                let dx = slot.0 - x;
                let dy = slot.1 - y;
                let d2 = dx * dx + dy * dy;
                if d2 < best_near_d2 {
                    best_near_d2 = d2;
                    best_near = Some(i);
                }
            }
        }
        let idx = best_near.unwrap_or(weakest);
        let cur = self.flashes[idx].2;
        self.flashes[idx] = (x, y, cur.max(intensity), STORM_FLASH_RADIUS);
    }

    /// Clear zones: base pocket + powered totems. Each is (cx, cy, radius).
    fn clear_zones(&self, world: &World) -> Vec<(f32, f32, f32)> {
        let mut zones = vec![(self.cx, self.cy, self.radius)];
        for n in world.nodes.values() {
            if n.kind == BuildingKind::Totem && n.powered {
                let (cx, cy) = n.center();
                zones.push((cx, cy, TOTEM_CLEAR_RADIUS));
            }
        }
        zones
    }

    /// Hard gameplay clear check — stable circles, ignores animated fog.
    fn in_clear(&self, wx: f32, wy: f32, zones: &[(f32, f32, f32)]) -> bool {
        zones.iter().any(|&(cx, cy, radius)| {
            let r = radius * STORM_HARD_CLEAR_SCALE;
            if r < 1.0 {
                return false;
            }
            let dx = wx - cx;
            let dy = wy - cy;
            dx * dx + dy * dy <= r * r
        })
    }

    fn point_in_storm(&self, wx: f32, wy: f32, world: &World) -> bool {
        let zones = self.clear_zones(world);
        !self.in_clear(wx, wy, &zones)
    }
}

fn storm_hash01(seed: f32) -> f32 {
    let x = (seed * 12.9898).sin() * 43758.5453;
    x.fract().abs()
}

fn create_storm_material() -> Option<Material> {
    let pipeline_params = PipelineParams {
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };
    load_material(
        ShaderSource::Glsl {
            vertex: STORM_VERTEX,
            fragment: STORM_FRAGMENT,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("ScreenSize", UniformType::Float2),
                UniformDesc::new("CamPos", UniformType::Float2),
                UniformDesc::new("CamZoom", UniformType::Float1),
                UniformDesc::new("Time", UniformType::Float1),
                UniformDesc::array(
                    UniformDesc::new("Totems", UniformType::Float4),
                    STORM_MAX_TOTEMS,
                ),
                UniformDesc::array(
                    UniformDesc::new("Flashes", UniformType::Float4),
                    STORM_MAX_FLASHES,
                ),
                UniformDesc::new("PixelSize", UniformType::Float1),
            ],
            pipeline_params,
            ..Default::default()
        },
    )
    .map_err(|e| {
        eprintln!("storm nebula shader unavailable, using CPU fog: {e:?}");
        e
    })
    .ok()
}

fn create_lighting_material() -> Option<Material> {
    let pipeline_params = PipelineParams {
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };
    load_material(
        ShaderSource::Glsl {
            vertex: STORM_VERTEX,
            fragment: LIGHT_FRAGMENT,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("ScreenSize", UniformType::Float2),
                UniformDesc::new("CamPos", UniformType::Float2),
                UniformDesc::new("CamZoom", UniformType::Float1),
                UniformDesc::new("Ambient", UniformType::Float1),
                UniformDesc::array(UniformDesc::new("Lights", UniformType::Float4), LIGHT_MAX),
            ],
            pipeline_params,
            ..Default::default()
        },
    )
    .map_err(|e| {
        eprintln!("world lighting shader unavailable: {e:?}");
        e
    })
    .ok()
}

fn create_ground_material() -> Option<Material> {
    load_material(
        ShaderSource::Glsl {
            vertex: STORM_VERTEX,
            fragment: GROUND_FRAGMENT,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("ScreenSize", UniformType::Float2),
                UniformDesc::new("CamPos", UniformType::Float2),
                UniformDesc::new("CamZoom", UniformType::Float1),
                UniformDesc::new("Time", UniformType::Float1),
                UniformDesc::array(
                    UniformDesc::new("Totems", UniformType::Float4),
                    STORM_MAX_TOTEMS,
                ),
            ],
            ..Default::default()
        },
    )
    .map_err(|e| {
        eprintln!("ground shader unavailable, using flat BG: {e:?}");
        e
    })
    .ok()
}

#[derive(Clone, Copy)]
enum ContextTarget {
    Empty,
}

/// In-progress power wire: start port + corner anchors (straight segments).
#[derive(Clone, Debug)]
struct WirePaint {
    from: (u32, usize),
    /// Includes start port world pos; anchors appended by clicks.
    points: Vec<(f32, f32)>,
}

#[derive(Clone, Copy)]
struct ContextMenu {
    sx: f32,
    sy: f32,
    target: ContextTarget,
}

struct Ui {
    build_open: bool,
    /// `None` = All categories.
    build_category: Option<BuildCategory>,
    build_search: String,
    build_search_focus: bool,
    build_scroll: f32,
    /// Drain the open-key char so it doesn't land in search.
    suppress_search_chars: bool,
    selected: Option<BuildingKind>,
    /// When `selected == Machine` (or hotbar Machine), which Era def.
    selected_machine: Option<u16>,
    hotbar: [Option<BuildingKind>; HOTBAR_SLOTS],
    hotbar_machine: [Option<u16>; HOTBAR_SLOTS],
    hotbar_index: usize,
    place_facing: Facing,
    wire_from: Option<(u32, usize)>,
    /// Click-to-route power wire (start port + corner anchors).
    wire_paint: Option<WirePaint>,
    /// Last tile painted while drag-placing belts (avoids re-paint spam).
    belt_paint_last: Option<(i32, i32)>,
    drag_node: Option<u32>,
    drag_off: (f32, f32),
    panning: bool,
    pan_last: (f32, f32),
    /// Dragging a building from the build menu onto the hotbar (Factorio-style).
    palette_drag: Option<BuildingKind>,
    palette_drag_machine: Option<u16>,
    palette_drag_origin: (f32, f32),
    /// Rearranging / clearing a hotbar slot by drag.
    hotbar_drag_from: Option<usize>,
    hotbar_drag_origin: (f32, f32),
    context_menu: Option<ContextMenu>,
    /// Non-build corner-wheel panels (tech / map / node chart).
    overlay: Option<CornerTool>,
    /// Factorio-style player inventory panel (Tab / E).
    inventory_open: bool,
    /// Recipe tree camera (graph space).
    recipe_cam_x: f32,
    recipe_cam_y: f32,
    recipe_zoom: f32,
    recipe_panning: bool,
    recipe_pan_last: (f32, f32),
    /// Fit camera once when opening the recipes overlay.
    recipe_fit_pending: bool,
    /// `false` = Helmod nested tree (default); `true` = full dependency web.
    recipe_view_web: bool,
    /// Root product for the nested production tree.
    recipe_root_item: Option<u16>,
    recipe_search: String,
    recipe_search_focus: bool,
    recipe_scroll: f32,
}

impl Ui {
    fn new() -> Self {
        Self {
            build_open: false,
            build_category: None,
            build_search: String::new(),
            build_search_focus: true,
            build_scroll: 0.0,
            suppress_search_chars: false,
            selected: None,
            selected_machine: None,
            hotbar: [None; HOTBAR_SLOTS],
            hotbar_machine: [None; HOTBAR_SLOTS],
            hotbar_index: 0,
            place_facing: Facing::E,
            wire_from: None,
            wire_paint: None,
            belt_paint_last: None,
            drag_node: None,
            drag_off: (0.0, 0.0),
            panning: false,
            pan_last: (0.0, 0.0),
            palette_drag: None,
            palette_drag_machine: None,
            palette_drag_origin: (0.0, 0.0),
            hotbar_drag_from: None,
            hotbar_drag_origin: (0.0, 0.0),
            context_menu: None,
            overlay: None,
            inventory_open: false,
            recipe_cam_x: 0.0,
            recipe_cam_y: 0.0,
            recipe_zoom: 1.0,
            recipe_panning: false,
            recipe_pan_last: (0.0, 0.0),
            recipe_fit_pending: true,
            recipe_view_web: false,
            recipe_root_item: None,
            recipe_search: String::new(),
            recipe_search_focus: true,
            recipe_scroll: 0.0,
        }
    }

    fn clear_tool(&mut self) {
        self.selected = None;
        self.selected_machine = None;
        self.wire_from = None;
        self.wire_paint = None;
        self.belt_paint_last = None;
        self.palette_drag = None;
        self.palette_drag_machine = None;
        self.hotbar_drag_from = None;
    }

    fn select_entry(&mut self, entry: BuildEntry) {
        self.selected = Some(entry.kind());
        self.selected_machine = entry.machine_id();
    }

    fn current_entry(&self) -> Option<BuildEntry> {
        let kind = self.selected?;
        if let Some(mid) = self.selected_machine {
            Some(BuildEntry::Era(mid))
        } else {
            Some(BuildEntry::Kind(kind))
        }
    }

    fn set_hotbar_entry(&mut self, i: usize, entry: Option<BuildEntry>) {
        if i >= HOTBAR_SLOTS {
            return;
        }
        match entry {
            Some(e) => {
                self.hotbar[i] = Some(e.kind());
                self.hotbar_machine[i] = e.machine_id();
            }
            None => {
                self.hotbar[i] = None;
                self.hotbar_machine[i] = None;
            }
        }
    }

    fn hotbar_entry(&self, i: usize) -> Option<BuildEntry> {
        let _kind = self.hotbar.get(i).copied().flatten()?;
        if let Some(mid) = self.hotbar_machine.get(i).copied().flatten() {
            Some(BuildEntry::Era(mid))
        } else {
            self.hotbar.get(i).copied().flatten().map(BuildEntry::Kind)
        }
    }

    fn palette_entry(&self) -> Option<BuildEntry> {
        let kind = self.palette_drag?;
        if let Some(mid) = self.palette_drag_machine {
            Some(BuildEntry::Era(mid))
        } else {
            Some(BuildEntry::Kind(kind))
        }
    }

    fn set_palette_entry(&mut self, entry: Option<BuildEntry>) {
        match entry {
            Some(e) => {
                self.palette_drag = Some(e.kind());
                self.palette_drag_machine = e.machine_id();
            }
            None => {
                self.palette_drag = None;
                self.palette_drag_machine = None;
            }
        }
    }

    fn filtered_entries(&self, tech: &content::TechState, creative: bool) -> Vec<BuildEntry> {
        let mut out: Vec<BuildEntry> = Vec::new();
        for &k in core_build_kinds() {
            if self.build_category.is_none() || Some(k.category()) == self.build_category {
                out.push(BuildEntry::Kind(k));
            }
        }
        // Debug spawn tools — Creative only (or Debug category while Creative).
        if creative
            && (self.build_category == Some(BuildCategory::Debug)
                || (self.build_category.is_none() && !self.build_search.trim().is_empty()))
        {
            for k in BuildingKind::DEBUG_TOOLS {
                if k.matches_query(&self.build_search) {
                    out.push(BuildEntry::Kind(k));
                }
            }
        }
        // All placeable Era machines from the content pack.
        for m in &content::content().machines {
            if !m.placeable {
                continue;
            }
            let entry = BuildEntry::Era(m.index);
            if let Some(cat) = self.build_category {
                if cat == BuildCategory::Debug || entry.category() != cat {
                    continue;
                }
            }
            out.push(entry);
        }
        out.into_iter()
            .filter(|e| e.matches_query(&self.build_search))
            .filter(|e| {
                creative || tech.machine_unlocked(&e.tech_unlock())
            })
            .collect()
    }

    fn open_build(&mut self) {
        self.build_open = true;
        self.inventory_open = false;
        self.overlay = None;
        self.wire_from = None;
        self.context_menu = None;
        self.drag_node = None;
        self.build_search_focus = true;
        self.build_scroll = 0.0;
        self.suppress_search_chars = true;
    }

    fn close_build(&mut self) {
        self.build_open = false;
        self.palette_drag = None;
        self.palette_drag_machine = None;
        self.build_search_focus = false;
    }

    fn toggle_build(&mut self) {
        if self.build_open {
            self.close_build();
        } else {
            self.open_build();
        }
    }

    fn open_inventory(&mut self) {
        self.inventory_open = true;
        self.close_build();
        self.overlay = None;
        self.context_menu = None;
        self.clear_tool();
    }

    fn close_inventory(&mut self) {
        self.inventory_open = false;
    }

    fn toggle_inventory(&mut self) {
        if self.inventory_open {
            self.close_inventory();
        } else {
            self.open_inventory();
        }
    }

    fn activate_corner(&mut self, tool: CornerTool) {
        match tool {
            CornerTool::Build => self.toggle_build(),
            other => {
                self.close_build();
                self.close_inventory();
                if self.overlay == Some(other) {
                    self.overlay = None;
                } else {
                    self.overlay = Some(other);
                    if other == CornerTool::Recipes {
                        self.recipe_fit_pending = true;
                        self.recipe_panning = false;
                        self.recipe_scroll = 0.0;
                        self.recipe_search_focus = true;
                        if self.recipe_root_item.is_none() {
                            let reg = content::content();
                            self.recipe_root_item = reg
                                .item_index("era1_logistics_green_wire")
                                .or_else(|| reg.item_index("era1_component_basic_circuit"));
                        }
                    }
                }
                self.context_menu = None;
            }
        }
    }
}

struct LightningFx {
    /// Main bolt polyline in world space.
    points: Vec<(f32, f32)>,
    /// Optional side branches.
    branches: Vec<Vec<(f32, f32)>>,
    life: f32,
    max_life: f32,
    width: f32,
}

struct App {
    screen: Screen,
    /// Where Settings returns to when closed.
    settings_return: Screen,
    settings_category: SettingsCategory,
    world: World,
    cam: Cam,
    ui: Ui,
    storm: Storm,
    art: art::Art,
    /// Soft world lighting (poles / working machines). Local FX only.
    lighting: Option<Material>,
    /// Dark stormglass crust (pixel mosaic; weathers inside clear zones).
    ground: Option<Material>,
    /// Charge / bolt plasma (storm-quality fullscreen pass).
    cannon_fx: Option<Material>,
    /// Rising gas plumes from vents (storm-quality mosaic).
    gas_fx: Option<Material>,
    settings: Settings,
    pause_open: bool,
    autosave_timer: f32,
    /// Accumulator for fixed 60 UPS world steps.
    sim_accum: f32,
    /// Displayed UPS — updated once per second from a step counter (stable, not per-frame noise).
    measured_ups: f32,
    ups_window_steps: u32,
    ups_window_start: Instant,
    lightning_cd: f32,
    lightning_fx: Vec<LightningFx>,
    status_toast: String,
    load_scroll: f32,
    load_selected: Option<usize>,
    load_preview_tex: Option<Texture2D>,
    /// Cached list for load screen (refreshed on enter / delete).
    load_list: Vec<save::SaveInfo>,
    /// Load screen opens for solo continue vs host-from-save.
    load_for_host: bool,
    play_intent: MenuPlayIntent,
    /// Unpaused playtime for the current session (seconds).
    play_seconds: f32,
    /// Last clean world screenshot (pre-HUD), used as save preview.
    world_preview: Option<Image>,
    /// Capture framebuffer after world draw (before HUD) on the next frame.
    pending_preview_capture: bool,
    /// If set, write preview PNG here after the deferred capture.
    pending_preview_save_path: Option<std::path::PathBuf>,
    net: Option<NetHandle>,
    peers: HashMap<u8, PeerPresence>,
    host_code: String,
    host_addr: String,
    join_code: String,
    join_focus: bool,
    join_status: String,
    last_cursor_send: Instant,
    last_cursor_x: f32,
    last_cursor_y: f32,
    cursor_clock: Instant,
    local_player_id: u8,
    last_snap_send: Instant,
    applying_snap: bool,
    player: Player,
    /// Local player materials (spend-on-place).
    inventory: Inventory,
    /// Host-tracked inventories for remote players (and mirror of local for host).
    peer_inventories: HashMap<u8, Inventory>,
    /// Survival (gated) vs Creative (free / unlocked).
    game_mode: GameMode,
}

impl App {
    async fn new() -> Self {
        let settings = Settings::load();
        Self {
            screen: Screen::Main,
            settings_return: Screen::Main,
            settings_category: SettingsCategory::Display,
            world: World::new(),
            cam: Cam {
                x: 0.0,
                y: 0.0,
                zoom: 1.0,
            },
            ui: Ui::new(),
            storm: Storm::new(None),
            art: art::Art::load().await,
            lighting: None,
            ground: None,
            cannon_fx: None,
            gas_fx: None,
            settings,
            pause_open: false,
            autosave_timer: 0.0,
            sim_accum: 0.0,
            measured_ups: TARGET_UPS as f32,
            ups_window_steps: 0,
            ups_window_start: Instant::now(),
            lightning_cd: 1.5,
            lightning_fx: Vec::new(),
            status_toast: String::new(),
            load_scroll: 0.0,
            load_selected: None,
            load_preview_tex: None,
            load_list: Vec::new(),
            load_for_host: false,
            play_intent: MenuPlayIntent::Solo,
            play_seconds: 0.0,
            world_preview: None,
            pending_preview_capture: false,
            pending_preview_save_path: None,
            net: None,
            peers: HashMap::new(),
            host_code: String::new(),
            host_addr: String::new(),
            join_code: String::new(),
            join_focus: false,
            join_status: String::new(),
            last_cursor_send: Instant::now(),
            last_cursor_x: f32::NAN,
            last_cursor_y: f32::NAN,
            cursor_clock: Instant::now(),
            local_player_id: 0,
            last_snap_send: Instant::now(),
            applying_snap: false,
            player: Player::new(0.0, 0.0),
            inventory: Inventory::starter(),
            peer_inventories: HashMap::new(),
            game_mode: GameMode::Survival,
        }
    }

    fn is_single_player(&self) -> bool {
        self.net.is_none()
    }

    fn is_creative(&self) -> bool {
        self.game_mode.is_creative()
    }

    fn open_settings(&mut self, from: Screen) {
        self.settings_return = from;
        self.settings_category = SettingsCategory::Display;
        self.screen = Screen::Settings;
    }

    fn enter_game_common(&mut self) {
        self.screen = Screen::Game;
        self.pause_open = false;
        self.ui = Ui::new();
        self.peers.clear();
        self.autosave_timer = 0.0;
        self.sim_accum = 0.0;
        self.measured_ups = TARGET_UPS as f32;
        self.ups_window_steps = 0;
        self.ups_window_start = Instant::now();
        self.pending_preview_capture = false;
        self.pending_preview_save_path = None;
        if let Some(net) = self.net.as_ref() {
            let _ = net.tx.send(NetCommand::Announce);
            if !net.is_host {
                let _ = net.tx.send(NetCommand::WantSnap);
            }
        }
    }

    fn enter_new_singleplayer(&mut self, mode: GameMode) {
        self.stop_net();
        self.game_mode = mode;
        self.play_seconds = 0.0;
        self.world_preview = None;
        self.world.clear();
        self.world.tech = content::TechState::default();
        if mode.is_creative() {
            self.world.tech.debug_unlock_all();
        }
        self.world
            .seed_nests(self.storm.cx, self.storm.cy, self.storm.radius);
        self.world
            .seed_deposits(self.storm.cx, self.storm.cy, self.storm.radius);
        self.cam = Cam {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        };
        self.player = Player::new(0.0, 0.0);
        self.inventory = Inventory::starter();
        self.peer_inventories.clear();
        self.enter_game_common();
        self.status_toast = format!("{} mode", mode.label());
    }

    fn enter_from_save(&mut self, save: &save::GameSave) -> Result<(), String> {
        self.stop_net();
        apply_save(&mut self.world, save)?;
        if self.world.nests.is_empty() {
            self.world
                .seed_nests(self.storm.cx, self.storm.cy, self.storm.radius);
        }
        if self.world.veins.is_empty() {
            self.world
                .seed_deposits(self.storm.cx, self.storm.cy, self.storm.radius);
        }
        self.cam = Cam {
            x: save.cam_x,
            y: save.cam_y,
            zoom: save.cam_zoom.clamp(MIN_ZOOM, MAX_ZOOM),
        };
        self.player = Player::new(save.cam_x, save.cam_y);
        self.inventory = match (save.inv_ore, save.inv_ingot) {
            (Some(ore), Some(ingot)) => Inventory::from_totals(ore, ingot),
            _ => Inventory::starter(),
        };
        self.peer_inventories.clear();
        self.game_mode = save.game_mode;
        self.play_seconds = save.play_seconds.max(0.0);
        self.world_preview = None;
        if self.game_mode.is_creative() {
            self.world.tech.debug_unlock_all();
        }
        self.enter_game_common();
        for (i, k) in save.hotbar.iter().enumerate() {
            self.ui.hotbar[i] = k.and_then(|v| BuildingKind::from_u8(v));
        }
        self.ui.hotbar_index = save.hotbar_index.min(HOTBAR_SLOTS - 1);
        self.ui.selected = self.ui.hotbar[self.ui.hotbar_index];
        Ok(())
    }

    fn capture_current_save(&self, label: &str) -> save::GameSave {
        capture_save(
            &self.world,
            self.cam.x,
            self.cam.y,
            self.cam.zoom,
            &self.ui.hotbar,
            self.ui.hotbar_index,
            self.inventory.ore(),
            self.inventory.ingot(),
            label,
            self.game_mode,
            self.play_seconds,
        )
    }

    fn do_manual_save(&mut self) {
        let save = self.capture_current_save("Manual Save");
        match write_manual_save(&save) {
            Ok(path) => {
                self.queue_preview_write(path);
                self.status_toast = "Game saved".into();
            }
            Err(e) => self.status_toast = format!("Save failed: {e}"),
        }
    }

    fn do_autosave(&mut self) {
        if !self.is_single_player() {
            return;
        }
        let mut save = self.capture_current_save("Autosave");
        match write_autosave(&mut self.settings, &mut save) {
            Ok(path) => {
                self.queue_preview_write(path);
                self.status_toast = format!("Autosaved ({})", save.label);
            }
            Err(e) => self.status_toast = format!("Autosave failed: {e}"),
        }
    }

    fn queue_preview_write(&mut self, path: std::path::PathBuf) {
        // Always refresh on save — readback runs after world draw on this/next frame.
        self.pending_preview_capture = true;
        self.pending_preview_save_path = Some(path);
    }

    fn open_load_game(&mut self) {
        self.load_for_host = false;
        self.open_load_game_inner();
    }

    fn open_load_game_for_host(&mut self) {
        self.load_for_host = true;
        self.open_load_game_inner();
    }

    fn open_load_game_inner(&mut self) {
        self.load_scroll = 0.0;
        self.load_list = list_saves();
        self.load_selected = if self.load_list.is_empty() {
            None
        } else {
            Some(0)
        };
        self.load_preview_tex = None;
        self.refresh_load_preview();
        self.screen = Screen::LoadGame;
    }

    fn begin_host_session(&mut self) {
        self.join_status = "Setting up session…".into();
        let handle = net::start_host();
        self.host_code = handle.code.clone();
        self.host_addr.clear();
        self.net = Some(handle);
        self.screen = Screen::HostLobby;
    }

    /// Promote the current solo session to multiplayer without leaving the world.
    fn promote_solo_to_multiplayer(&mut self) {
        if self.net.is_some() {
            return;
        }
        self.join_status = "Setting up session…".into();
        let handle = net::start_host();
        self.host_code = handle.code.clone();
        self.host_addr.clear();
        self.net = Some(handle);
        self.status_toast = "Multiplayer started — tap the code to copy".into();
    }

    fn copy_session_code(&mut self) {
        if self.host_code.is_empty() {
            self.status_toast = "No session code yet".into();
            return;
        }
        if ui_chrome::copy_to_clipboard(&self.host_code) {
            self.status_toast = "Code copied".into();
        } else {
            self.status_toast = "Could not copy code".into();
        }
    }

    fn kick_peer(&mut self, id: u8) {
        let Some(net) = self.net.as_ref() else {
            return;
        };
        if !net.is_host || id == self.local_player_id {
            return;
        }
        let _ = net.tx.send(NetCommand::Kick { id });
        self.peers.remove(&id);
        self.peer_inventories.remove(&id);
        self.status_toast = format!("Kicked player {id}");
    }

    fn enter_new_host(&mut self, mode: GameMode) {
        self.stop_net();
        self.game_mode = mode;
        self.play_seconds = 0.0;
        self.world_preview = None;
        self.world.clear();
        self.world.tech = content::TechState::default();
        if mode.is_creative() {
            self.world.tech.debug_unlock_all();
        }
        self.world
            .seed_nests(self.storm.cx, self.storm.cy, self.storm.radius);
        self.world
            .seed_deposits(self.storm.cx, self.storm.cy, self.storm.radius);
        self.cam = Cam {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        };
        self.player = Player::new(0.0, 0.0);
        self.inventory = Inventory::starter();
        self.peer_inventories.clear();
        self.begin_host_session();
        self.status_toast = format!("Hosting · {}", mode.label());
    }

    fn refresh_load_preview(&mut self) {
        self.load_preview_tex = None;
        let Some(i) = self.load_selected else {
            return;
        };
        let Some(info) = self.load_list.get(i) else {
            return;
        };
        let Some(path) = info.preview_path.as_ref() else {
            return;
        };
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(img) = Image::from_file_with_format(&bytes, Some(ImageFormat::Png)) {
                self.load_preview_tex = Some(Texture2D::from_image(&img));
            }
        }
    }

    fn write_preview_for(&self, save_json: &std::path::Path) {
        let Some(img) = &self.world_preview else {
            return;
        };
        let path = preview_path_for(save_json);
        if let Err(e) = save_preview_png(&path, img) {
            eprintln!("preview write failed: {e}");
        }
    }

    fn flush_pending_preview_capture(&mut self) {
        if !self.pending_preview_capture {
            return;
        }
        self.pending_preview_capture = false;
        if let Some(img) = capture_preview_now() {
            self.world_preview = Some(img);
        }
        if let Some(path) = self.pending_preview_save_path.take() {
            self.write_preview_for(&path);
        }
    }

    fn return_to_main_menu(&mut self) {
        self.stop_net();
        self.pause_open = false;
        self.ui = Ui::new();
        self.world.clear();
        self.lightning_fx.clear();
        self.lightning_cd = 1.5;
        self.screen = Screen::Main;
    }

    fn stop_net(&mut self) {
        if let Some(net) = self.net.take() {
            let _ = net.tx.send(NetCommand::Stop);
        }
        self.peers.clear();
        self.peer_inventories.clear();
        self.host_code.clear();
        self.host_addr.clear();
        self.join_status.clear();
    }
}

fn window_conf() -> Conf {
    let settings = Settings::load();
    Conf {
        window_title: "FactoryPlanner".to_owned(),
        window_width: settings.window_w,
        window_height: settings.window_h,
        fullscreen: !settings.display_mode.is_windowed(),
        high_dpi: false,
        platform: miniquad::conf::Platform {
            swap_interval: Some(if settings.vsync { 1 } else { 0 }),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    match content::init_content() {
        Ok(reg) => {
            println!(
                "{}: {} items, {} fluids, {} recipes, {} machines, {} techs",
                reg.era_name,
                reg.stats.items,
                reg.stats.fluids,
                reg.stats.recipes,
                reg.stats.machines,
                reg.stats.technologies
            );
        }
        Err(e) => {
            eprintln!("FATAL: failed to load Era 1 content packs: {e}");
            panic!("Era 1 content load failed: {e}");
        }
    }
    let mut app = App::new().await;
    app.storm.material = create_storm_material();
    app.lighting = create_lighting_material();
    app.ground = create_ground_material();
    app.cannon_fx = create_cannon_material();
    app.gas_fx = create_gas_material();
    if app.storm.material.is_none() {
        app.status_toast = "GPU fog shader unavailable — using lightweight fallback".into();
    }
    app.settings.apply_runtime();
    let mut perf = perf_log::PerfLog::start(&app.settings);
    if app.storm.material.is_none() {
        perf.note("storm GPU shader missing — CPU/lightweight fog path");
    }
    if app.ground.is_none() {
        perf.note("ground shader missing");
    }
    eprintln!(
        "Perf log: {} (send this file when reporting low FPS)",
        perf_log::log_path().display()
    );
    // Wall clock for sim pacing — `get_frame_time` jitters with vsync/sleep and lies about UPS.
    let mut prev_wall = Instant::now();

    loop {
        let now = Instant::now();
        let wall_dt = (now - prev_wall).as_secs_f32().clamp(0.0, 0.05);
        prev_wall = now;
        // Menus / FX can use wall_dt; keep a soft alias for existing call sites.
        let frame_dt = wall_dt;
        let frame_start = now;
        let mouse = ui_chrome::pointer();
        perf.set_screen(app.screen.as_str());

        match app.screen {
            Screen::Main => screen_main(&mut app, mouse, frame_dt),
            Screen::SinglePlayer => screen_single_player(&mut app, mouse, frame_dt),
            Screen::Play => screen_play(&mut app, mouse, frame_dt),
            Screen::Multiplayer => screen_multiplayer(&mut app, mouse, frame_dt),
            Screen::HostSetup => screen_host_setup(&mut app, mouse, frame_dt),
            Screen::HostLobby => screen_host_lobby(&mut app, mouse, frame_dt),
            Screen::JoinLobby => screen_join_lobby(&mut app, mouse, frame_dt),
            Screen::Settings => screen_settings(&mut app, mouse, frame_dt),
            Screen::LoadGame => screen_load_game(&mut app, mouse, frame_dt),
            Screen::Game => {
                drain_net(&mut app);
                maybe_host_snapshot(&mut app);
                let (wx, wy) = app.cam.screen_to_world(mouse.0, mouse.1);
                handle_hotkeys(&mut app, wx, wy);
                if !app.pause_open {
                    handle_build_search_input(&mut app.ui);
                }
                if !app.pause_open {
                    handle_pan_zoom(&mut app, mouse);
                    handle_hud_input(&mut app, mouse, wx, wy);
                    if !app.ui.build_open
                        && !app.ui.inventory_open
                        && app.ui.context_menu.is_none()
                        && app.ui.overlay.is_none()
                    {
                        handle_world_input(&mut app, mouse, wx, wy);
                    }
                    // Fixed 60 UPS — accumulate real wall time, not noisy get_frame_time.
                    app.sim_accum += wall_dt;
                    app.play_seconds += wall_dt;
                    let mut steps = 0u32;
                    // Sample wish once per frame so multi-step catch-up doesn't re-read keys.
                    let wish = if app.ui.build_open
                        || app.ui.inventory_open
                        || app.ui.overlay.is_some()
                        || app.ui.context_menu.is_some()
                    {
                        Vec2::ZERO
                    } else {
                        player::movement_wish()
                    };
                    while app.sim_accum >= FIXED_DT && steps < MAX_SIM_STEPS {
                        app.sim_accum -= FIXED_DT;
                        app.player.tick(FIXED_DT, wish);
                        {
                            let zones = app.storm.clear_zones(&app.world);
                            resolve_player_crack_collision(
                                &mut app.player,
                                &app.world,
                                &app.storm,
                                &app.art,
                                &zones,
                            );
                        }
                        // Peers advance on the same UPS clock as local motion.
                        advance_peer_cursors(&mut app, FIXED_DT);
                        // Broadcast pose every sim tick so remotes match 60 UPS.
                        send_cursor_ups(&mut app, wx, wy);
                        let zones = app.storm.clear_zones(&app.world);
                        app.world.tick(FIXED_DT, &zones);
                        let report = app.world.combat_step(FIXED_DT, &zones);
                        if let Some(tid) = app.world.tech_completed.take() {
                            app.status_toast = format!("Researched: {tid}");
                        } else if app.world.era1_complete
                            && app.status_toast != "Planetary Fabrication Nexus commissioned — Era 1 complete!"
                        {
                            // One-shot milestone banner.
                            if app.world.tech.nexus_complete {
                                app.status_toast =
                                    "Planetary Fabrication Nexus commissioned — Era 1 complete!"
                                        .into();
                            }
                        }
                        if report.nests_reawakened > 0 {
                            app.status_toast = "Hateful nest reawakened — bigger swarm!".into();
                        } else if report.nests_revealed > 0 {
                            app.status_toast = if report.nests_revealed == 1 {
                                "Nest revealed! Swarm incoming — fortify!".into()
                            } else {
                                format!(
                                    "{} nests revealed! Swarms incoming!",
                                    report.nests_revealed
                                )
                            };
                        } else if report.waves_launched > 0 {
                            let tier = app
                                .world
                                .nests
                                .iter()
                                .filter(|n| n.active)
                                .map(|n| n.threat_tier())
                                .next()
                                .unwrap_or("Raiders");
                            app.status_toast = format!("Enemy wave — threat: {tier}");
                        } else if report.destroyed > 0 {
                            app.status_toast = if report.destroyed == 1 {
                                "Raiders destroyed a building!".into()
                            } else {
                                format!("Raiders destroyed {} buildings!", report.destroyed)
                            };
                        }
                        tick_storm_lightning(&mut app, FIXED_DT);
                        // Storm fog time tracks sim, not wall/FPS.
                        app.storm.tick(FIXED_DT);
                        steps += 1;
                    }
                    // Soft clamp debt — recover from hitches without spiral-of-death freezes.
                    if app.sim_accum > MAX_SIM_DEBT {
                        app.sim_accum = MAX_SIM_DEBT;
                    }
                    // Rolling 1s UPS window — never flicker from 0/1 steps-per-frame noise.
                    app.ups_window_steps += steps;
                    let window_t = app.ups_window_start.elapsed().as_secs_f32();
                    if window_t >= 1.0 {
                        app.measured_ups = app.ups_window_steps as f32 / window_t;
                        app.ups_window_steps = 0;
                        app.ups_window_start = Instant::now();
                    }
                    // Camera follow / leash after sim (and after MMB pan).
                    match app.player.cam_mode {
                        CamMode::Follow => {
                            player::snap_cam_to_player(
                                &mut app.cam.x,
                                &mut app.cam.y,
                                app.player.x,
                                app.player.y,
                            );
                        }
                        CamMode::Free => {
                            player::clamp_cam_to_player(
                                &mut app.cam.x,
                                &mut app.cam.y,
                                app.cam.zoom,
                                app.player.x,
                                app.player.y,
                            );
                        }
                    }
                    app.cam.quantize();
                    if app.is_single_player() {
                        app.autosave_timer += wall_dt;
                        if app.autosave_timer >= AUTOSAVE_INTERVAL_SECS {
                            app.autosave_timer = 0.0;
                            app.do_autosave();
                        }
                    }
                } else {
                    // Paused: fog still drifts visually on wall time.
                    app.storm.tick(frame_dt);
                    app.cam.quantize();
                }
                // Refresh mouse world after cam lock/quantize so HUD text doesn't shimmer.
                let (wx, wy) = app.cam.screen_to_world(mouse.0, mouse.1);
                // Lightning bolts animate at render rate.
                tick_lightning_fx(&mut app, frame_dt);
                draw_game(&mut app, mouse, wx, wy);
                if app.pause_open {
                    draw_and_handle_pause_menu(&mut app, mouse);
                }
                if !app.status_toast.is_empty() {
                    ui_chrome::toast_bar(&app.status_toast);
                }
            }
        }

        next_frame().await;
        let spent = frame_start.elapsed();
        let frame_ms = spent.as_secs_f32() * 1000.0;
        let fps = get_fps() as f32;
        perf.frame(
            frame_ms,
            fps,
            app.measured_ups,
            app.world.nodes.len(),
            app.world.belt_tiles.len(),
            app.peers.len(),
            app.settings.effect_quality,
            app.settings.fps_limit,
        );
        if let Some(budget) = app.settings.fps_limit.frame_budget() {
            if spent < budget {
                std::thread::sleep(budget - spent);
            }
        }
    }
}

fn peer_color(id: u8) -> Color {
    const PALETTE: [Color; 8] = [
        Color::from_rgba(255, 120, 100, 255),
        Color::from_rgba(100, 200, 255, 255),
        Color::from_rgba(180, 255, 120, 255),
        Color::from_rgba(255, 180, 80, 255),
        Color::from_rgba(200, 140, 255, 255),
        Color::from_rgba(80, 220, 180, 255),
        Color::from_rgba(255, 100, 180, 255),
        Color::from_rgba(160, 180, 255, 255),
    ];
    PALETTE[id as usize % PALETTE.len()]
}

fn button(label: &str, x: f32, y: f32, w: f32, h: f32, mouse: (f32, f32)) -> bool {
    ui_chrome::button(label, x, y, w, h, mouse)
}

fn button_primary(label: &str, x: f32, y: f32, w: f32, h: f32, mouse: (f32, f32)) -> bool {
    ui_chrome::button_styled(label, x, y, w, h, mouse, ButtonStyle::Primary)
}

fn menu_panel_geom(btn_count: usize) -> (f32, f32, f32, f32, f32) {
    let bw = 340.0;
    let bh = 50.0;
    let gap = 12.0;
    let title_block = 118.0;
    let total_h = title_block + btn_count as f32 * (bh + gap);
    let bx = (screen_width() - bw) * 0.5;
    let top = ((screen_height() - total_h) * 0.5).max(36.0);
    (bx, top, bw, bh, gap)
}

fn draw_menu_storm_backdrop_ex(app: &mut App, dt: f32, title: Option<(&str, &str)>) {
    app.storm.tick(dt);
    let cam = Cam {
        x: 80.0,
        y: -40.0,
        zoom: 0.28,
    };
    clear_background(BG);
    draw_ground(app, &cam, &[]);
    draw_infinite_grid(&cam);
    draw_storm(&app.storm, &[], &cam);
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(6, 8, 12, 150),
    );
    // Vignette bands
    draw_rectangle(0.0, 0.0, screen_width(), 90.0, Color::from_rgba(0, 0, 0, 70));
    draw_rectangle(
        0.0,
        screen_height() - 100.0,
        screen_width(),
        100.0,
        Color::from_rgba(0, 0, 0, 90),
    );
    if let Some((title, subtitle)) = title {
        let (_, top, _, _, _) = menu_panel_geom(5);
        ui_chrome::menu_title(title, subtitle, top);
    }
}

/// Top-of-menu brand mark. Temporary text until logo art replaces it.
fn draw_main_menu_logo(cx: f32, top: f32) -> f32 {
    let title = "FactoryPlanner";
    let tw = measure_text(title, None, 52, 1.0).width;
    let sx = cx - tw * 0.5;
    draw_rectangle(
        sx - 20.0,
        top + 4.0,
        tw + 40.0,
        72.0,
        Color::from_rgba(8, 12, 18, 140),
    );
    draw_text(title, sx, top + 48.0, 52.0, UI_CYAN);
    draw_rectangle(sx, top + 56.0, tw * 0.35, 3.0, UI_AMBER);
    top + 86.0
}

fn draw_main_menu_version() {
    let label = format!("v{}", env!("CARGO_PKG_VERSION"));
    let fs = 16.0;
    let tw = measure_text(&label, None, fs as u16, 1.0).width;
    draw_text(
        &label,
        screen_width() - tw - 18.0,
        screen_height() - 18.0,
        fs,
        TEXT_DIM,
    );
    let hint = format!("Perf log: {}", perf_log::log_path().display());
    draw_text(&hint, 18.0, screen_height() - 18.0, 14.0, TEXT_DIM);
}

fn screen_main(app: &mut App, mouse: (f32, f32), dt: f32) {
    draw_menu_storm_backdrop_ex(app, dt, None);

    let bw = 340.0;
    let bh = 50.0;
    let gap = 14.0;
    let bx = (screen_width() - bw) * 0.5;
    let cx = screen_width() * 0.5;

    // Logo → Single Player → Multiplayer → Settings → Exit
    let stack_h = 86.0 + bh + gap + bh + gap + bh + gap + bh;
    let mut y = ((screen_height() - stack_h) * 0.42).max(40.0);

    y = draw_main_menu_logo(cx, y);

    if button_primary("Single Player", bx, y, bw, bh, mouse) {
        app.screen = Screen::SinglePlayer;
    }
    y += bh + gap;

    if button("Multiplayer", bx, y, bw, bh, mouse) {
        app.screen = Screen::Multiplayer;
    }
    y += bh + gap;

    if button("Settings", bx, y, bw, bh, mouse) {
        app.open_settings(Screen::Main);
    }
    y += bh + gap;

    if ui_chrome::button_styled("Exit", bx, y, bw, bh, mouse, ButtonStyle::Danger) {
        perf_log::append_shutdown_note("user Exit");
        std::process::exit(0);
    }

    draw_main_menu_version();
}

fn screen_single_player(app: &mut App, mouse: (f32, f32), dt: f32) {
    draw_menu_storm_backdrop_ex(app, dt, None);
    let recent = save::most_recent_save();
    let rows = if recent.is_some() { 3 } else { 2 };
    let panel_h = 200.0 + rows as f32 * 64.0;
    let (px, py, panel_w, panel_h, pad, _) = titled_menu_panel("Single Player", 400.0, panel_h);
    let bw = panel_w - pad * 2.0;
    let bh = 50.0;
    let bx = px + pad;
    let mut by = py + pad + 8.0;

    if let Some(info) = recent.as_ref() {
        let label = format!("Continue — {}", info.label);
        if button_primary(&label, bx, by, bw, bh, mouse) {
            match read_save(&info.path) {
                Ok(save) => {
                    if let Err(e) = app.enter_from_save(&save) {
                        app.status_toast = e;
                    }
                }
                Err(e) => app.status_toast = e,
            }
        }
        by += bh + 14.0;
    }

    if button_primary("New Game", bx, by, bw, bh, mouse) {
        app.play_intent = MenuPlayIntent::Solo;
        app.screen = Screen::Play;
    }
    by += bh + 14.0;
    if button("Load Game", bx, by, bw, bh, mouse) {
        app.open_load_game();
    }
    by = py + panel_h - pad - bh;
    if button("Back", bx, by, bw, bh, mouse) {
        app.screen = Screen::Main;
    }
    let _ = dt;
}

fn screen_play(app: &mut App, mouse: (f32, f32), dt: f32) {
    draw_menu_storm_backdrop_ex(app, dt, None);
    let (px, py, panel_w, panel_h, pad, _) = titled_menu_panel("New Game", 440.0, 340.0);
    let bw = panel_w - pad * 2.0;
    let bh = 48.0;
    let bx = px + pad;
    let mut by = py + pad + 4.0;

    if button_primary("Survival", bx, by, bw, bh, mouse) {
        match app.play_intent {
            MenuPlayIntent::Solo => app.enter_new_singleplayer(GameMode::Survival),
            MenuPlayIntent::Host => app.enter_new_host(GameMode::Survival),
        }
    }
    by += bh + 6.0;
    draw_text(GameMode::Survival.blurb(), bx + 4.0, by + 14.0, 14.0, TEXT_DIM);
    by += 28.0;

    if button_primary("Creative", bx, by, bw, bh, mouse) {
        match app.play_intent {
            MenuPlayIntent::Solo => app.enter_new_singleplayer(GameMode::Creative),
            MenuPlayIntent::Host => app.enter_new_host(GameMode::Creative),
        }
    }
    by += bh + 6.0;
    draw_text(GameMode::Creative.blurb(), bx + 4.0, by + 14.0, 14.0, TEXT_DIM);

    by = py + panel_h - pad - bh;
    if button("Back", bx, by, bw, bh, mouse) {
        app.screen = match app.play_intent {
            MenuPlayIntent::Solo => Screen::SinglePlayer,
            MenuPlayIntent::Host => Screen::HostSetup,
        };
    }
}

fn screen_multiplayer(app: &mut App, mouse: (f32, f32), dt: f32) {
    draw_menu_storm_backdrop_ex(app, dt, None);
    let (px, py, panel_w, panel_h, pad, _) = titled_menu_panel("Multiplayer", 400.0, 300.0);
    let bw = panel_w - pad * 2.0;
    let bh = 50.0;
    let bx = px + pad;
    let mut by = py + pad + 8.0;

    draw_text(
        "Play with friends using a session code",
        bx,
        by + 4.0,
        15.0,
        TEXT_DIM,
    );
    by += 28.0;

    if button_primary("Host Game", bx, by, bw, bh, mouse) {
        app.screen = Screen::HostSetup;
    }
    by += bh + 14.0;
    if button("Join Game", bx, by, bw, bh, mouse) {
        app.stop_net();
        app.join_status.clear();
        app.join_code.clear();
        app.join_focus = true;
        app.screen = Screen::JoinLobby;
    }
    by = py + panel_h - pad - bh;
    if button("Back", bx, by, bw, bh, mouse) {
        app.screen = Screen::Main;
    }
    let _ = dt;
}

fn screen_host_setup(app: &mut App, mouse: (f32, f32), dt: f32) {
    draw_menu_storm_backdrop_ex(app, dt, None);
    let (px, py, panel_w, panel_h, pad, _) = titled_menu_panel("Host Game", 420.0, 300.0);
    let bw = panel_w - pad * 2.0;
    let bh = 50.0;
    let bx = px + pad;
    let mut by = py + pad + 8.0;

    draw_text(
        "Start a new world or open a save for others to join",
        bx,
        by + 4.0,
        14.0,
        TEXT_DIM,
    );
    by += 28.0;

    if button_primary("New World", bx, by, bw, bh, mouse) {
        app.play_intent = MenuPlayIntent::Host;
        app.screen = Screen::Play;
    }
    by += bh + 14.0;
    if button("Load Save", bx, by, bw, bh, mouse) {
        app.open_load_game_for_host();
    }
    by = py + panel_h - pad - bh;
    if button("Back", bx, by, bw, bh, mouse) {
        app.screen = Screen::Multiplayer;
    }
    let _ = dt;
}

fn screen_settings(app: &mut App, mouse: (f32, f32), dt: f32) {
    draw_menu_storm_backdrop_ex(app, dt, None);

    let panel_w = 460.0;
    let panel_h = 500.0;
    let footer_h = 52.0;
    let bh = 48.0;
    let gap = 12.0;

    let shell = ui_chrome::menu_shell("Settings", panel_w, panel_h, footer_h);

    let cat_labels: Vec<&str> = SettingsCategory::ALL
        .iter()
        .map(|c| c.label())
        .collect();
    if let Some(i) = ui_chrome::category_rail(
        &cat_labels,
        app.settings_category.index(),
        shell.cat_origin,
        mouse,
    ) {
        app.settings_category = SettingsCategory::from_index(i);
    }

    let bx = shell.content.x;
    let bw = shell.content.w;
    let mut by = shell.content.y;

    match app.settings_category {
        SettingsCategory::Display => {
            let mode_label = format!("Display mode: {}", app.settings.display_mode.label());
            if button(&mode_label, bx, by, bw, bh, mouse) {
                app.settings.display_mode = app.settings.display_mode.next();
            }
            by += bh + gap;

            if ui_chrome::checkbox_row(
                "Wait for VSync (restart)",
                app.settings.vsync,
                bx,
                by,
                bw,
                bh,
                mouse,
            ) {
                app.settings.vsync = !app.settings.vsync;
            }
            by += bh + gap;

            let windowed = app.settings.display_mode.is_windowed();
            let res_label = if windowed {
                format!(
                    "Resolution: {}×{}",
                    app.settings.window_w, app.settings.window_h
                )
            } else {
                "Resolution: (windowed only)".into()
            };
            if windowed && button(&res_label, bx, by, bw, bh, mouse) {
                let presets = [(1280, 720), (1400, 900), (1600, 900), (1920, 1080)];
                let idx = presets
                    .iter()
                    .position(|&(w, h)| w == app.settings.window_w && h == app.settings.window_h)
                    .unwrap_or(1);
                let (w, h) = presets[(idx + 1) % presets.len()];
                app.settings.window_w = w;
                app.settings.window_h = h;
            } else if !windowed {
                let fill = Color::from_rgba(18, 22, 28, 255);
                draw_rectangle(bx, by, bw, bh, fill);
                draw_rectangle_lines(bx, by, bw, bh, 1.2, UI_EDGE);
                let tw = measure_text(&res_label, None, 18, 1.0).width;
                draw_text(
                    &res_label,
                    bx + (bw - tw) * 0.5,
                    by + bh * 0.5 + 6.0,
                    18.0,
                    TEXT_DIM,
                );
            }
            by += bh + gap;

            if ui_chrome::checkbox_row(
                "Show FPS",
                app.settings.show_fps,
                bx,
                by,
                bw,
                bh,
                mouse,
            ) {
                app.settings.show_fps = !app.settings.show_fps;
            }
            by += bh + gap;

            let fps_label = format!("FPS limit: {}", app.settings.fps_limit.label());
            if button(&fps_label, bx, by, bw, bh, mouse) {
                app.settings.fps_limit = app.settings.fps_limit.next();
            }
            by += bh + gap;

            let fx_label = format!(
                "Effect quality: {}",
                app.settings.effect_quality.label()
            );
            if button(&fx_label, bx, by, bw, bh, mouse) {
                app.settings.effect_quality = app.settings.effect_quality.next();
            }
        }
        SettingsCategory::Audio => {
            draw_text("Audio", bx, by + 22.0, 22.0, UI_TEXT);
            draw_text(
                "Coming soon — volume mixers will live here.",
                bx,
                by + 52.0,
                16.0,
                TEXT_DIM,
            );
        }
        SettingsCategory::Keybinds => {
            draw_text("Keybinds", bx, by + 22.0, 22.0, UI_TEXT);
            draw_text(
                "Coming soon — remapping will live here.",
                bx,
                by + 52.0,
                16.0,
                TEXT_DIM,
            );
        }
    }

    // Footer: Apply + Back side by side.
    let fw = (shell.footer.w - 12.0) * 0.5;
    let fy = shell.footer.y + (shell.footer.h - bh) * 0.5;
    if button_primary("Apply", shell.footer.x, fy, fw, bh, mouse) {
        if let Err(e) = app.settings.save() {
            app.status_toast = e;
        } else {
            app.settings.apply_runtime();
            app.status_toast = "Settings applied".into();
        }
    }
    if button("Back", shell.footer.x + fw + 12.0, fy, fw, bh, mouse) {
        let _ = app.settings.save();
        if app.settings_return == Screen::Game {
            app.screen = Screen::Game;
            app.pause_open = true;
        } else {
            app.screen = app.settings_return;
        }
    }
}

fn screen_load_game(app: &mut App, mouse: (f32, f32), dt: f32) {
    draw_menu_storm_backdrop_ex(app, dt, None);

    let panel_w = 980.0_f32.min(screen_width() - 48.0);
    let panel_h = 560.0_f32.min(screen_height() - 100.0);
    let title_gap = 58.0;
    let ox = (screen_width() - panel_w) * 0.5;
    let oy = ((screen_height() - (title_gap + panel_h)) * 0.5).max(20.0);

    let title = "Load Game";
    let tw = measure_text(title, None, 40, 1.0).width;
    let tx = ox + 24.0;
    draw_text(title, tx, oy + 36.0, 40.0, UI_CYAN);
    draw_rectangle(tx, oy + 44.0, tw * 0.28, 3.0, UI_AMBER);

    let px = ox;
    let py = oy + title_gap;
    ui_chrome::panel(px, py, panel_w, panel_h);

    let pad = 18.0;
    let footer_h = 52.0;
    let list_w = panel_w * 0.36;
    let gap_mid = 14.0;
    let detail_x = px + pad + list_w + gap_mid;
    let detail_w = panel_w - pad * 2.0 - list_w - gap_mid;
    let content_top = py + pad;
    let content_h = panel_h - pad * 2.0 - footer_h - 8.0;
    let row_h = 44.0;
    let row_gap = 6.0;
    let visible = ((content_h + row_gap) / (row_h + row_gap)).floor() as usize;

    let wheel = mouse_wheel().1;
    if wheel != 0.0 {
        app.load_scroll = (app.load_scroll - wheel).max(0.0);
    }
    let max_scroll = app.load_list.len().saturating_sub(visible) as f32;
    app.load_scroll = app.load_scroll.min(max_scroll);
    let start = app.load_scroll as usize;

    if app.load_list.is_empty() {
        let msg = "No saves found";
        let mw = measure_text(msg, None, 18, 1.0).width;
        draw_text(
            msg,
            px + pad + (list_w - mw) * 0.5,
            content_top + 40.0,
            18.0,
            TEXT_DIM,
        );
    } else {
        let mut clicked = None;
        for (vis_i, info) in app.load_list.iter().skip(start).take(visible).enumerate() {
            let i = start + vis_i;
            let y = content_top + vis_i as f32 * (row_h + row_gap);
            let selected = app.load_selected == Some(i);
            if ui_chrome::list_row(
                &info.label,
                &format_saved_at(info.saved_at),
                px + pad,
                y,
                list_w,
                row_h,
                selected,
                mouse,
            ) {
                clicked = Some(i);
            }
        }
        if let Some(i) = clicked {
            app.load_selected = Some(i);
            app.refresh_load_preview();
        }
    }

    draw_rectangle(
        detail_x,
        content_top,
        detail_w,
        content_h,
        Color::from_rgba(12, 16, 22, 200),
    );
    draw_rectangle_lines(detail_x, content_top, detail_w, content_h, 1.0, UI_EDGE);

    if let Some(i) = app.load_selected {
        if let Some(info) = app.load_list.get(i).cloned() {
            let preview_h = content_h * 0.58;
            let preview_pad = 10.0;
            if let Some(tex) = app.load_preview_tex.as_ref() {
                let aspect = tex.width() / tex.height().max(1.0);
                let mut pw = detail_w - preview_pad * 2.0;
                let mut ph = pw / aspect;
                if ph > preview_h - preview_pad {
                    ph = preview_h - preview_pad;
                    pw = ph * aspect;
                }
                let ix = detail_x + (detail_w - pw) * 0.5;
                let iy = content_top + preview_pad;
                draw_texture_ex(
                    tex,
                    ix,
                    iy,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(pw, ph)),
                        ..Default::default()
                    },
                );
            } else {
                let msg = "No preview";
                let mw = measure_text(msg, None, 18, 1.0).width;
                draw_text(
                    msg,
                    detail_x + (detail_w - mw) * 0.5,
                    content_top + preview_h * 0.5,
                    18.0,
                    TEXT_DIM,
                );
            }

            let stats_y = content_top + preview_h + 8.0;
            let sw = detail_w - 24.0;
            let sx = detail_x + 12.0;
            let mut sy = stats_y + 18.0;
            let line = 22.0;
            ui_chrome::stat_line("Mode", info.game_mode.label(), sx, sy, sw);
            sy += line;
            ui_chrome::stat_line(
                "Playtime",
                &format_playtime(info.play_seconds),
                sx,
                sy,
                sw,
            );
            sy += line;
            ui_chrome::stat_line("Saved", &format_saved_at(info.saved_at), sx, sy, sw);
            sy += line;
            ui_chrome::stat_line("Buildings", &format!("{}", info.buildings), sx, sy, sw);
            sy += line;
            ui_chrome::stat_line("Belts", &format!("{}", info.belts), sx, sy, sw);
            sy += line;
            ui_chrome::stat_line("Version", &format!("{}", info.version), sx, sy, sw);
            sy += line;
            ui_chrome::stat_line("Size", &format_bytes(info.file_bytes), sx, sy, sw);
        }
    } else {
        let msg = "Select a save";
        let mw = measure_text(msg, None, 18, 1.0).width;
        draw_text(
            msg,
            detail_x + (detail_w - mw) * 0.5,
            content_top + content_h * 0.45,
            18.0,
            TEXT_DIM,
        );
    }

    let fy = py + panel_h - pad - 44.0;
    let bh = 44.0;
    let btn_w = 140.0;
    if button("Back", px + pad, fy, btn_w, bh, mouse) {
        if app.settings_return == Screen::Game || app.pause_open {
            app.screen = Screen::Game;
            app.pause_open = true;
        } else if app.load_for_host {
            app.screen = Screen::HostSetup;
        } else {
            app.screen = Screen::SinglePlayer;
        }
    }
    let del_x = px + pad + btn_w + 12.0;
    if app.load_selected.is_some()
        && ui_chrome::button_styled("Delete", del_x, fy, btn_w, bh, mouse, ButtonStyle::Danger)
    {
        if let Some(i) = app.load_selected {
            if let Some(info) = app.load_list.get(i).cloned() {
                match delete_save(&info) {
                    Ok(()) => {
                        app.load_list = list_saves();
                        app.load_selected = if app.load_list.is_empty() {
                            None
                        } else {
                            Some(i.min(app.load_list.len() - 1))
                        };
                        app.refresh_load_preview();
                        app.status_toast = "Save deleted".into();
                    }
                    Err(e) => app.status_toast = e,
                }
            }
        }
    }
    let load_w = 180.0;
    let load_x = px + panel_w - pad - load_w;
    if app.load_selected.is_some() {
        if button_primary("Load", load_x, fy, load_w, bh, mouse) {
            if let Some(i) = app.load_selected {
                if let Some(info) = app.load_list.get(i) {
                    match read_save(&info.path) {
                        Ok(save) => {
                            if let Err(e) = app.enter_from_save(&save) {
                                app.status_toast = e;
                            } else if app.load_for_host {
                                app.begin_host_session();
                                app.status_toast =
                                    "Save ready — share your code, then enter the world".into();
                            }
                        }
                        Err(e) => app.status_toast = e,
                    }
                }
            }
        }
    } else {
        draw_rectangle(load_x, fy, load_w, bh, Color::from_rgba(22, 28, 34, 255));
        draw_rectangle_lines(load_x, fy, load_w, bh, 1.0, UI_EDGE);
        let label = "Load";
        let lw = measure_text(label, None, 18, 1.0).width;
        draw_text(
            label,
            load_x + (load_w - lw) * 0.5,
            fy + bh * 0.5 + 6.0,
            18.0,
            TEXT_DIM,
        );
    }

    let _ = dt;
}

fn pause_menu_rect(app: &App) -> Rect {
    let w = 380.0;
    let mut rows = 3usize; // Resume, Settings, Main Menu
    if app.is_single_player() {
        rows += 3; // Save, Load, Start Multiplayer
    } else if app.net.as_ref().map(|n| n.is_host).unwrap_or(false) {
        rows += 2; // code row + players header
        rows += app.peers.len().min(6);
    } else if app.net.is_some() {
        rows += 1; // code display for joiner? skip - only host has code
    }
    let h = 70.0 + rows as f32 * 56.0 + 24.0;
    Rect {
        x: (screen_width() - w) * 0.5,
        y: ((screen_height() - h) * 0.5).max(24.0),
        w,
        h: h.min(screen_height() - 48.0),
    }
}

fn draw_and_handle_pause_menu(app: &mut App, mouse: (f32, f32)) {
    ui_chrome::scrim(170);
    let r = pause_menu_rect(app);
    ui_chrome::panel(r.x, r.y, r.w, r.h);
    ui_chrome::panel_header(r.x, r.y + 8.0, r.w, "Paused", Some("Esc to resume"));

    let bw = r.w - 48.0;
    let bh = 44.0;
    let bx = r.x + 24.0;
    let mut by = r.y + 78.0;
    let gap = 10.0;
    let (mx, my) = ui_chrome::pointer();

    if button_primary("Resume", bx, by, bw, bh, mouse) {
        app.pause_open = false;
        return;
    }
    by += bh + gap;

    if app.is_single_player() {
        if button("Save Game", bx, by, bw, bh, mouse) {
            app.do_manual_save();
        }
        by += bh + gap;
        if button("Load Game", bx, by, bw, bh, mouse) {
            app.settings_return = Screen::Game;
            app.open_load_game();
            return;
        }
        by += bh + gap;
        if button_primary("Start Multiplayer", bx, by, bw, bh, mouse) {
            app.promote_solo_to_multiplayer();
            return;
        }
        by += bh + gap;
    } else if app.net.as_ref().map(|n| n.is_host).unwrap_or(false) {
        // Session code — click to copy
        let code = if app.host_code.is_empty() {
            "Setting up…"
        } else {
            app.host_code.as_str()
        };
        let code_h = 52.0;
        let hovered = ui_chrome::point_in(mx, my, bx, by, bw, code_h);
        draw_rectangle(
            bx,
            by,
            bw,
            code_h,
            if hovered {
                Color::from_rgba(36, 52, 58, 255)
            } else {
                Color::from_rgba(22, 28, 36, 255)
            },
        );
        draw_rectangle_lines(
            bx,
            by,
            bw,
            code_h,
            1.4,
            if hovered { UI_CYAN } else { UI_EDGE },
        );
        draw_text("Session code (click to copy)", bx + 12.0, by + 16.0, 13.0, TEXT_DIM);
        let cw = measure_text(code, None, 28, 1.0).width;
        draw_text(code, bx + (bw - cw) * 0.5, by + 42.0, 28.0, UI_CYAN);
        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            app.copy_session_code();
        }
        by += code_h + gap;

        if !app.peers.is_empty() {
            draw_text("Players", bx + 4.0, by + 14.0, 14.0, TEXT_DIM);
            by += 22.0;
            let mut ids: Vec<u8> = app.peers.keys().copied().collect();
            ids.sort_unstable();
            for id in ids.into_iter().take(6) {
                let label = format!("Player {id}");
                let kick_w = 88.0;
                draw_text(&label, bx + 8.0, by + bh * 0.5 + 5.0, 16.0, UI_TEXT);
                if ui_chrome::button_styled(
                    "Kick",
                    bx + bw - kick_w,
                    by,
                    kick_w,
                    bh - 4.0,
                    mouse,
                    ButtonStyle::Danger,
                ) {
                    app.kick_peer(id);
                }
                by += bh + 4.0;
            }
        } else {
            draw_text("No players connected yet", bx + 4.0, by + 14.0, 14.0, TEXT_DIM);
            by += 28.0;
        }
    }

    if button("Settings", bx, by, bw, bh, mouse) {
        app.open_settings(Screen::Game);
        return;
    }
    by += bh + gap;
    if ui_chrome::button_styled("Main Menu", bx, by, bw, bh, mouse, ButtonStyle::Ghost) {
        app.return_to_main_menu();
    }
}

fn screen_host_lobby(app: &mut App, mouse: (f32, f32), dt: f32) {
    drain_net(app);
    draw_menu_storm_backdrop_ex(app, dt, None);
    let (px, py, panel_w, panel_h, pad, _) = titled_menu_panel("Host", 440.0, 400.0);
    let bw = panel_w - pad * 2.0;
    let bh = 48.0;
    let bx = px + pad;
    let mut by = py + pad + 4.0;

    draw_text("Share this code with friends", bx, by + 4.0, 15.0, TEXT_DIM);
    by += 28.0;
    let code = if app.host_code.is_empty() {
        "……"
    } else {
        app.host_code.as_str()
    };
    let code_h = 72.0;
    let (mx, my) = ui_chrome::pointer();
    let code_hovered = ui_chrome::point_in(mx, my, bx, by, bw, code_h);
    draw_rectangle(
        bx,
        by,
        bw,
        code_h,
        if code_hovered {
            Color::from_rgba(36, 52, 58, 255)
        } else {
            Color::from_rgba(22, 28, 36, 255)
        },
    );
    draw_rectangle_lines(
        bx,
        by,
        bw,
        code_h,
        1.4,
        if code_hovered { UI_CYAN } else { UI_EDGE },
    );
    let cw = measure_text(code, None, 48, 1.0).width;
    draw_text(code, bx + (bw - cw) * 0.5, by + 44.0, 48.0, CYAN);
    draw_text(
        "Click to copy",
        bx + 12.0,
        by + code_h - 10.0,
        13.0,
        TEXT_DIM,
    );
    if code_hovered && is_mouse_button_pressed(MouseButton::Left) && !app.host_code.is_empty() {
        app.copy_session_code();
    }
    by += code_h + 12.0;

    if !app.join_status.is_empty() {
        draw_text(&app.join_status, bx, by + 4.0, 16.0, ACCENT);
    }

    by = py + panel_h - pad - bh * 2.0 - 12.0;
    if button_primary("Enter World", bx, by, bw, bh, mouse) {
        if app.net.is_some() {
            let hotbar = app.ui.hotbar;
            let hotbar_index = app.ui.hotbar_index;
            let selected = app.ui.selected;
            app.enter_game_common();
            app.ui.hotbar = hotbar;
            app.ui.hotbar_index = hotbar_index;
            app.ui.selected = selected;
        }
    }
    by += bh + 12.0;
    if button("Back", bx, by, bw, bh, mouse) {
        app.stop_net();
        app.screen = Screen::HostSetup;
    }
}

fn text_field(
    label: &str,
    value: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    focused: bool,
    _mouse: (f32, f32),
) -> bool {
    let (mx, my) = ui_chrome::pointer();
    let hovered = ui_chrome::point_in(mx, my, x, y, w, h);
    draw_text(label, x, y - 8.0, 16.0, TEXT_DIM);
    ui_chrome::text_field_frame(x, y, w, h, focused);
    let shown = if value.is_empty() && !focused {
        "…"
    } else {
        value
    };
    draw_text(
        shown,
        x + 12.0,
        y + h * 0.5 + 6.0,
        18.0,
        if value.is_empty() && !focused {
            TEXT_DIM
        } else {
            TEXT
        },
    );
    if focused {
        let tw = measure_text(value, None, 18, 1.0).width;
        let cx = x + 12.0 + tw + 2.0;
        if ((get_time() * 2.0) as i32) % 2 == 0 {
            draw_rectangle(cx, y + 10.0, 2.0, h - 20.0, UI_CYAN);
        }
    }
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn handle_text_input(target: &mut String) {
    while let Some(c) = get_char_pressed() {
        if c.is_ascii_alphanumeric() && target.len() < 12 {
            target.push(c.to_ascii_uppercase());
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        target.pop();
    }
}

fn screen_join_lobby(app: &mut App, mouse: (f32, f32), dt: f32) {
    drain_net(app);
    draw_menu_storm_backdrop_ex(app, dt, None);
    let (px, py, panel_w, panel_h, pad, _) = titled_menu_panel("Join Game", 440.0, 340.0);
    let bw = panel_w - pad * 2.0;
    let bh = 48.0;
    let bx = px + pad;
    let mut by = py + pad + 8.0;

    if text_field(
        "Session code",
        &app.join_code,
        bx,
        by,
        bw,
        56.0,
        app.join_focus,
        mouse,
    ) {
        app.join_focus = true;
    }
    if app.join_focus {
        handle_text_input(&mut app.join_code);
    }
    by += 72.0;

    if !app.join_status.is_empty() {
        draw_text(&app.join_status, bx, by, 16.0, ACCENT);
    }

    by = py + panel_h - pad - bh * 2.0 - 12.0;
    if button_primary("Connect", bx, by, bw, bh, mouse) {
        app.stop_net();
        app.world.clear();
        app.join_status = "Connecting…".into();
        let handle = net::start_client("", &app.join_code);
        app.net = Some(handle);
    }
    by += bh + 12.0;
    if button("Back", bx, by, bw, bh, mouse) {
        app.stop_net();
        app.join_focus = false;
        app.screen = Screen::Multiplayer;
    }
    let _ = dt;
}

fn send_world_snapshot(app: &App) {
    let Some(net) = app.net.as_ref() else {
        return;
    };
    let _ = net.tx.send(NetCommand::SnapBegin);
    push_world_ops(app, net);
    let _ = net.tx.send(NetCommand::SnapEnd);
    // Push inventories so joiners match host-authoritative material counts.
    broadcast_inventory(app, app.local_player_id);
    for &id in app.peer_inventories.keys() {
        broadcast_inventory(app, id);
    }
}

fn inventory_counts(app: &App, player_id: u8) -> (u32, u32) {
    if player_id == app.local_player_id {
        (app.inventory.ore(), app.inventory.ingot())
    } else if let Some(inv) = app.peer_inventories.get(&player_id) {
        (inv.ore(), inv.ingot())
    } else {
        let s = Inventory::starter();
        (s.ore(), s.ingot())
    }
}

fn inventory_of_mut<'a>(app: &'a mut App, player_id: u8) -> &'a mut Inventory {
    if player_id == app.local_player_id {
        &mut app.inventory
    } else {
        app.peer_inventories
            .entry(player_id)
            .or_insert_with(Inventory::starter)
    }
}

fn broadcast_inventory(app: &App, player_id: u8) {
    let Some(net) = app.net.as_ref() else {
        return;
    };
    let (ore, ingot) = inventory_counts(app, player_id);
    let _ = net.tx.send(NetCommand::SetInventory {
        id: player_id,
        ore,
        ingot,
    });
}

fn try_spend_for(app: &mut App, player_id: u8, costs: &[(crate::sim::Item, u32)]) -> bool {
    if app.is_creative() || costs.is_empty() {
        return true;
    }
    let ok = inventory_of_mut(app, player_id).try_spend(costs);
    if ok {
        broadcast_inventory(app, player_id);
    }
    ok
}

fn refund_for(app: &mut App, player_id: u8, costs: &[(crate::sim::Item, u32)]) {
    if app.is_creative() || costs.is_empty() {
        return;
    }
    inventory_of_mut(app, player_id).refund(costs);
    broadcast_inventory(app, player_id);
}

fn push_world_ops(app: &App, net: &NetHandle) {
    let mut ids: Vec<u32> = app.world.nodes.keys().copied().collect();
    ids.sort_unstable();
    for id in ids {
        if let Some(n) = app.world.nodes.get(&id) {
            let _ = net.tx.send(NetCommand::Place {
                id,
                kind: n.kind,
                x: n.x,
                y: n.y,
                facing: n.facing,
                request: false,
            });
        }
    }
    for l in &app.world.links {
        let _ = net.tx.send(NetCommand::Link {
            power: true,
            from_node: l.from_node,
            from_port: l.from_port,
            to_node: l.to_node,
            to_port: l.to_port,
            request: false,
        });
    }
    // Belt tiles: multiplayer sync TBD — topology-only power links for now.
}

fn maybe_host_snapshot(app: &mut App) {
    let Some(net) = app.net.as_ref() else {
        return;
    };
    if !net.is_host {
        return;
    }
    // Soft reconcile (no wipe) so lossy brokers heal without flickering.
    if app.last_snap_send.elapsed().as_millis() < 500 {
        return;
    }
    app.last_snap_send = Instant::now();
    push_world_ops(app, net);
}

fn peer_label(id: u8) -> String {
    format!("Player {}", id + 1)
}

fn advance_peer_cursors(app: &mut App, dt: f32) {
    for peer in app.peers.values_mut() {
        // Dead-reckon mouse + drone at UPS until the next authoritative sample.
        peer.x += peer.vx * dt;
        peer.y += peer.vy * dt;
        peer.drone.tick(dt);
    }
}

fn drain_net(app: &mut App) {
    let events: Vec<NetEvent> = match app.net.as_ref() {
        Some(net) => {
            let mut evs = Vec::new();
            while let Ok(ev) = net.rx.try_recv() {
                evs.push(ev);
            }
            evs
        }
        None => return,
    };

    let is_host = app.net.as_ref().map(|n| n.is_host).unwrap_or(false);

    for ev in events {
        match ev {
            NetEvent::HostReady { code, addr } => {
                app.host_code = code;
                app.host_addr = addr;
            }
            NetEvent::Joined { player_id } => {
                app.local_player_id = player_id;
                app.world.set_id_namespace(player_id);
                if player_id != 0 {
                    // Clients start with a fresh kit; host may overwrite via INV on snap.
                    app.inventory = Inventory::starter();
                    app.peer_inventories.clear();
                }
                app.join_status = if player_id == 0 {
                    "Host online — share your code".into()
                } else {
                    format!("Joined as player {player_id}")
                };
                if app.screen == Screen::JoinLobby {
                    app.cam = Cam {
                        x: 0.0,
                        y: 0.0,
                        zoom: 1.0,
                    };
                    app.player = Player::new(0.0, 0.0);
                    app.enter_game_common();
                }
            }
            NetEvent::JoinFailed { reason } => {
                app.join_status = format!("Failed: {reason}");
                app.net = None;
            }
            NetEvent::Kicked => {
                app.stop_net();
                app.pause_open = false;
                app.status_toast = "Kicked by host".into();
                app.screen = Screen::Multiplayer;
            }
            NetEvent::PeerHello | NetEvent::WantSnap => {
                if is_host {
                    send_world_snapshot(app);
                    app.last_snap_send = Instant::now();
                    app.join_status = "Synced world to joiner".into();
                }
            }
            NetEvent::PlaceRequest {
                owner,
                kind,
                x,
                y,
                facing,
            } => {
                if is_host {
                    let unlock = kind.tech_unlock();
                    if !app.is_creative() && !app.world.tech.machine_unlocked(unlock) {
                        app.join_status = format!("Player {owner}: locked ({unlock})");
                    } else {
                        let costs = building_recipe(kind);
                        if !try_spend_for(app, owner, costs) {
                            app.join_status = format!(
                                "Player {owner}: {}",
                                inventory_of_mut(app, owner).missing_hint(costs)
                            );
                        } else if let Some(id) = app.world.place_node(kind, x, y, facing) {
                            if let Some(net) = app.net.as_ref() {
                                let _ = net.tx.send(NetCommand::Place {
                                    id,
                                    kind,
                                    x,
                                    y,
                                    facing,
                                    request: false,
                                });
                            }
                            app.join_status = format!("Host placed {}", kind.short());
                        } else {
                            refund_for(app, owner, costs);
                        }
                    }
                }
            }
            NetEvent::RemoveRequest { owner, id } => {
                if is_host {
                    let kind = app.world.nodes.get(&id).map(|n| n.kind);
                    app.world.remove_node(id);
                    if let Some(k) = kind {
                        refund_for(app, owner, building_recipe(k));
                    }
                    if let Some(net) = app.net.as_ref() {
                        let _ = net.tx.send(NetCommand::Remove {
                            id,
                            request: false,
                        });
                    }
                }
            }
            NetEvent::MoveRequest { id, x, y } => {
                if is_host {
                    if app.world.try_move_node(id, x, y) {
                        if let Some(net) = app.net.as_ref() {
                            let _ = net.tx.send(NetCommand::Move {
                                id,
                                x,
                                y,
                                request: false,
                            });
                        }
                    }
                }
            }
            NetEvent::RotateRequest { id, facing } => {
                if is_host {
                    app.world.force_set_facing(id, facing);
                    if let Some(net) = app.net.as_ref() {
                        let _ = net.tx.send(NetCommand::Rotate {
                            id,
                            facing,
                            request: false,
                        });
                    }
                }
            }
            NetEvent::LinkRequest {
                power,
                from_node,
                from_port,
                to_node,
                to_port,
            } => {
                if is_host {
                    let ok = if power {
                        app.world
                            .connect_power((from_node, from_port), (to_node, to_port))
                    } else {
                        // Grid belts replaced port-to-port belt links; ignore legacy.
                        false
                    };
                    if ok {
                        if let Some(net) = app.net.as_ref() {
                            let _ = net.tx.send(NetCommand::Link {
                                power,
                                from_node,
                                from_port,
                                to_node,
                                to_port,
                                request: false,
                            });
                        }
                    }
                }
            }
            NetEvent::SnapBegin => {
                if !is_host {
                    app.world.nodes.clear();
                    app.world.links.clear();
                    app.world.belt_tiles.clear();
                    app.applying_snap = true;
                    app.join_status = "Receiving world…".into();
                }
            }
            NetEvent::SnapEnd => {
                if !is_host {
                    app.applying_snap = false;
                    app.join_status = "World synced".into();
                }
            }
            NetEvent::PeerCursor {
                id,
                x,
                y,
                selected,
                facing,
                t_ms,
                dx,
                dy,
                dfacing,
                dvx,
                dvy,
            } => {
                if id == app.local_player_id {
                    continue;
                }
                if let Some(peer) = app.peers.get_mut(&id) {
                    if t_ms + 0.5 < peer.last_sample_t {
                        continue;
                    }
                    let dt = ((t_ms - peer.last_sample_t) / 1000.0).max(FIXED_DT);
                    // Authoritative snap — same pose the peer simulated this UPS tick.
                    peer.vx = if t_ms > peer.last_sample_t {
                        (x - peer.x) / dt
                    } else {
                        0.0
                    };
                    peer.vy = if t_ms > peer.last_sample_t {
                        (y - peer.y) / dt
                    } else {
                        0.0
                    };
                    // Prefer packet deltas when present; else derive from positions.
                    if dvx.abs() + dvy.abs() > 1e-3 {
                        peer.drone.apply_net(dx, dy, dvx, dvy, dfacing);
                    } else {
                        let odx = peer.drone.x;
                        let ody = peer.drone.y;
                        peer.drone
                            .apply_net(dx, dy, (dx - odx) / dt, (dy - ody) / dt, dfacing);
                    }
                    peer.x = x;
                    peer.y = y;
                    peer.selected = selected;
                    peer.facing = facing;
                    peer.last_sample_t = t_ms;
                } else {
                    app.peers.insert(
                        id,
                        PeerPresence {
                            id,
                            x,
                            y,
                            vx: 0.0,
                            vy: 0.0,
                            drone: {
                                let mut d = player::RemoteDrone::new(dx, dy, dfacing);
                                d.apply_net(dx, dy, dvx, dvy, dfacing);
                                d
                            },
                            selected,
                            facing,
                            last_sample_t: t_ms,
                        },
                    );
                }
            }
            NetEvent::PeerPlace {
                id,
                kind,
                x,
                y,
                facing,
            } => {
                let _ = app.world.place_node_with_id(id, kind, x, y, facing);
                if !app.applying_snap {
                    app.join_status = format!("Synced {}", kind.short());
                }
            }
            NetEvent::PeerRemove { id } => {
                app.world.remove_node(id);
            }
            NetEvent::PeerMove { id, x, y } => {
                app.world.force_move_node(id, x, y);
            }
            NetEvent::PeerRotate { id, facing } => {
                app.world.force_set_facing(id, facing);
            }
            NetEvent::PeerLink {
                power,
                from_node,
                from_port,
                to_node,
                to_port,
            } => {
                if power {
                    let _ = app
                        .world
                        .connect_power((from_node, from_port), (to_node, to_port));
                }
                // Legacy non-power ("belt") links are ignored — belts are a tile grid.
            }
            NetEvent::PeerGone { id } => {
                app.peers.remove(&id);
                app.peer_inventories.remove(&id);
            }
            NetEvent::PeerInventory { id, ore, ingot } => {
                let inv = Inventory::from_totals(ore, ingot);
                if id == app.local_player_id {
                    app.inventory = inv;
                } else {
                    app.peer_inventories.insert(id, inv);
                }
            }
            NetEvent::Info(msg) => {
                // Surface peer presence clearly in lobby + HUD.
                app.join_status = msg;
            }
        }
    }
}

fn send_cursor_ups(app: &mut App, wx: f32, wy: f32) {
    let Some(net) = app.net.as_ref() else {
        return;
    };
    app.last_cursor_send = Instant::now();
    app.last_cursor_x = wx;
    app.last_cursor_y = wy;
    let t_ms = app.cursor_clock.elapsed().as_secs_f32() * 1000.0;
    let _ = net.tx.send(NetCommand::SetCursor {
        x: wx,
        y: wy,
        selected: app.ui.selected,
        facing: app.ui.place_facing,
        t_ms,
        dx: app.player.x,
        dy: app.player.y,
        dfacing: app.player.facing,
        dvx: app.player.vx,
        dvy: app.player.vy,
    });
}

fn handle_hotkeys(app: &mut App, wx: f32, wy: f32) {
    if app.pause_open {
        if is_key_pressed(KeyCode::Escape) {
            app.pause_open = false;
        }
        return;
    }

    if is_key_pressed(KeyCode::F7) {
        if let Some(t) = content::content()
            .techs
            .iter()
            .find(|t| !app.world.tech.is_researched(&t.id))
        {
            let id = t.id.clone();
            let name = t.name.clone();
            app.world.tech.debug_unlock(&id);
            app.status_toast = format!("DEBUG unlocked: {name}");
        } else {
            app.status_toast = "DEBUG: all techs already unlocked".into();
        }
    }
    if is_key_pressed(KeyCode::F8) {
        app.world.tech.debug_unlock_all();
        app.status_toast = "DEBUG: all Era 1 techs unlocked".into();
    }
    if is_key_pressed(KeyCode::F9) {
        if let Some(t) = content::content()
            .techs
            .iter()
            .find(|t| app.world.tech.can_start(&t.id))
        {
            let id = t.id.clone();
            if app.is_creative() {
                for (key, &need) in &t.science_cost {
                    app.world.tech.deposit_science(key, need as f32);
                }
            }
            if app.world.tech.start_research(&id) {
                app.status_toast = format!("Research started: {}", id);
            }
        } else {
            app.status_toast = "No researchable tech (prereqs / done)".into();
        }
    }

    // While the build menu is open, letter keys feed search — don't toggle/rotate/clear.
    if !app.ui.build_open {
        if is_key_pressed(KeyCode::B) {
            app.ui.toggle_build();
        }
        if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::E) {
            app.ui.toggle_inventory();
        }
        if is_key_pressed(KeyCode::C) {
            app.player.cam_mode = app.player.cam_mode.toggle();
            if app.player.cam_mode == CamMode::Follow {
                player::snap_cam_to_player(
                    &mut app.cam.x,
                    &mut app.cam.y,
                    app.player.x,
                    app.player.y,
                );
            }
            app.status_toast = format!("{}  (C to toggle)", app.player.cam_mode.label());
        }
        if is_key_pressed(KeyCode::Q) {
            app.ui.clear_tool();
            app.ui.context_menu = None;
        }
        if is_key_pressed(KeyCode::R) {
            app.ui.place_facing = app.ui.place_facing.rotate_cw();
            let rotated_id = if let Some(id) = app.ui.drag_node {
                if app.world.try_rotate_node(id) {
                    Some(id)
                } else {
                    None
                }
            } else if let Some(id) = app.world.hit_node(wx, wy) {
                if app.world.try_rotate_node(id) {
                    Some(id)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(id) = rotated_id {
                if let Some(n) = app.world.nodes.get(&id) {
                    if let Some(net) = app.net.as_ref() {
                        let _ = net.tx.send(NetCommand::Rotate {
                            id,
                            facing: n.facing,
                            request: !net.is_host,
                        });
                    }
                }
            }
        }
    } else if is_key_pressed(KeyCode::B) && app.ui.build_search.is_empty() {
        // B still closes when search is empty (no conflict with typing).
        app.ui.close_build();
    }

    if is_key_pressed(KeyCode::Escape) {
        if app.pause_open {
            app.pause_open = false;
        } else if app.ui.inventory_open {
            app.ui.close_inventory();
        } else if app.ui.context_menu.take().is_some() {
            // closed pie / context
        } else if app.ui.overlay.take().is_some() {
            // closed corner overlay
        } else if app.ui.build_open {
            if !app.ui.build_search.is_empty() {
                app.ui.build_search.clear();
                app.ui.build_scroll = 0.0;
            } else {
                app.ui.close_build();
            }
        } else if app.ui.wire_paint.take().is_some()
            || app.ui.wire_from.take().is_some()
            || app.ui.selected.take().is_some()
            || app.ui.drag_node.is_some()
        {
            if let Some(id) = app.ui.drag_node.take() {
                app.world.set_node_held(id, false);
            }
            app.ui.hotbar_drag_from = None;
        } else {
            app.pause_open = true;
        }
    }

    // Backspace mines / removes under the cursor (no Delete key).
    if is_key_pressed(KeyCode::Backspace)
        && !app.ui.build_open
        && !app.ui.inventory_open
        && app.ui.context_menu.is_none()
    {
        if let Some(id) = app.world.hit_node(wx, wy) {
            remove_building(app, id, true);
        } else {
            let (tx, ty) = world_to_tile(wx, wy);
            if app.world.remove_belt_at(tx, ty) {
                refund_for(app, app.local_player_id, belt_recipe());
                app.status_toast = "Belt removed".into();
            }
        }
    }

    for (i, key) in [
        KeyCode::Key1,
        KeyCode::Key2,
        KeyCode::Key3,
        KeyCode::Key4,
        KeyCode::Key5,
        KeyCode::Key6,
        KeyCode::Key7,
        KeyCode::Key8,
        KeyCode::Key9,
    ]
    .iter()
    .enumerate()
    {
        if is_key_pressed(*key) {
            if app.ui.build_open {
                let entry = app.ui.palette_entry().or_else(|| app.ui.current_entry());
                if let Some(entry) = entry {
                    app.ui.set_hotbar_entry(i, Some(entry));
                    app.ui.hotbar_index = i;
                }
            } else {
                app.ui.hotbar_index = i;
                if let Some(entry) = app.ui.hotbar_entry(i) {
                    app.ui.select_entry(entry);
                } else {
                    app.ui.clear_tool();
                }
                app.ui.wire_from = None;
                app.ui.context_menu = None;
            }
        }
    }
}

fn handle_pan_zoom(app: &mut App, mouse: (f32, f32)) {
    let follow = app.player.cam_mode == CamMode::Follow;
    let cam = &mut app.cam;
    let ui = &mut app.ui;

    // Middle-mouse free pan — only in Free mode (Follow keeps cam on the mech).
    if !follow {
        if is_mouse_button_pressed(MouseButton::Middle) {
            ui.panning = true;
            ui.pan_last = mouse;
        }
        if is_mouse_button_released(MouseButton::Middle) {
            ui.panning = false;
        }
        if ui.panning && is_mouse_button_down(MouseButton::Middle) {
            let dx = mouse.0 - ui.pan_last.0;
            let dy = mouse.1 - ui.pan_last.1;
            cam.x -= dx / cam.zoom;
            cam.y -= dy / cam.zoom;
            ui.pan_last = mouse;
        }
    } else {
        ui.panning = false;
    }

    // Build menu owns the wheel for grid scrolling while open.
    if ui.build_open {
        return;
    }

    let wheel = mouse_wheel().1;
    if wheel != 0.0 {
        let old = cam.zoom;
        cam.zoom = (cam.zoom * (1.0 + wheel * 0.1)).clamp(MIN_ZOOM, MAX_ZOOM);
        let before_x = (mouse.0 - screen_width() * 0.5) / old + cam.x;
        let before_y = (mouse.1 - screen_height() * 0.5) / old + cam.y;
        let after_x = (mouse.0 - screen_width() * 0.5) / cam.zoom + cam.x;
        let after_y = (mouse.1 - screen_height() * 0.5) / cam.zoom + cam.y;
        cam.x += before_x - after_x;
        cam.y += before_y - after_y;
    }
    cam.quantize();
}

fn ui_scale() -> f32 {
    let by_h = screen_height() / 900.0;
    let by_w = screen_width() / 1400.0;
    by_h.max(by_w).clamp(1.0, 1.5)
}

fn s(v: f32) -> f32 {
    v * ui_scale()
}

/// Floating hotbar — only as wide as the slots, sits above the bottom edge.
fn hotbar_geom() -> (f32, f32, f32, f32) {
    let slot = s(56.0);
    let gap = s(6.0);
    let width = HOTBAR_SLOTS as f32 * slot + (HOTBAR_SLOTS - 1) as f32 * gap;
    let x = (screen_width() - width) * 0.5;
    let y = screen_height() - slot - s(22.0);
    (x, y, slot, gap)
}

/// Vertical tool rail in the bottom-right — icon-only, labels appear on hover.
fn tool_button_rect(index: usize) -> Rect {
    let size = s(48.0);
    let gap = s(10.0);
    let total_h = 4.0 * size + 3.0 * gap;
    let x = screen_width() - size - s(18.0);
    let y0 = screen_height() - total_h - s(22.0);
    Rect {
        x,
        y: y0 + index as f32 * (size + gap),
        w: size,
        h: size,
    }
}

fn point_in_tool_button(mx: f32, my: f32) -> Option<CornerTool> {
    for (i, tool) in CornerTool::ALL.iter().enumerate() {
        let r = tool_button_rect(i);
        if mx >= r.x && mx <= r.x + r.w && my >= r.y && my <= r.y + r.h {
            return Some(*tool);
        }
    }
    None
}

fn point_in_hotbar(mx: f32, my: f32) -> Option<usize> {
    let (bar_x, bar_y, slot, gap) = hotbar_geom();
    if my < bar_y || my > bar_y + slot {
        return None;
    }
    for i in 0..HOTBAR_SLOTS {
        let x = bar_x + i as f32 * (slot + gap);
        if mx >= x && mx <= x + slot {
            return Some(i);
        }
    }
    None
}

fn point_in_hud_chrome(mx: f32, my: f32) -> bool {
    point_in_hotbar(mx, my).is_some() || point_in_tool_button(mx, my).is_some()
}

fn kind_swatch(kind: BuildingKind) -> Color {
    match kind {
        BuildingKind::Solar => Color::from_rgba(80, 160, 220, 255),
        BuildingKind::PowerPole => POWER_C,
        BuildingKind::OreNode => ORE_C,
        BuildingKind::Smelter => Color::from_rgba(220, 120, 70, 255),
        BuildingKind::Assembler => Color::from_rgba(90, 150, 200, 255),
        BuildingKind::Box => Color::from_rgba(160, 170, 190, 255),
        BuildingKind::Splitter => BELT_YELLOW,
        BuildingKind::Totem => Color::from_rgba(140, 100, 220, 255),
        BuildingKind::Turret => Color::from_rgba(200, 90, 90, 255),
        BuildingKind::PowerWire => Color::from_rgba(255, 190, 70, 255),
        BuildingKind::Conveyor => BELT_YELLOW,
        BuildingKind::SpawnAssault => Color::from_rgba(180, 60, 70, 255),
        BuildingKind::SpawnHunter => Color::from_rgba(200, 90, 50, 255),
        BuildingKind::SpawnSaboteur => Color::from_rgba(160, 70, 180, 255),
        BuildingKind::SpawnFogcaller => Color::from_rgba(70, 100, 160, 255),
        BuildingKind::SpawnNest => Color::from_rgba(160, 50, 70, 255),
        BuildingKind::Machine => Color::from_rgba(100, 140, 170, 255),
        BuildingKind::Lab => Color::from_rgba(80, 180, 200, 255),
        BuildingKind::FluidTank => Color::from_rgba(70, 130, 190, 255),
        BuildingKind::Pipe => Color::from_rgba(90, 140, 170, 255),
        BuildingKind::Wall => Color::from_rgba(120, 120, 130, 255),
        BuildingKind::ReinforcedWall => Color::from_rgba(90, 95, 110, 255),
        BuildingKind::BallisticTurret => Color::from_rgba(180, 120, 70, 255),
        BuildingKind::LaserTurret => Color::from_rgba(120, 220, 255, 255),
        BuildingKind::NexusSite => Color::from_rgba(200, 170, 80, 255),
        BuildingKind::Nexus => Color::from_rgba(255, 210, 90, 255),
    }
}

fn draw_recipe_icon(cx: f32, cy: f32, color: Color) {
    let u = s(1.0);
    // Layered progression: three nodes left → right with connecting strokes.
    draw_circle(cx - 8.0 * u, cy + 2.0 * u, 3.2 * u, color);
    draw_circle(cx, cy - 4.0 * u, 3.2 * u, color);
    draw_circle(cx + 8.0 * u, cy + 2.0 * u, 3.2 * u, color);
    draw_line(
        cx - 5.0 * u,
        cy + 1.0 * u,
        cx - 2.5 * u,
        cy - 2.0 * u,
        1.6,
        color,
    );
    draw_line(
        cx + 2.5 * u,
        cy - 2.0 * u,
        cx + 5.0 * u,
        cy + 1.0 * u,
        1.6,
        color,
    );
}

fn draw_tech_icon(cx: f32, cy: f32, color: Color) {
    let u = s(1.0);
    draw_circle(cx, cy - 5.5 * u, 3.0 * u, color);
    draw_circle(cx - 7.5 * u, cy + 5.0 * u, 3.0 * u, color);
    draw_circle(cx + 7.5 * u, cy + 5.0 * u, 3.0 * u, color);
    draw_line(cx, cy - 5.5 * u, cx - 7.5 * u, cy + 5.0 * u, 1.7 * u, color);
    draw_line(cx, cy - 5.5 * u, cx + 7.5 * u, cy + 5.0 * u, 1.7 * u, color);
}

fn draw_build_icon(cx: f32, cy: f32, color: Color) {
    let u = s(1.0);
    // Wrench silhouette — matches other vector corner icons.
    draw_line(
        cx - 8.0 * u,
        cy + 8.0 * u,
        cx + 6.0 * u,
        cy - 6.0 * u,
        2.4 * u,
        color,
    );
    draw_circle(cx + 7.0 * u, cy - 7.0 * u, 3.6 * u, color);
    draw_circle(cx + 7.0 * u, cy - 7.0 * u, 1.4 * u, Color::from_rgba(22, 26, 34, 255));
    draw_rectangle(cx - 9.5 * u, cy + 5.5 * u, 5.0 * u, 5.0 * u, color);
}

fn draw_map_icon(cx: f32, cy: f32, color: Color) {
    let u = s(1.0);
    draw_rectangle_lines(cx - 9.0 * u, cy - 7.5 * u, 18.0 * u, 15.0 * u, 1.7 * u, color);
    draw_line(cx - 3.0 * u, cy - 7.5 * u, cx - 3.0 * u, cy + 7.5 * u, 1.4 * u, color);
    draw_line(cx + 3.0 * u, cy - 7.5 * u, cx + 3.0 * u, cy + 7.5 * u, 1.4 * u, color);
    draw_line(cx - 9.0 * u, cy - 1.0 * u, cx + 9.0 * u, cy + 1.5 * u, 1.4 * u, color);
}

fn draw_nodes_icon(cx: f32, cy: f32, color: Color) {
    let u = s(1.0);
    draw_circle(cx - 7.5 * u, cy - 5.0 * u, 3.2 * u, color);
    draw_circle(cx + 7.5 * u, cy - 5.0 * u, 3.2 * u, color);
    draw_circle(cx, cy + 7.5 * u, 3.2 * u, color);
    draw_line(cx - 7.5 * u, cy - 5.0 * u, cx + 7.5 * u, cy - 5.0 * u, 1.6 * u, color);
    draw_line(cx - 7.5 * u, cy - 5.0 * u, cx, cy + 7.5 * u, 1.6 * u, color);
    draw_line(cx + 7.5 * u, cy - 5.0 * u, cx, cy + 7.5 * u, 1.6 * u, color);
}

fn build_menu_rect() -> Rect {
    let w = s(760.0);
    let h = s(540.0);
    Rect {
        x: (screen_width() - w) * 0.5,
        y: (screen_height() - h) * 0.5 - s(20.0),
        w,
        h,
    }
}

fn handle_build_search_input(ui: &mut Ui) {
    if !ui.build_open {
        return;
    }
    if ui.suppress_search_chars {
        ui.suppress_search_chars = false;
        while get_char_pressed().is_some() {}
        return;
    }
    // Auto-focus search so type-to-filter always works while the menu is open.
    ui.build_search_focus = true;
    while let Some(c) = get_char_pressed() {
        if ui.build_search.len() >= 48 {
            break;
        }
        if c.is_ascii_alphabetic() || c == ' ' || c == '-' || c == '_' {
            ui.build_search.push(c);
            ui.build_scroll = 0.0;
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        ui.build_search.pop();
        ui.build_scroll = 0.0;
    }
}

fn revealed_ore_under(app: &App, x: f32, y: f32, w: f32, h: f32) -> bool {
    let zones = app.storm.clear_zones(&app.world);
    app.world.veins.iter().any(|v| {
        v.yield_pct > 1.0
            && v.overlaps_rect(x, y, w, h)
            && app.storm.in_clear(v.x, v.y, &zones)
    })
}

fn place_building_entry(app: &mut App, entry: BuildEntry, x: f32, y: f32, facing: Facing) {
    let kind = entry.kind();
    let machine_id = entry.machine_id();
    if kind.is_debug_tool() || kind.is_cable() || kind.is_belt_tool() {
        return;
    }
    let unlock = entry.tech_unlock();
    if !app.is_creative() && !app.world.tech.machine_unlocked(&unlock) {
        app.status_toast = format!("Locked — research {unlock}");
        return;
    }
    if kind == BuildingKind::OreNode {
        let mut probe = Node::new(kind, x, y, facing);
        if let Some(mid) = machine_id {
            probe.set_machine_id(Some(mid));
        }
        if !revealed_ore_under(app, x, y, probe.w(), probe.h()) {
            app.status_toast = "Mining drill needs a revealed gas vent".into();
            return;
        }
    }
    let costs = building_recipe(kind);
    let is_host = app.net.as_ref().map(|n| n.is_host).unwrap_or(true);
    if !app.is_creative() && !costs.is_empty() && !app.inventory.can_afford(costs) {
        app.status_toast = app.inventory.missing_hint(costs);
        return;
    }
    if app.net.is_none() || is_host {
        if !try_spend_for(app, app.local_player_id, costs) {
            app.status_toast = app.inventory.missing_hint(costs);
            return;
        }
        if let Some(id) = app.world.place_node_machine(kind, machine_id, x, y, facing) {
            if let Some(net) = app.net.as_ref() {
                let _ = net.tx.send(NetCommand::Place {
                    id,
                    kind,
                    x,
                    y,
                    facing,
                    request: false,
                });
            }
            let mut probe = Node::new(kind, x, y, facing);
            if let Some(mid) = machine_id {
                probe.set_machine_id(Some(mid));
            }
            let (cx, cy) = (x + probe.w() * 0.5, y + probe.h() * 0.5);
            if app.storm.point_in_storm(cx, cy, &app.world) {
                app.join_status = format!("Placed #{id} · exposed to storm!");
            } else {
                app.join_status = format!("Placed #{id}");
            }
        } else {
            refund_for(app, app.local_player_id, costs);
            app.status_toast = if kind == BuildingKind::OreNode {
                "Mining drill needs a revealed gas vent".into()
            } else {
                "Can't place here".into()
            };
        }
    } else if let Some(net) = app.net.as_ref() {
        // Host spends authoritatively; local check above only gates the request.
        let _ = net.tx.send(NetCommand::Place {
            id: 0,
            kind,
            x,
            y,
            facing,
            request: true,
        });
        app.join_status = format!("Placing {}…", entry.short());
    }
}

fn remove_building(app: &mut App, id: u32, refund: bool) {
    let is_host = app.net.as_ref().map(|n| n.is_host).unwrap_or(true);
    let kind = app.world.nodes.get(&id).map(|n| n.kind);
    if app.net.is_none() || is_host {
        app.world.remove_node(id);
        if refund {
            if let Some(k) = kind {
                refund_for(app, app.local_player_id, building_recipe(k));
            }
        }
        if let Some(net) = app.net.as_ref() {
            let _ = net.tx.send(NetCommand::Remove {
                id,
                request: false,
            });
        }
    } else if let Some(net) = app.net.as_ref() {
        let _ = net.tx.send(NetCommand::Remove { id, request: true });
        app.join_status = "Removing…".into();
    }
}

fn connect_ports_net(
    app: &mut App,
    from: (u32, usize),
    to: (u32, usize),
    path: Vec<(f32, f32)>,
) -> bool {
    let tool = app.ui.selected;
    let is_host = app.net.as_ref().map(|n| n.is_host).unwrap_or(true);
    if app.net.is_none() || is_host {
        let ok = match tool {
            Some(BuildingKind::PowerWire) => {
                if path.len() >= 2 {
                    app.world.connect_power_path(from, to, path)
                } else {
                    app.world.connect_power(from, to)
                }
            }
            _ => connect_with_tool(&mut app.world, from, to, tool).is_some(),
        };
        if ok {
            if let Some(net) = app.net.as_ref() {
                let power = matches!(tool, Some(BuildingKind::PowerWire));
                let _ = net.tx.send(NetCommand::Link {
                    power,
                    from_node: from.0,
                    from_port: from.1,
                    to_node: to.0,
                    to_port: to.1,
                    request: false,
                });
            }
            true
        } else {
            if let Some(hint) = app.world.connect_fail_hint(from, to) {
                app.status_toast = hint.into();
            } else {
                app.status_toast = "Can't connect those ports".into();
            }
            false
        }
    } else if let Some(net) = app.net.as_ref() {
        let power = matches!(tool, Some(BuildingKind::PowerWire));
        let _ = net.tx.send(NetCommand::Link {
            power,
            from_node: from.0,
            from_port: from.1,
            to_node: to.0,
            to_port: to.1,
            request: true,
        });
        true
    } else {
        false
    }
}

fn connect_with_tool(
    world: &mut World,
    from: (u32, usize),
    to: (u32, usize),
    tool: Option<BuildingKind>,
) -> Option<(bool, (u32, usize), (u32, usize))> {
    match tool {
        Some(BuildingKind::PowerWire) => {
            let before = world.links.len();
            if !world.connect_power(from, to) {
                return None;
            }
            let l = world.links.get(before)?;
            Some((true, (l.from_node, l.from_port), (l.to_node, l.to_port)))
        }
        _ => None,
    }
}

fn handle_hud_input(app: &mut App, mouse: (f32, f32), wx: f32, wy: f32) {
    // Finish palette → hotbar drops even while the build menu is open.
    if is_mouse_button_released(MouseButton::Left) {
        if let Some(entry) = app.ui.palette_entry() {
            app.ui.set_palette_entry(None);
            let dx = mouse.0 - app.ui.palette_drag_origin.0;
            let dy = mouse.1 - app.ui.palette_drag_origin.1;
            let dragged = dx * dx + dy * dy > 64.0;
            if let Some(i) = point_in_hotbar(mouse.0, mouse.1) {
                app.ui.set_hotbar_entry(i, Some(entry));
                app.ui.hotbar_index = i;
            } else if !dragged {
                // Click (not drag): equip and close menu.
                app.ui.select_entry(entry);
                app.ui.set_hotbar_entry(app.ui.hotbar_index, Some(entry));
                app.ui.close_build();
                app.ui.wire_from = None;
            }
            return;
        }
        if let Some(from) = app.ui.hotbar_drag_from.take() {
            let dx = mouse.0 - app.ui.hotbar_drag_origin.0;
            let dy = mouse.1 - app.ui.hotbar_drag_origin.1;
            let dragged = dx * dx + dy * dy > 64.0;
            if let Some(to) = point_in_hotbar(mouse.0, mouse.1) {
                if dragged && to != from {
                    app.ui.hotbar.swap(from, to);
                    app.ui.hotbar_machine.swap(from, to);
                }
                app.ui.hotbar_index = to;
                if let Some(entry) = app.ui.hotbar_entry(to) {
                    app.ui.select_entry(entry);
                } else {
                    app.ui.clear_tool();
                }
            } else if dragged && !point_in_hud_chrome(mouse.0, mouse.1) {
                // Dragged off the bar → clear slot.
                app.ui.set_hotbar_entry(from, None);
                if app.ui.hotbar_index == from {
                    app.ui.clear_tool();
                }
            } else {
                app.ui.hotbar_index = from;
                if let Some(entry) = app.ui.hotbar_entry(from) {
                    app.ui.select_entry(entry);
                } else {
                    app.ui.clear_tool();
                }
            }
            return;
        }
    }

    if handle_context_menu_input(app, mouse) {
        return;
    }

    // Dismiss overlay when clicking outside its panel (wheel still clickable).
    if app.ui.overlay.is_some() && is_mouse_button_pressed(MouseButton::Left) {
        if point_in_tool_button(mouse.0, mouse.1).is_none() {
            let w = s(560.0);
            let h = s(400.0);
            let x = (screen_width() - w) * 0.5;
            let y = (screen_height() - h) * 0.5 - s(40.0);
            let inside =
                mouse.0 >= x && mouse.0 <= x + w && mouse.1 >= y && mouse.1 <= y + h;
            if !inside {
                app.ui.overlay = None;
                return;
            }
        }
    }

    if app.ui.panning {
        return;
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(tool) = point_in_tool_button(mouse.0, mouse.1) {
            app.ui.activate_corner(tool);
            return;
        }
        if let Some(i) = point_in_hotbar(mouse.0, mouse.1) {
            app.ui.context_menu = None;
            if app.ui.build_open {
                // While B is open, clicking a slot just highlights it as drop target.
                app.ui.hotbar_index = i;
            } else if app.ui.hotbar[i].is_some() {
                app.ui.hotbar_drag_from = Some(i);
                app.ui.hotbar_drag_origin = mouse;
                app.ui.hotbar_index = i;
                if let Some(entry) = app.ui.hotbar_entry(i) {
                    app.ui.select_entry(entry);
                }
                app.ui.wire_from = None;
            } else {
                // Empty slot: clear tool / select empty.
                app.ui.hotbar_index = i;
                app.ui.clear_tool();
            }
            return;
        }
    }

    // Right-click on hotbar slot clears it.
    if is_mouse_button_pressed(MouseButton::Right) {
        if let Some(i) = point_in_hotbar(mouse.0, mouse.1) {
            app.ui.set_hotbar_entry(i, None);
            if app.ui.hotbar_index == i {
                app.ui.clear_tool();
            }
            return;
        }
    }

    // World right-click: mine / remove (Factorio-style). No Delete key needed.
    if !app.ui.build_open
        && !app.ui.inventory_open
        && is_mouse_button_pressed(MouseButton::Right)
        && !point_in_hud_chrome(mouse.0, mouse.1)
    {
        // Belt / wire tools: RMB erases under the cursor (keep tool equipped).
        // Wire routing: undo last corner first (handled below) — don't cancel here.
        if app.ui.wire_from.take().is_some() {
            return;
        }
        if app.ui.selected == Some(BuildingKind::Conveyor) {
            let (tx, ty) = world_to_tile(wx, wy);
            if app.world.remove_belt_at(tx, ty) {
                refund_for(app, app.local_player_id, belt_recipe());
                app.status_toast = "Belt removed".into();
            }
            return;
        }
        if app.ui.selected == Some(BuildingKind::PowerWire) {
            // RMB while routing: undo last corner, or cancel if only the start remains.
            if let Some(paint) = app.ui.wire_paint.as_mut() {
                if paint.points.len() > 1 {
                    paint.points.pop();
                } else {
                    app.ui.wire_paint = None;
                }
                return;
            }
            if app.world.remove_wire_at(wx, wy) {
                app.status_toast = "Wire removed".into();
            }
            return;
        }
        if app.ui.selected.take().is_some() {
            // Right-click cancels other place tools first.
            return;
        }
        if let Some(id) = app.world.hit_node(wx, wy) {
            remove_building(app, id, true);
            return;
        }
        let (tx, ty) = world_to_tile(wx, wy);
        if app.world.remove_belt_at(tx, ty) {
            refund_for(app, app.local_player_id, belt_recipe());
            app.status_toast = "Belt removed".into();
            return;
        }
        // Empty ground: soft context (build / clear).
        app.ui.context_menu = Some(ContextMenu {
            sx: mouse.0,
            sy: mouse.1,
            target: ContextTarget::Empty,
        });
    }
}

fn context_items(target: ContextTarget) -> Vec<(&'static str, ContextAction)> {
    match target {
        ContextTarget::Empty => vec![
            ("Build menu", ContextAction::OpenBuild),
            ("Inventory", ContextAction::OpenInventory),
            ("Clear tool", ContextAction::ClearTool),
        ],
    }
}

#[derive(Clone, Copy)]
enum ContextAction {
    OpenBuild,
    OpenInventory,
    ClearTool,
}

fn context_menu_rect(menu: &ContextMenu) -> Rect {
    let n = context_items(menu.target).len() as f32;
    let w = 168.0;
    let h = 10.0 + n * 34.0;
    let mut x = menu.sx;
    let mut y = menu.sy;
    if x + w > screen_width() - 8.0 {
        x = screen_width() - w - 8.0;
    }
    if y + h > screen_height() - 8.0 {
        y = screen_height() - h - 8.0;
    }
    Rect { x, y, w, h }
}

fn handle_context_menu_input(app: &mut App, mouse: (f32, f32)) -> bool {
    let Some(menu) = app.ui.context_menu.clone() else {
        return false;
    };
    let r = context_menu_rect(&menu);
    let items = context_items(menu.target);

    if is_mouse_button_pressed(MouseButton::Left) {
        let inside = mouse.0 >= r.x
            && mouse.0 <= r.x + r.w
            && mouse.1 >= r.y
            && mouse.1 <= r.y + r.h;
        if !inside {
            app.ui.context_menu = None;
            return true;
        }
        for (i, (_, action)) in items.iter().enumerate() {
            let y = r.y + 6.0 + i as f32 * 34.0;
            if mouse.1 >= y && mouse.1 <= y + 30.0 {
                apply_context_action(app, menu.target, *action);
                app.ui.context_menu = None;
                return true;
            }
        }
        return true;
    }
    if is_mouse_button_pressed(MouseButton::Right) {
        app.ui.context_menu = None;
        return true;
    }
    true // swallow world input while open
}

fn apply_context_action(app: &mut App, _target: ContextTarget, action: ContextAction) {
    match action {
        ContextAction::OpenBuild => app.ui.open_build(),
        ContextAction::OpenInventory => app.ui.open_inventory(),
        ContextAction::ClearTool => app.ui.clear_tool(),
    }
}

fn handle_world_input(app: &mut App, mouse: (f32, f32), wx: f32, wy: f32) {
    if app.ui.panning || point_in_hud_chrome(mouse.0, mouse.1) {
        return;
    }

    // Click selected hotbar slot again to unequip (handled when press selects;
    // toggle here on press over nothing with same selection — skip).

    // Conveyor tool: drag-paint Factorio-style belt tiles on the grid.
    // (RMB erase is handled in handle_hud_input so it isn't eaten by tool-cancel.)
    if app.ui.selected == Some(BuildingKind::Conveyor) {
        if is_mouse_button_down(MouseButton::Left) {
            let (tx, ty) = world_to_tile(wx, wy);
            if app.ui.belt_paint_last != Some((tx, ty)) {
                let costs = belt_recipe();
                let is_new = app.world.belt_at(tx, ty).is_none();
                if is_new && !app.is_creative() && !app.inventory.can_afford(costs) {
                    if app.ui.belt_paint_last.is_none() {
                        app.status_toast = app.inventory.missing_hint(costs);
                    }
                } else if app.world.paint_belt(tx, ty, app.ui.place_facing) {
                    if is_new {
                        let _ = try_spend_for(app, app.local_player_id, costs);
                    }
                    app.ui.belt_paint_last = Some((tx, ty));
                } else if app.ui.belt_paint_last.is_none() {
                    app.status_toast = "Can't place belt on a building".into();
                }
            }
            return;
        }
        if is_mouse_button_released(MouseButton::Left) {
            app.ui.belt_paint_last = None;
        }
        return;
    } else {
        app.ui.belt_paint_last = None;
    }

    // Power wire: click ◆ port → click corner anchors → click ◆ port to finish.
    if app.ui.selected == Some(BuildingKind::PowerWire) {
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(port) = app.world.nearest_energy_port(wx, wy, WIRE_PORT_SNAP) {
                if let Some((px, py)) = app
                    .world
                    .nodes
                    .get(&port.0)
                    .and_then(|n| n.port_world(port.1))
                {
                    if app.ui.wire_paint.is_none() {
                        app.ui.wire_paint = Some(WirePaint {
                            from: port,
                            points: vec![(px, py)],
                        });
                    } else if let Some(paint) = app.ui.wire_paint.take() {
                        if port != paint.from {
                            let mut points = paint.points;
                            points.push((px, py));
                            let reach_ok = {
                                let a = app
                                    .world
                                    .nodes
                                    .get(&paint.from.0)
                                    .and_then(|n| n.port_world(paint.from.1));
                                let b = Some((px, py));
                                match (a, b) {
                                    (Some((ax, ay)), Some((bx, by))) => {
                                        (ax - bx).abs() + (ay - by).abs() <= POWER_WIRE_MAX_REACH
                                    }
                                    _ => false,
                                }
                            };
                            if !reach_ok {
                                app.status_toast = "Wire too long — place a pole".into();
                            } else if connect_ports_net(app, paint.from, port, points) {
                                app.status_toast = "Wire connected".into();
                            }
                        } else {
                            // Clicked start port again — cancel.
                            app.status_toast = "Wire cancelled".into();
                        }
                    }
                }
            } else if let Some(paint) = app.ui.wire_paint.as_mut() {
                let anchor = snap_wire_anchor(wx, wy);
                let last = *paint.points.last().unwrap_or(&anchor);
                let dx = anchor.0 - last.0;
                let dy = anchor.1 - last.1;
                if dx * dx + dy * dy >= (TILE_SIZE * 0.35).powi(2) {
                    paint.points.push(anchor);
                }
            }
        }
        return;
    } else {
        app.ui.wire_paint = None;
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(entry) = app.ui.current_entry() {
            let kind = entry.kind();
            if kind.is_cable() || kind.is_belt_tool() {
                return;
            }
            if kind.is_debug_tool() {
                let zones = app.storm.clear_zones(&app.world);
                if app.world.spawn_debug_at(kind, wx, wy, &zones) {
                    app.status_toast = format!("Spawned {}", kind.short());
                } else {
                    app.status_toast = "Could not spawn (cap reached?)".into();
                }
                return;
            }
            if app.world.hit_node(wx, wy).is_none() {
                let mut probe = Node::new(kind, 0.0, 0.0, app.ui.place_facing);
                if let Some(mid) = entry.machine_id() {
                    probe.set_machine_id(Some(mid));
                }
                let (x, y) =
                    snap_building_xy_size(probe.footprint(), app.ui.place_facing, wx, wy);
                place_building_entry(app, entry, x, y, app.ui.place_facing);
                return;
            }
        }
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(id) = app.world.hit_node(wx, wy) {
            if let Some(n) = app.world.nodes.get(&id) {
                if n.kind.is_cable() {
                    app.ui.wire_from = None;
                    app.ui.drag_node = None;
                    return;
                }
                app.ui.drag_node = Some(id);
                app.ui.drag_off = (wx - n.x, wy - n.y);
                app.ui.wire_from = None;
                app.world.set_node_held(id, true);
            }
        }
    }
    if is_mouse_button_released(MouseButton::Left) {
        if let Some(id) = app.ui.drag_node.take() {
            app.world.set_node_held(id, false);
            if let Some(n) = app.world.nodes.get(&id) {
                // Snap dragged buildings onto the grid.
                let (sx, sy) =
                    snap_building_xy_size(n.footprint(), n.facing, n.x + n.w() * 0.5, n.y + n.h() * 0.5);
                let _ = app.world.try_move_node(id, sx, sy);
                if let Some(n) = app.world.nodes.get(&id) {
                    if let Some(net) = app.net.as_ref() {
                        let _ = net.tx.send(NetCommand::Move {
                            id,
                            x: n.x,
                            y: n.y,
                            request: !net.is_host,
                        });
                    }
                }
            }
        }
    }
    if let Some(id) = app.ui.drag_node {
        if is_mouse_button_down(MouseButton::Left) {
            let nx = wx - app.ui.drag_off.0;
            let ny = wy - app.ui.drag_off.1;
            let _ = app.world.try_move_node(id, nx, ny);
        }
    }
}

fn draw_game(app: &mut App, mouse: (f32, f32), wx: f32, wy: f32) {
    clear_background(BG);
    let zones = app.storm.clear_zones(&app.world);
    draw_ground(app, &app.cam, &zones);
    draw_infinite_grid(&app.cam);
    draw_deposits(
        &app.world,
        &app.cam,
        &app.storm,
        &app.art,
        app.ui.selected == Some(BuildingKind::OreNode),
        &zones,
    );
    draw_coverage_rings(&app.world, &app.cam, &app.ui);
    draw_belt_tiles(&app.world, &app.cam, &app.ui, &app.art, wx, wy);
    draw_power_links(&app.world, &app.cam, &app.ui, wx, wy);
    draw_nests_and_raiders(&app.world, &app.cam, &app.storm, &zones);
    draw_storm_blots(&app.world, &app.cam);
    let hover_id = if point_in_hud_chrome(mouse.0, mouse.1) || app.ui.build_open {
        None
    } else {
        app.world.hit_node(wx, wy).filter(|&id| {
            app.world
                .nodes
                .get(&id)
                .map(|n| !n.kind.is_cable())
                .unwrap_or(false)
        })
    };
    draw_nodes(&app.world, &app.cam, &app.ui, hover_id, &app.art);
    draw_cannon_fx(app);
    draw_combat_shots(&app.world, &app.cam, app.cannon_fx.is_some());
    player::draw_player(
        &app.player,
        app.cam.x,
        app.cam.y,
        app.cam.zoom,
        peer_color(app.local_player_id),
        None,
    );
    draw_placement_ghost(
        &app.world,
        &app.ui,
        &app.cam,
        &app.storm,
        &app.inventory,
        &app.art,
        wx,
        wy,
        app.is_creative(),
    );
    draw_peer_cursors(app);
    draw_storm(&app.storm, &zones, &app.cam);
    draw_lightning_fx(app);
    // After storm — darkening under fog made the nebula blow out to white.
    draw_world_lighting(app);
    // Gas plumes above lighting so they stay readable in the dark clear pocket.
    draw_gas_vents(app, &zones);
    // Deferred GL readback — only when saving, never on a timer.
    if app.pending_preview_capture {
        // 1px low-alpha grid vanishes under ambient darken + thumbnail resize;
        // stamp a readable grid on top just for the snapshot.
        draw_infinite_grid_ex(&app.cam, 2.0, 2.4, 160, 200);
    }
    app.flush_pending_preview_capture();
    draw_controls_chip(app);
    if app.settings.show_fps {
        let fps = get_fps();
        let ups = app.measured_ups.round() as i32;
        let label = format!("{fps} FPS · {ups} UPS");
        let tw = measure_text(&label, None, 18, 1.0).width;
        draw_text(
            &label,
            screen_width() - tw - 16.0,
            28.0,
            18.0,
            TEXT_DIM,
        );
    }
    if let Some(net) = app.net.as_ref() {
        let y0 = if app.settings.show_fps { 52.0 } else { 28.0 };
        let line = if net.is_host {
            format!("Host · {}", app.host_code)
        } else if !app.join_status.is_empty() {
            app.join_status.clone()
        } else {
            "Online".into()
        };
        let tw = measure_text(&line, None, 16, 1.0).width;
        draw_text(
            &line,
            screen_width() - tw - 16.0,
            y0,
            16.0,
            if net.is_host { TEXT_DIM } else { ACCENT },
        );
    }
    // Build menu under the hotbar so slots stay visible as drop targets.
    if app.ui.build_open {
        draw_and_handle_build_menu(app, mouse);
    }
    if app.ui.inventory_open {
        draw_and_handle_inventory(app, mouse);
    }
    if let Some(overlay) = app.ui.overlay {
        draw_corner_overlay(overlay, &app.world, mouse, &mut app.ui);
    }
    draw_hotbar(&app.ui, mouse);
    draw_tool_dock(app, mouse);
    if app.ui.context_menu.is_some() {
        draw_context_menu(&app.ui, mouse);
    }
    draw_drag_ghost(&app.ui, mouse);
}

fn capture_preview_now() -> Option<Image> {
    // Full framebuffer readback is expensive — only call on save, never per-frame.
    let full = get_screen_data();
    Some(process_preview_image(&full, 800))
}

fn process_preview_image(src: &Image, max_w: u32) -> Image {
    let w = src.width as u32;
    let h = src.height as u32;
    let mut rgba = image::RgbaImage::from_raw(w, h, src.bytes.clone())
        .unwrap_or_else(|| image::RgbaImage::new(1, 1));
    image::imageops::flip_vertical_in_place(&mut rgba);
    let (dw, dh) = if w > max_w {
        let scale = max_w as f32 / w as f32;
        (max_w, ((h as f32) * scale).round().max(1.0) as u32)
    } else {
        (w, h)
    };
    // Triangle keeps thin grid lines; Nearest often drops 1px features entirely.
    let resized = if dw != w || dh != h {
        image::imageops::resize(&rgba, dw, dh, image::imageops::FilterType::Triangle)
    } else {
        rgba
    };
    Image {
        width: resized.width() as u16,
        height: resized.height() as u16,
        bytes: resized.into_raw(),
    }
}

fn save_preview_png(path: &std::path::Path, img: &Image) -> Result<(), String> {
    image::save_buffer(
        path,
        &img.bytes,
        img.width as u32,
        img.height as u32,
        image::ColorType::Rgba8,
    )
    .map_err(|e| e.to_string())
}

fn titled_menu_panel(
    title: &str,
    panel_w: f32,
    panel_h: f32,
) -> (f32, f32, f32, f32, f32, f32) {
    let title_gap = 64.0;
    let ox = (screen_width() - panel_w) * 0.5;
    let oy = ((screen_height() - (title_gap + panel_h)) * 0.5).max(28.0);
    let tw = measure_text(title, None, 44, 1.0).width;
    let tx = ox + (panel_w - tw) * 0.5;
    draw_rectangle(
        tx - 18.0,
        oy + 4.0,
        tw + 36.0,
        48.0,
        Color::from_rgba(8, 12, 18, 150),
    );
    draw_text(title, tx, oy + 38.0, 44.0, UI_CYAN);
    draw_rectangle(tx, oy + 46.0, tw * 0.32, 3.0, UI_AMBER);
    let px = ox;
    let py = oy + title_gap;
    ui_chrome::panel(px, py, panel_w, panel_h);
    let pad = 28.0;
    (px, py, panel_w, panel_h, pad, pad)
}

fn tick_storm_lightning(app: &mut App, dt: f32) {
    app.lightning_cd -= dt;
    if app.lightning_cd > 0.0 {
        return;
    }
    let seed = app.storm.time * 11.17 + app.lightning_fx.len() as f32 * 3.9;
    // Ambient bolts are frequent; damaging hits are a subset.
    let q = app.settings.effect_quality;
    let wait_mul = match q {
        EffectQuality::Low => 2.2,
        EffectQuality::Medium => 1.0,
        EffectQuality::High => 0.65,
    };
    let wait = (0.35 + storm_hash01(seed) * 0.85) * wait_mul;
    app.lightning_cd = wait;

    if matches!(q, EffectQuality::Low) && storm_hash01(seed + 0.7) < 0.55 {
        return;
    }

    let zones = app.storm.clear_zones(&app.world);
    let intensity = 0.55 + storm_hash01(seed + 2.2) * 0.85;

    // Ambient strike somewhere in the storm (often near the camera for visibility).
    let (tx, ty) = ambient_strike_point(app, seed + 5.0, &zones);
    app.storm.trigger_flash(tx, ty, intensity);
    spawn_lightning_bolt(app, tx, ty - 520.0 - storm_hash01(seed + 8.0) * 240.0, tx, ty, seed, 0.28, 2.4);

    // Chance of a second ambient fork nearby.
    let fork_chance = match q {
        EffectQuality::Low => 0.85,
        EffectQuality::Medium => 0.55,
        EffectQuality::High => 0.35,
    };
    if storm_hash01(seed + 12.0) > fork_chance {
        let (tx2, ty2) = ambient_strike_point(app, seed + 15.0, &zones);
        spawn_lightning_bolt(
            app,
            tx2,
            ty2 - 380.0 - storm_hash01(seed + 16.0) * 160.0,
            tx2,
            ty2,
            seed + 20.0,
            0.22,
            1.8,
        );
        if storm_hash01(seed + 21.0) > 0.7 {
            app.storm.trigger_flash(tx2, ty2, intensity * 0.65);
        }
    }

    // Sometimes the storm hits an exposed building (rare accent — nests are the real threat).
    let victims: Vec<(u32, f32, f32)> = app
        .world
        .nodes
        .iter()
        .filter_map(|(&id, n)| {
            if n.kind.is_cable() {
                return None;
            }
            let (cx, cy) = n.center();
            if app.storm.in_clear(cx, cy, &zones) {
                None
            } else {
                Some((id, cx, cy))
            }
        })
        .collect();
    if victims.is_empty() || storm_hash01(seed + 30.0) < 0.88 {
        return;
    }

    let pick = (storm_hash01(seed + 31.0) * victims.len() as f32).floor() as usize;
    let (id, cx, cy) = victims[pick.min(victims.len() - 1)];
    spawn_lightning_bolt(
        app,
        cx + (storm_hash01(seed + 32.0) - 0.5) * 60.0,
        cy - 480.0 - storm_hash01(seed + 33.0) * 200.0,
        cx,
        cy,
        seed + 40.0,
        0.4,
        3.2,
    );
    app.storm.trigger_flash(cx, cy, 1.15);

    let destroyed = if let Some(n) = app.world.nodes.get_mut(&id) {
        n.hp = (n.hp - LIGHTNING_DAMAGE).max(0.0);
        n.hp <= 0.0
    } else {
        false
    };
    if destroyed {
        remove_building(app, id, false);
        app.status_toast = "Storm lightning destroyed a building!".into();
    }
}

fn ambient_strike_point(app: &App, seed: f32, zones: &[(f32, f32, f32)]) -> (f32, f32) {
    // Bias toward the view so players see the show.
    let base_r = app.storm.radius * STORM_HARD_CLEAR_SCALE;
    for attempt in 0..8 {
        let s = seed + attempt as f32 * 17.3;
        let ang = storm_hash01(s) * std::f32::consts::TAU;
        let dist = base_r * (1.15 + storm_hash01(s + 1.0) * 2.2);
        let mut x = app.storm.cx + ang.cos() * dist + app.cam.x * 0.15;
        let mut y = app.storm.cy + ang.sin() * dist + app.cam.y * 0.15;
        // Pull toward camera a bit.
        x = x * 0.55 + app.cam.x * 0.45;
        y = y * 0.55 + app.cam.y * 0.45;
        if !app.storm.in_clear(x, y, zones) {
            return (x, y);
        }
    }
    let ang = storm_hash01(seed) * std::f32::consts::TAU;
    (
        app.cam.x + ang.cos() * base_r * 1.4,
        app.cam.y + ang.sin() * base_r * 1.4,
    )
}

fn spawn_lightning_bolt(
    app: &mut App,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    seed: f32,
    life: f32,
    width: f32,
) {
    let segs = 10 + (storm_hash01(seed) * 6.0) as usize;
    let points = jagged_polyline(x0, y0, x1, y1, segs, 55.0, seed);
    let mut branches = Vec::new();
    let branch_count = 1 + (storm_hash01(seed + 50.0) * 2.5) as usize;
    for b in 0..branch_count {
        if points.len() < 4 {
            break;
        }
        let idx = 2 + ((storm_hash01(seed + 60.0 + b as f32) * (points.len() - 3) as f32) as usize);
        let idx = idx.min(points.len() - 2);
        let (px, py) = points[idx];
        let (nx, ny) = points[idx + 1];
        let dx = nx - px;
        let dy = ny - py;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let px_dir = -dy / len;
        let py_dir = dx / len;
        let side = if storm_hash01(seed + 70.0 + b as f32) > 0.5 {
            1.0
        } else {
            -1.0
        };
        let blen = 70.0 + storm_hash01(seed + 80.0 + b as f32) * 140.0;
        let ex = px + dx * 0.3 + px_dir * side * blen;
        let ey = py + dy * 0.3 + py_dir * side * blen;
        let bsegs = 4 + (storm_hash01(seed + 90.0 + b as f32) * 3.0) as usize;
        branches.push(jagged_polyline(px, py, ex, ey, bsegs, 28.0, seed + 100.0 + b as f32));
    }
    app.lightning_fx.push(LightningFx {
        points,
        branches,
        life,
        max_life: life,
        width,
    });
}

fn jagged_polyline(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    segs: usize,
    jag: f32,
    seed: f32,
) -> Vec<(f32, f32)> {
    let segs = segs.max(2);
    let mut pts = Vec::with_capacity(segs + 1);
    for i in 0..=segs {
        let t = i as f32 / segs as f32;
        let mut x = x0 + (x1 - x0) * t;
        let mut y = y0 + (y1 - y0) * t;
        if i > 0 && i < segs {
            let s = seed + i as f32 * 19.7;
            let dx = x1 - x0;
            let dy = y1 - y0;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            let px = -dy / len;
            let py = dx / len;
            let amp = jag * (1.0 - (t - 0.5).abs() * 1.2) * (0.45 + storm_hash01(s) * 0.9);
            let off = (storm_hash01(s + 3.0) - 0.5) * 2.0 * amp;
            x += px * off;
            y += py * off;
        }
        pts.push((x, y));
    }
    pts
}

fn tick_lightning_fx(app: &mut App, dt: f32) {
    for fx in &mut app.lightning_fx {
        fx.life -= dt;
    }
    app.lightning_fx.retain(|fx| fx.life > 0.0);
}

/// Snap screen coords to the same camera-locked mosaic as the storm/cannon shaders.
fn snap_fx_screen(cam: &Cam, sx: f32, sy: f32) -> (f32, f32) {
    let p = FX_PIXEL;
    let lx = ((sx + cam.x * cam.zoom) / p).floor() * p + p * 0.5;
    let ly = ((sy + cam.y * cam.zoom) / p).floor() * p + p * 0.5;
    (lx - cam.x * cam.zoom, ly - cam.y * cam.zoom)
}

fn draw_fx_pixel_cell(cx: f32, cy: f32, color: Color) {
    let p = FX_PIXEL;
    draw_rectangle(cx - p * 0.5, cy - p * 0.5, p + 0.5, p + 0.5, color);
}

fn draw_fx_pixel_disc(cam: &Cam, sx: f32, sy: f32, radius: f32, color: Color) {
    let (cx, cy) = snap_fx_screen(cam, sx, sy);
    let cells = (radius / FX_PIXEL).ceil().max(1.0) as i32;
    let r2 = cells * cells;
    for dy in -cells..=cells {
        for dx in -cells..=cells {
            if dx * dx + dy * dy <= r2 {
                draw_fx_pixel_cell(
                    cx + dx as f32 * FX_PIXEL,
                    cy + dy as f32 * FX_PIXEL,
                    color,
                );
            }
        }
    }
}

fn draw_fx_pixel_line(cam: &Cam, sx0: f32, sy0: f32, sx1: f32, sy1: f32, thickness: f32, color: Color) {
    let (x0, y0) = snap_fx_screen(cam, sx0, sy0);
    let (x1, y1) = snap_fx_screen(cam, sx1, sy1);
    let dx = x1 - x0;
    let dy = y1 - y0;
    let dist = (dx * dx + dy * dy).sqrt();
    let steps = ((dist / FX_PIXEL).ceil() as i32).max(1);
    let thick = ((thickness / FX_PIXEL).ceil().max(1.0) as i32 - 1) / 2;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let (cx, cy) = snap_fx_screen(cam, x0 + dx * t, y0 + dy * t);
        for oy in -thick..=thick {
            for ox in -thick..=thick {
                draw_fx_pixel_cell(
                    cx + ox as f32 * FX_PIXEL,
                    cy + oy as f32 * FX_PIXEL,
                    color,
                );
            }
        }
    }
}

fn draw_lightning_polyline(cam: &Cam, pts: &[(f32, f32)], width: f32, a: f32) {
    if pts.len() < 2 {
        return;
    }
    // Skip bolts whose entire polyline is off-screen.
    let mut any = false;
    for &(wx, wy) in pts {
        if cam.world_circle_visible(wx, wy, width * 4.0, 32.0) {
            any = true;
            break;
        }
    }
    if !any {
        return;
    }
    let glow = Color::from_rgba(140, 180, 255, (70.0 * a) as u8);
    let mid = Color::from_rgba(200, 220, 255, (160.0 * a) as u8);
    let core = Color::from_rgba(255, 255, 255, (240.0 * a) as u8);
    for (w, col) in [
        (width * 3.2, glow),
        (width * 1.6, mid),
        (width * 0.7, core),
    ] {
        let thick = (w * cam.zoom.max(0.5)).max(FX_PIXEL);
        for i in 0..pts.len() - 1 {
            let (sx0, sy0) = cam.world_to_screen(pts[i].0, pts[i].1);
            let (sx1, sy1) = cam.world_to_screen(pts[i + 1].0, pts[i + 1].1);
            draw_fx_pixel_line(cam, sx0, sy0, sx1, sy1, thick, col);
        }
    }
}

fn draw_lightning_fx(app: &App) {
    if matches!(app.settings.effect_quality, EffectQuality::Low) {
        return;
    }
    for fx in &app.lightning_fx {
        let a = (fx.life / fx.max_life).clamp(0.0, 1.0);
        // Brief bright hold then falloff.
        let a = if a > 0.7 { 1.0 } else { a / 0.7 };
        draw_lightning_polyline(&app.cam, &fx.points, fx.width, a);
        for branch in &fx.branches {
            draw_lightning_polyline(&app.cam, branch, fx.width * 0.55, a * 0.85);
        }
        if let Some(&(x, y)) = fx.points.last() {
            let (sx, sy) = app.cam.world_to_screen(x, y);
            draw_fx_pixel_disc(
                &app.cam,
                sx,
                sy,
                (14.0 + fx.width * 2.0) * a * app.cam.zoom,
                Color::from_rgba(220, 235, 255, (100.0 * a) as u8),
            );
            draw_fx_pixel_disc(
                &app.cam,
                sx,
                sy,
                (6.0 + fx.width) * a * app.cam.zoom,
                Color::from_rgba(255, 255, 255, (180.0 * a) as u8),
            );
        }
    }
    // No full-screen flash — illumination stays inside the storm fog shader only.
}

fn draw_storm(storm: &Storm, zones: &[(f32, f32, f32)], cam: &Cam) {
    if let Some(mat) = storm.material.as_ref() {
        mat.set_uniform("ScreenSize", vec2(screen_width(), screen_height()));
        mat.set_uniform("CamPos", vec2(cam.x, cam.y));
        mat.set_uniform("CamZoom", cam.zoom);
        mat.set_uniform("Time", storm.time);
        // ~8 screen px mosaic — obvious pixel filter without the old giant chunks.
        let pixel_size: f32 = FX_PIXEL;
        mat.set_uniform("PixelSize", pixel_size);
        let mut totems = [vec4(0.0, 0.0, 0.0, 0.0); STORM_MAX_TOTEMS];
        for (i, &(cx, cy, radius)) in zones.iter().take(STORM_MAX_TOTEMS).enumerate() {
            totems[i] = vec4(cx, cy, radius, 1.0);
        }
        mat.set_uniform_array("Totems", &totems);
        let mut flashes = [vec4(0.0, 0.0, 0.0, 0.0); STORM_MAX_FLASHES];
        for (i, &(x, y, inten, rad)) in storm.flashes.iter().enumerate() {
            flashes[i] = vec4(x, y, inten, rad);
        }
        mat.set_uniform_array("Flashes", &flashes);
        gl_use_material(mat);
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), WHITE);
        gl_use_default_material();
        return;
    }

    // Shader missing (common on some Windows GL drivers). NEVER draw tens of thousands of
    // CPU mosaic rects — that tanks FPS to ~10–15. One fullscreen tint only.
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(32, 36, 52, 140),
    );
    let _ = (zones, cam);
}

fn draw_world_lighting(app: &App) {
    let Some(mat) = app.lighting.as_ref() else {
        return;
    };
    let mut lights = [vec4(0.0, 0.0, 0.0, 0.0); LIGHT_MAX];
    let mut i = 0usize;
    let mut push = |x: f32, y: f32, radius: f32, intensity: f32| {
        if i >= LIGHT_MAX || intensity <= 0.01 {
            return;
        }
        lights[i] = vec4(x, y, radius, intensity);
        i += 1;
    };

    for n in app.world.nodes.values() {
        let (cx, cy) = n.center();
        match n.kind {
            BuildingKind::PowerPole if n.working => {
                push(cx, cy, POLE_RADIUS * 1.05, 0.95);
            }
            BuildingKind::Solar if n.working => {
                push(cx, cy, 90.0, 0.35);
            }
            BuildingKind::OreNode | BuildingKind::Smelter | BuildingKind::Assembler if n.working => {
                push(cx, cy, 70.0, 0.28);
            }
            BuildingKind::Turret if n.powered => {
                let (ux, uy) = (n.aim_angle.sin(), -n.aim_angle.cos());
                let mx = cx + ux * 36.0;
                let my = cy + uy * 36.0;
                push(cx, cy, 55.0, 0.18);
                if n.charge > 0.05 {
                    push(mx, my, 18.0 + n.charge * 22.0, 0.2 + n.charge * 0.55);
                } else if n.cooldown > 0.0 {
                    let t = (n.cooldown / TURRET_FIRE_INTERVAL).clamp(0.0, 1.0);
                    push(mx, my, 16.0, 0.15 * t);
                }
            }
            BuildingKind::Totem if n.powered => {
                push(cx, cy, 110.0, 0.30);
            }
            BuildingKind::PowerWire if n.powered => {
                push(cx, cy, 36.0, 0.12);
            }
            _ => {}
        }
    }

    mat.set_uniform("ScreenSize", vec2(screen_width(), screen_height()));
    mat.set_uniform("CamPos", vec2(app.cam.x, app.cam.y));
    mat.set_uniform("CamZoom", app.cam.zoom);
    // Float1 must be f32 (4 bytes). f64 literals silently fail and leave Ambient=0.
    let ambient: f32 = 0.52;
    mat.set_uniform("Ambient", ambient);
    mat.set_uniform_array("Lights", &lights);
    gl_use_material(mat);
    // Vertex color is unused by the lighting shader; keep alpha=1 for the quad itself.
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(255, 255, 255, 255),
    );
    gl_use_default_material();
}

/// Thin gameplay range rings only — no filled fake light discs.
fn draw_coverage_rings(world: &World, cam: &Cam, ui: &Ui) {
    let show_power = matches!(
        ui.selected,
        Some(BuildingKind::PowerWire | BuildingKind::PowerPole | BuildingKind::Solar)
    );
    let show_totem = matches!(ui.selected, Some(BuildingKind::Totem));
    let show_turret = matches!(
        ui.selected,
        Some(
            BuildingKind::Turret
                | BuildingKind::BallisticTurret
                | BuildingKind::LaserTurret
        )
    );

    for n in world.nodes.values() {
        let (cx, cy) = n.center();
        match n.kind {
            BuildingKind::PowerPole if show_power => {
                if !cam.world_circle_visible(cx, cy, POLE_RADIUS, 16.0) {
                    continue;
                }
                let (sx, sy) = cam.world_to_screen(cx, cy);
                let r = POLE_RADIUS * cam.zoom;
                draw_circle_lines(
                    sx,
                    sy,
                    r,
                    1.0,
                    if n.working {
                        with_alpha(POWER_C, 0.28)
                    } else {
                        Color::from_rgba(120, 120, 130, 40)
                    },
                );
            }
            BuildingKind::Totem if show_totem || n.powered => {
                let wr = TOTEM_CLEAR_RADIUS * STORM_HARD_CLEAR_SCALE;
                if !cam.world_circle_visible(cx, cy, wr, 16.0) {
                    continue;
                }
                let (sx, sy) = cam.world_to_screen(cx, cy);
                let r = wr * cam.zoom;
                draw_circle_lines(
                    sx,
                    sy,
                    r,
                    1.1,
                    if n.powered {
                        Color::from_rgba(170, 140, 240, 70)
                    } else {
                        Color::from_rgba(100, 90, 120, 35)
                    },
                );
            }
            BuildingKind::Turret
            | BuildingKind::BallisticTurret
            | BuildingKind::LaserTurret
                if show_turret || n.powered =>
            {
                if !cam.world_circle_visible(cx, cy, TURRET_RANGE, 16.0) {
                    continue;
                }
                let (sx, sy) = cam.world_to_screen(cx, cy);
                let r = TURRET_RANGE * cam.zoom;
                draw_circle_lines(
                    sx,
                    sy,
                    r,
                    1.0,
                    if n.powered {
                        Color::from_rgba(220, 110, 110, 55)
                    } else {
                        Color::from_rgba(120, 80, 80, 28)
                    },
                );
            }
            _ => {}
        }
    }
}

fn draw_ground(app: &App, cam: &Cam, zones: &[(f32, f32, f32)]) {
    let Some(mat) = app.ground.as_ref() else {
        return;
    };
    let mut totems = [vec4(0.0, 0.0, 0.0, 0.0); STORM_MAX_TOTEMS];
    for (i, &(cx, cy, radius)) in zones.iter().take(STORM_MAX_TOTEMS).enumerate() {
        totems[i] = vec4(cx, cy, radius, 1.0);
    }
    // Pass cam already on the zoom lattice so world↔screen math stays stable.
    let z = cam.zoom.max(1e-4);
    let cx = (cam.x * z).round() / z;
    let cy = (cam.y * z).round() / z;
    mat.set_uniform("ScreenSize", vec2(screen_width(), screen_height()));
    mat.set_uniform("CamPos", vec2(cx, cy));
    mat.set_uniform("CamZoom", z);
    mat.set_uniform("Time", app.storm.time);
    mat.set_uniform_array("Totems", &totems);
    gl_use_material(mat);
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(255, 255, 255, 255),
    );
    gl_use_default_material();
}

fn draw_infinite_grid(cam: &Cam) {
    draw_infinite_grid_ex(
        cam,
        1.0,
        1.35,
        (GRID_MINOR_C.a * 255.0).round() as u8,
        100,
    );
}

fn draw_infinite_grid_ex(cam: &Cam, minor_w: f32, major_w: f32, minor_a: u8, major_a: u8) {
    let (x0, y0) = cam.screen_to_world(0.0, 0.0);
    let (x1, y1) = cam.screen_to_world(screen_width(), screen_height());
    let start_x = ((x0 / GRID_MINOR).floor() as i32) - 1;
    let end_x = ((x1 / GRID_MINOR).ceil() as i32) + 1;
    let start_y = ((y0 / GRID_MINOR).floor() as i32) - 1;
    let end_y = ((y1 / GRID_MINOR).ceil() as i32) + 1;

    let minor = Color::from_rgba(70, 78, 62, minor_a);
    let major = Color::from_rgba(88, 98, 78, major_a.min(70));

    for gx in start_x..=end_x {
        let wx = gx as f32 * GRID_MINOR;
        let (sx, sy0) = cam.world_to_screen(wx, y0);
        let (_, sy1) = cam.world_to_screen(wx, y1);
        // Use exact world→screen (already cam-quantized + rounded). Do NOT snap to
        // FX_PIXEL — that made lines stick then jump while the world slid underneath.
        let is_major = gx.rem_euclid(GRID_MAJOR_EVERY) == 0;
        draw_line(
            sx,
            sy0,
            sx,
            sy1,
            if is_major { major_w } else { minor_w },
            if is_major { major } else { minor },
        );
    }
    for gy in start_y..=end_y {
        let wy = gy as f32 * GRID_MINOR;
        let (sx0, sy) = cam.world_to_screen(x0, wy);
        let (sx1, _) = cam.world_to_screen(x1, wy);
        let is_major = gy.rem_euclid(GRID_MAJOR_EVERY) == 0;
        draw_line(
            sx0,
            sy,
            sx1,
            sy,
            if is_major { major_w } else { minor_w },
            if is_major { major } else { minor },
        );
    }
}

fn draw_power_manhattan(cam: &Cam, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
    let (sx0, sy0) = cam.world_to_screen(x0, y0);
    let (sx1, sy1) = cam.world_to_screen(x1, y1);
    let mx = (sx0 + sx1) * 0.5;
    // Physical copper-style cable: dark sheath + bright core.
    let outer = (4.2 * cam.zoom).clamp(2.5, 6.5);
    let inner = (outer - 1.6).max(1.2);
    let sheath = Color::from_rgba(90, 70, 30, 220);
    // Soft copper glow under sheath.
    draw_line(sx0, sy0, mx, sy0, outer * 1.8, with_alpha(color, 0.12));
    draw_line(mx, sy0, mx, sy1, outer * 1.8, with_alpha(color, 0.12));
    draw_line(mx, sy1, sx1, sy1, outer * 1.8, with_alpha(color, 0.12));
    draw_line(sx0, sy0, mx, sy0, outer, sheath);
    draw_line(mx, sy0, mx, sy1, outer, sheath);
    draw_line(mx, sy1, sx1, sy1, outer, sheath);
    draw_line(sx0, sy0, mx, sy0, inner, color);
    draw_line(mx, sy0, mx, sy1, inner, color);
    draw_line(mx, sy1, sx1, sy1, inner, color);
    // Tick marks show it's a live power run (not a belt).
    draw_manhattan_ticks(sx0, sy0, mx, sy0, sx1, sy1, cam.zoom, color);
}

fn manhattan_segments(
    sx0: f32,
    sy0: f32,
    sx1: f32,
    sy1: f32,
) -> [(f32, f32, f32, f32); 3] {
    let mx = (sx0 + sx1) * 0.5;
    [(sx0, sy0, mx, sy0), (mx, sy0, mx, sy1), (mx, sy1, sx1, sy1)]
}

fn draw_manhattan_ticks(
    sx0: f32,
    sy0: f32,
    mx: f32,
    _my0: f32,
    sx1: f32,
    sy1: f32,
    zoom: f32,
    color: Color,
) {
    let segs = manhattan_segments(sx0, sy0, sx1, sy1);
    let _ = mx;
    let step = (28.0 * zoom).clamp(14.0, 36.0);
    let tick = (3.5 * zoom).clamp(2.0, 5.0);
    for (ax, ay, bx, by) in segs {
        let dx = bx - ax;
        let dy = by - ay;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 8.0 {
            continue;
        }
        let (ux, uy) = (dx / len, dy / len);
        let (nx, ny) = (-uy, ux);
        let mut d = step * 0.5;
        while d < len - 2.0 {
            let px = ax + ux * d;
            let py = ay + uy * d;
            draw_line(
                px - nx * tick,
                py - ny * tick,
                px + nx * tick,
                py + ny * tick,
                1.2,
                with_alpha(color, 0.55),
            );
            d += step;
        }
    }
}

fn draw_chevron(cx: f32, cy: f32, dx: f32, dy: f32, size: f32, color: Color) {
    let len = (dx * dx + dy * dy).sqrt().max(1e-3);
    let (ux, uy) = (dx / len, dy / len);
    let (nx, ny) = (-uy, ux);
    let tip_x = cx + ux * size;
    let tip_y = cy + uy * size;
    let back = size * 0.85;
    let wing = size * 0.7;
    let bx = cx - ux * back * 0.15;
    let by = cy - uy * back * 0.15;
    draw_triangle(
        Vec2::new(tip_x, tip_y),
        Vec2::new(bx - nx * wing, by - ny * wing),
        Vec2::new(bx + nx * wing, by + ny * wing),
        color,
    );
}

fn draw_power_polyline(cam: &Cam, pts: &[(f32, f32)], color: Color) {
    if pts.len() < 2 {
        return;
    }
    let outer = (4.2 * cam.zoom).clamp(2.5, 6.5);
    let inner = (outer - 1.6).max(1.2);
    let sheath = Color::from_rgba(90, 70, 30, 220);
    for w in pts.windows(2) {
        let (sx0, sy0) = cam.world_to_screen(w[0].0, w[0].1);
        let (sx1, sy1) = cam.world_to_screen(w[1].0, w[1].1);
        draw_line(sx0, sy0, sx1, sy1, outer * 1.8, with_alpha(color, 0.12));
        draw_line(sx0, sy0, sx1, sy1, outer, sheath);
        draw_line(sx0, sy0, sx1, sy1, inner, color);
    }
}

fn draw_power_links(world: &World, cam: &Cam, ui: &Ui, wx: f32, wy: f32) {
    for l in &world.links {
        let Some(a) = world.nodes.get(&l.from_node) else {
            continue;
        };
        let Some(b) = world.nodes.get(&l.to_node) else {
            continue;
        };
        let Some((ax, ay)) = a.port_world(l.from_port) else {
            continue;
        };
        let Some((bx, by)) = b.port_world(l.to_port) else {
            continue;
        };
        let mid_x = (ax + bx) * 0.5;
        let mid_y = (ay + by) * 0.5;
        let span = ((bx - ax).abs() + (by - ay).abs()) * 0.5 + 32.0;
        if !cam.world_circle_visible(mid_x, mid_y, span, 0.0)
            && !cam.world_circle_visible(ax, ay, 8.0, 0.0)
            && !cam.world_circle_visible(bx, by, 8.0, 0.0)
        {
            continue;
        }
        if l.path.len() >= 2 {
            draw_power_polyline(cam, &l.path, POWER_C);
        } else {
            draw_power_manhattan(cam, ax, ay, bx, by, POWER_C);
        }
    }

    if let Some(paint) = ui.wire_paint.as_ref() {
        let cursor = if world.nearest_energy_port(wx, wy, WIRE_PORT_SNAP).is_some() {
            (wx, wy)
        } else {
            snap_wire_anchor(wx, wy)
        };
        let mut pts = paint.points.clone();
        pts.push(cursor);
        let snap = world.nearest_energy_port(wx, wy, WIRE_PORT_SNAP);
        let ok = snap.map(|p| p != paint.from).unwrap_or(false);
        let col = if ok {
            Color::from_rgba(255, 190, 70, 220)
        } else {
            Color::from_rgba(255, 170, 90, 170)
        };
        draw_power_polyline(cam, &pts, col);
        // Corner anchors
        for &(ax, ay) in paint.points.iter().skip(1) {
            let (sx, sy) = cam.world_to_screen(ax, ay);
            let d = (4.0 * cam.zoom).clamp(3.0, 6.0);
            draw_rectangle(sx - d, sy - d, d * 2.0, d * 2.0, Color::from_rgba(20, 24, 30, 220));
            draw_rectangle_lines(sx - d, sy - d, d * 2.0, d * 2.0, 1.4, col);
        }
        if let Some((sx, sy)) = paint
            .points
            .first()
            .map(|&(x, y)| cam.world_to_screen(x, y))
        {
            draw_circle_lines(
                sx,
                sy,
                POWER_WIRE_MAX_REACH * cam.zoom,
                1.2,
                Color::from_rgba(255, 190, 70, 55),
            );
        }
        let (csx, csy) = cam.world_to_screen(cursor.0, cursor.1);
        draw_circle(csx, csy, (3.5 * cam.zoom).clamp(2.5, 5.0), col);
    }
}

fn snap_wire_anchor(wx: f32, wy: f32) -> (f32, f32) {
    let g = TILE_SIZE * 0.5;
    ((wx / g).round() * g, (wy / g).round() * g)
}

fn draw_belt_tiles(world: &World, cam: &Cam, ui: &Ui, art: &art::Art, wx: f32, wy: f32) {
    let ts = TILE_SIZE * cam.zoom;
    let (min_x, min_y, max_x, max_y) = cam.view_world_aabb(TILE_SIZE);
    for (&(tx, ty), tile) in &world.belt_tiles {
        let (ox, oy) = tile_origin(tx, ty);
        if ox + TILE_SIZE < min_x || ox > max_x || oy + TILE_SIZE < min_y || oy > max_y {
            continue;
        }
        let (sx, sy) = cam.world_to_screen(ox, oy);
        let load = tile.item_count();
        let congested = load >= 8;
        let fill = if congested {
            Color::from_rgba(78, 42, 24, 230)
        } else {
            Color::from_rgba(42, 36, 22, 225)
        };
        let edge = if congested {
            Color::from_rgba(240, 140, 70, 255)
        } else {
            BELT_YELLOW
        };
        let shape = belts::belt_shape(&world.belt_tiles, tx, ty, tile.dir);
        draw_belt_tile_shape(sx, sy, ts, tile.dir, shape, fill, edge);

        for lane in 0..2 {
            for it in &tile.lanes[lane].items {
                let (iwx, iwy) =
                    belts::item_world_pos_shaped(tx, ty, tile.dir, shape, lane, it.progress);
                let (isx, isy) = cam.world_to_screen(iwx, iwy);
                draw_item_chip(art, isx, isy, cam.zoom, it.item);
            }
        }
    }

    // Ghost belt under cursor while paint tool is equipped.
    if ui.selected == Some(BuildingKind::Conveyor) {
        let (tx, ty) = world_to_tile(wx, wy);
        let (ox, oy) = tile_origin(tx, ty);
        let (sx, sy) = cam.world_to_screen(ox, oy);
        let building_hit = world
            .hit_node(ox + TILE_SIZE * 0.5, oy + TILE_SIZE * 0.5)
            .is_some();
        let col = if building_hit {
            Color::from_rgba(220, 70, 70, 100)
        } else {
            Color::from_rgba(210, 170, 55, 90)
        };
        draw_rectangle(sx, sy, ts, ts, col);
        let (cx, cy) = (sx + ts * 0.5, sy + ts * 0.5);
        let (dx, dy) = match ui.place_facing {
            Facing::E => (1.0, 0.0),
            Facing::W => (-1.0, 0.0),
            Facing::S => (0.0, 1.0),
            Facing::N => (0.0, -1.0),
        };
        draw_chevron(cx, cy, dx, dy, ts * 0.28, BELT_YELLOW);
    }
}

fn facing_unit(f: Facing) -> (f32, f32) {
    match f {
        Facing::E => (1.0, 0.0),
        Facing::W => (-1.0, 0.0),
        Facing::S => (0.0, 1.0),
        Facing::N => (0.0, -1.0),
    }
}

fn draw_belt_tile_shape(
    sx: f32,
    sy: f32,
    ts: f32,
    dir: Facing,
    shape: belts::BeltShape,
    _fill: Color,
    _edge: Color,
) {
    let cx = sx + ts * 0.5;
    let cy = sy + ts * 0.5;
    let (fx, fy) = facing_unit(dir);
    let inset = (ts * 0.07).clamp(1.2, 3.5);
    // Concept: dark metal frame + segmented belt + bright yellow direction triangles.
    let frame = Color::from_rgba(42, 46, 54, 255);
    let belt = Color::from_rgba(28, 30, 36, 255);
    let seg = Color::from_rgba(55, 58, 66, 220);
    let arrow = BELT_YELLOW;
    match shape {
        belts::BeltShape::Straight => {
            draw_rectangle(sx, sy, ts, ts, frame);
            draw_rectangle(
                sx + inset,
                sy + inset,
                ts - inset * 2.0,
                ts - inset * 2.0,
                belt,
            );
            // Segment lines across the travel direction (like belt rollers / plates).
            let n = 4;
            for i in 1..n {
                let t = i as f32 / n as f32;
                if fx.abs() > fy.abs() {
                    let x = sx + inset + (ts - inset * 2.0) * t;
                    draw_line(
                        x,
                        sy + inset,
                        x,
                        sy + ts - inset,
                        1.2,
                        seg,
                    );
                } else {
                    let y = sy + inset + (ts - inset * 2.0) * t;
                    draw_line(
                        sx + inset,
                        y,
                        sx + ts - inset,
                        y,
                        1.2,
                        seg,
                    );
                }
            }
            draw_rectangle_lines(sx, sy, ts, ts, 1.8, Color::from_rgba(70, 75, 88, 255));
            // Yellow equilateral direction markers.
            draw_chevron(cx - fx * ts * 0.18, cy - fy * ts * 0.18, fx, fy, ts * 0.18, arrow);
            draw_chevron(cx + fx * ts * 0.12, cy + fy * ts * 0.12, fx, fy, ts * 0.18, arrow);
        }
        belts::BeltShape::CornerLeft | belts::BeltShape::CornerRight => {
            let pts = corner_triangle(sx, sy, ts, dir, shape);
            draw_poly_fill(&pts, frame);
            let (ax, ay) = triangle_centroid(&pts);
            let mut inset_pts = pts;
            for p in &mut inset_pts {
                p.0 = p.0 + (ax - p.0) * 0.22;
                p.1 = p.1 + (ay - p.1) * 0.22;
            }
            draw_poly_fill(&inset_pts, belt);
            draw_poly_outline(&pts, 1.8, Color::from_rgba(70, 75, 88, 255));
            let entry = match shape {
                belts::BeltShape::CornerLeft => belts::facing_left(dir),
                belts::BeltShape::CornerRight => belts::facing_right(dir),
                belts::BeltShape::Straight => unreachable!(),
            };
            let (ix, iy) = facing_unit(belts::facing_opposite(entry));
            let mut bx = ix + fx;
            let mut by = iy + fy;
            let len = (bx * bx + by * by).sqrt().max(1e-3);
            bx /= len;
            by /= len;
            draw_chevron(ax, ay, bx, by, ts * 0.2, arrow);
        }
    }
}

/// Right-triangle covering the tile except the empty inner corner of the bend.
fn corner_triangle(
    sx: f32,
    sy: f32,
    ts: f32,
    exit: Facing,
    shape: belts::BeltShape,
) -> [(f32, f32); 3] {
    let nw = (sx, sy);
    let ne = (sx + ts, sy);
    let sw = (sx, sy + ts);
    let se = (sx + ts, sy + ts);
    let entry = match shape {
        belts::BeltShape::CornerLeft => belts::facing_left(exit),
        belts::BeltShape::CornerRight => belts::facing_right(exit),
        belts::BeltShape::Straight => unreachable!(),
    };
    // Empty corner = opposite(entry) + opposite(exit) (inner elbow of the turn).
    let (dx, dy) = (
        belts::facing_delta(belts::facing_opposite(entry)).0
            + belts::facing_delta(belts::facing_opposite(exit)).0,
        belts::facing_delta(belts::facing_opposite(entry)).1
            + belts::facing_delta(belts::facing_opposite(exit)).1,
    );
    let empty = if dx < 0 && dy < 0 {
        nw
    } else if dx > 0 && dy < 0 {
        ne
    } else if dx < 0 && dy > 0 {
        sw
    } else {
        se
    };
    let all = [nw, ne, se, sw];
    let mut out = [(0.0, 0.0); 3];
    let mut i = 0;
    for p in all {
        if p != empty {
            out[i] = p;
            i += 1;
        }
    }
    out
}

fn triangle_centroid(pts: &[(f32, f32); 3]) -> (f32, f32) {
    (
        (pts[0].0 + pts[1].0 + pts[2].0) / 3.0,
        (pts[0].1 + pts[1].1 + pts[2].1) / 3.0,
    )
}

fn draw_nodes(world: &World, cam: &Cam, ui: &Ui, hover_id: Option<u32>, art: &art::Art) {
    let connect_tool = ui.selected.filter(|k| k.is_cable());
    // Opaque silhouettes — no need to sort every frame; cull off-screen first.
    for (&id, n) in &world.nodes {
        if n.kind.is_cable() {
            continue;
        }
        if !cam.world_rect_visible(n.x, n.y, n.w(), n.h(), 48.0) {
            continue;
        }
        draw_node(
            cam,
            n,
            hover_id == Some(id),
            connect_tool,
            ui.wire_paint.as_ref().map(|p| p.from),
            world,
            id,
            art,
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NodeLod {
    Far,
    Mid,
    Near,
}

fn node_lod(zoom: f32) -> NodeLod {
    if zoom < 0.55 {
        NodeLod::Far
    } else if zoom < 0.95 {
        NodeLod::Mid
    } else {
        NodeLod::Near
    }
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a,
    }
}


fn item_chip_colors(item: Item) -> (Color, Color, Color) {
    match item {
        Item::IronOre => (
            Color::from_rgba(95, 98, 108, 255),
            Color::from_rgba(58, 60, 68, 255),
            Color::from_rgba(200, 150, 90, 200),
        ),
        Item::CopperOre => (
            Color::from_rgba(185, 110, 65, 255),
            Color::from_rgba(120, 70, 40, 255),
            Color::from_rgba(230, 170, 90, 220),
        ),
        Item::Stone => (
            Color::from_rgba(160, 155, 145, 255),
            Color::from_rgba(110, 105, 95, 255),
            Color::from_rgba(200, 195, 185, 200),
        ),
        Item::Coal => (
            Color::from_rgba(55, 58, 65, 255),
            Color::from_rgba(30, 32, 38, 255),
            Color::from_rgba(90, 95, 105, 180),
        ),
        Item::CrudeOil => (
            Color::from_rgba(45, 95, 60, 255),
            Color::from_rgba(25, 55, 35, 255),
            Color::from_rgba(70, 140, 90, 200),
        ),
        Item::IronIngot => (
            INGOT_C,
            Color::from_rgba(55, 62, 72, 255),
            Color::from_rgba(245, 250, 255, 160),
        ),
        Item::CopperIngot => (
            Color::from_rgba(210, 130, 70, 255),
            Color::from_rgba(140, 80, 40, 255),
            Color::from_rgba(255, 200, 120, 160),
        ),
        Item::Slag => (
            Color::from_rgba(90, 85, 75, 255),
            Color::from_rgba(50, 48, 42, 255),
            Color::from_rgba(140, 120, 90, 160),
        ),
        Item::Coke => (
            Color::from_rgba(40, 42, 48, 255),
            Color::from_rgba(20, 22, 26, 255),
            Color::from_rgba(70, 75, 85, 160),
        ),
        Item::Gear | Item::Rivet | Item::Pipe | Item::Frame | Item::ShellCasing => (
            Color::from_rgba(150, 155, 165, 255),
            Color::from_rgba(80, 85, 95, 255),
            Color::from_rgba(210, 215, 225, 160),
        ),
        Item::Wire | Item::CircuitShard | Item::ChargeCell | Item::SolarCell => (
            Color::from_rgba(70, 160, 120, 255),
            Color::from_rgba(30, 90, 70, 255),
            Color::from_rgba(140, 230, 180, 160),
        ),
        Item::Brick | Item::BeltLink | Item::PoleKit => (
            Color::from_rgba(170, 120, 90, 255),
            Color::from_rgba(100, 70, 50, 255),
            Color::from_rgba(220, 170, 120, 160),
        ),
        Item::TotemCore => (
            Color::from_rgba(140, 100, 220, 255),
            Color::from_rgba(70, 50, 120, 255),
            Color::from_rgba(200, 170, 255, 160),
        ),
        Item::ScienceRed => (
            Color::from_rgba(200, 70, 60, 255),
            Color::from_rgba(110, 30, 30, 255),
            Color::from_rgba(255, 140, 110, 160),
        ),
        Item::ScienceGreen => (
            Color::from_rgba(60, 170, 90, 255),
            Color::from_rgba(25, 90, 45, 255),
            Color::from_rgba(140, 230, 160, 160),
        ),
        _ => (
            Color::from_rgba(120, 140, 160, 255),
            Color::from_rgba(60, 70, 85, 255),
            Color::from_rgba(180, 200, 220, 160),
        ),
    }
}

fn item_is_ore(item: Item) -> bool {
    if matches!(
        item,
        Item::IronOre | Item::CopperOre | Item::Stone | Item::Coal
    ) {
        return true;
    }
    if let Some(it) = content::try_content().and_then(|c| c.item(item.as_u16())) {
        if it.category == "raw" {
            return true;
        }
        let id = it.id.to_ascii_lowercase();
        let name = it.name.to_ascii_lowercase();
        return id.contains("_ore") || name.ends_with(" ore") || name.contains(" ore ");
    }
    false
}

fn item_is_ingot(item: Item) -> bool {
    if matches!(item, Item::IronIngot | Item::CopperIngot) {
        return true;
    }
    if let Some(it) = content::try_content().and_then(|c| c.item(item.as_u16())) {
        let id = it.id.to_ascii_lowercase();
        let name = it.name.to_ascii_lowercase();
        return id.contains("ingot") || name.contains("ingot");
    }
    false
}

fn item_tint(item: Item) -> Color {
    item_chip_colors(item).0
}

fn draw_item_chip(art: &art::Art, sx: f32, sy: f32, zoom: f32, item: Item) {
    let r = (5.0 * zoom).clamp(3.2, 7.5);
    if item == Item::CrudeOil {
        let (fill, dark, highlight) = item_chip_colors(item);
        draw_circle(sx + r * 0.05, sy + r * 0.2, r * 1.0, Color::from_rgba(0, 0, 0, 40));
        draw_ellipse(sx, sy + r * 0.15, r * 0.95, r * 0.7, 0.0, dark);
        draw_ellipse(sx - r * 0.1, sy - r * 0.05, r * 0.7, r * 0.5, 0.0, fill);
        draw_ellipse(sx + r * 0.2, sy + r * 0.05, r * 0.35, r * 0.25, 0.0, highlight);
        return;
    }
    if item_is_ore(item) {
        draw_circle(sx + r * 0.05, sy + r * 0.2, r * 0.95, Color::from_rgba(0, 0, 0, 35));
        art::draw_tinted_item(&art.ore, sx, sy, r * 2.15, item_tint(item));
        return;
    }
    if item_is_ingot(item) {
        draw_circle(sx + r * 0.05, sy + r * 0.2, r * 0.9, Color::from_rgba(0, 0, 0, 35));
        art::draw_tinted_item(&art.ingot, sx, sy, r * 2.2, item_tint(item));
        return;
    }
    // Generic faceted rock for other solid items.
    let (rock, rock_d, vein) = item_chip_colors(item);
    let rock_h = Color::from_rgba(
        (rock.r * 255.0 * 1.35).min(255.0) as u8,
        (rock.g * 255.0 * 1.35).min(255.0) as u8,
        (rock.b * 255.0 * 1.35).min(255.0) as u8,
        220,
    );
    draw_circle(sx + r * 0.05, sy + r * 0.25, r * 1.05, Color::from_rgba(0, 0, 0, 40));
    draw_circle(sx - r * 0.2, sy + r * 0.1, r * 0.72, rock_d);
    draw_circle(sx + r * 0.28, sy - r * 0.05, r * 0.58, rock);
    draw_circle(sx + r * 0.05, sy + r * 0.28, r * 0.48, rock_d);
    draw_circle(sx - r * 0.05, sy - r * 0.22, r * 0.42, rock);
    draw_circle(sx - r * 0.3, sy - r * 0.05, r * 0.22, rock_h);
    draw_circle(sx + r * 0.15, sy + r * 0.05, r * 0.12, vein);
    draw_circle(sx + r * 0.35, sy - r * 0.2, r * 0.08, Color::from_rgba(230, 210, 140, 180));
    draw_circle_lines(sx - r * 0.2, sy + r * 0.1, r * 0.72, 1.0, Color::from_rgba(40, 42, 48, 160));
    draw_circle_lines(sx + r * 0.28, sy - r * 0.05, r * 0.58, 1.0, Color::from_rgba(40, 42, 48, 140));
}

fn draw_poly_fill(pts: &[(f32, f32)], color: Color) {
    if pts.len() < 3 {
        return;
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    for &(x, y) in pts {
        cx += x;
        cy += y;
    }
    let n = pts.len() as f32;
    cx /= n;
    cy /= n;
    for i in 0..pts.len() {
        let (x0, y0) = pts[i];
        let (x1, y1) = pts[(i + 1) % pts.len()];
        draw_triangle(Vec2::new(cx, cy), Vec2::new(x0, y0), Vec2::new(x1, y1), color);
    }
}

fn draw_poly_outline(pts: &[(f32, f32)], thickness: f32, color: Color) {
    if pts.len() < 2 {
        return;
    }
    for i in 0..pts.len() {
        let (x0, y0) = pts[i];
        let (x1, y1) = pts[(i + 1) % pts.len()];
        draw_line(x0, y0, x1, y1, thickness, color);
    }
}


/// Unique building silhouette inside the AABB — readable at any zoom, zero art.
fn draw_building_silhouette(
    kind: BuildingKind,
    facing: Facing,
    sx: f32,
    sy: f32,
    w: f32,
    h: f32,
    fill: Color,
    border: Color,
    accent: Color,
    lit: bool,
    detail: bool,
) {
    let cx = sx + w * 0.5;
    let cy = sy + h * 0.5;
    let edge = (w.min(h) * 0.04).clamp(1.3, 3.2);
    let (fx, fy) = match facing {
        Facing::E => (1.0, 0.0),
        Facing::W => (-1.0, 0.0),
        Facing::S => (0.0, 1.0),
        Facing::N => (0.0, -1.0),
    };
    match kind {
        BuildingKind::Solar => {
            // Grid of PV cells — pure top-down panel array.
            let frame = Color::from_rgba(36, 42, 52, 255);
            draw_rectangle(sx, sy, w, h, frame);
            draw_rectangle_lines(sx, sy, w, h, edge, border);
            let cols = 4;
            let rows = 4;
            let pad = w.min(h) * 0.06;
            let gap = w.min(h) * 0.02;
            let cell_w = (w - pad * 2.0 - gap * (cols - 1) as f32) / cols as f32;
            let cell_h = (h - pad * 2.0 - gap * (rows - 1) as f32) / rows as f32;
            let cell = if lit {
                Color::from_rgba(55, 120, 200, 255)
            } else {
                Color::from_rgba(40, 70, 110, 255)
            };
            let gloss = Color::from_rgba(140, 200, 255, if lit { 90 } else { 40 });
            for row in 0..rows {
                for col in 0..cols {
                    let x = sx + pad + col as f32 * (cell_w + gap);
                    let y = sy + pad + row as f32 * (cell_h + gap);
                    draw_rectangle(x, y, cell_w, cell_h, cell);
                    if detail {
                        draw_line(
                            x + cell_w * 0.15,
                            y + cell_h * 0.2,
                            x + cell_w * 0.85,
                            y + cell_h * 0.75,
                            1.2,
                            gloss,
                        );
                    }
                    draw_rectangle_lines(x, y, cell_w, cell_h, 1.0, Color::from_rgba(20, 30, 45, 180));
                }
            }
        }
        BuildingKind::PowerPole => {
            // Central hub + cardinal arms (top-down junction).
            let hub_r = w.min(h) * 0.22;
            let metal = Color::from_rgba(70, 78, 90, 255);
            draw_circle(cx, cy, hub_r * 1.15, Color::from_rgba(0, 0, 0, 50));
            draw_circle(cx, cy, hub_r, metal);
            draw_circle(cx, cy, hub_r * 0.55, if lit { accent } else { Color::from_rgba(50, 58, 70, 255) });
            draw_circle_lines(cx, cy, hub_r, edge, border);
            let arm = hub_r * 0.9;
            let thick = hub_r * 0.35;
            for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
                let ax = cx + dx * (hub_r * 0.7);
                let ay = cy + dy * (hub_r * 0.7);
                if dx.abs() > 0.0 {
                    draw_rectangle(ax.min(ax + dx * arm), ay - thick * 0.5, arm, thick, metal);
                } else {
                    draw_rectangle(ax - thick * 0.5, ay.min(ay + dy * arm), thick, arm, metal);
                }
            }
            if lit && detail {
                // Soft power field rays.
                for i in 0..8 {
                    let a = i as f32 / 8.0 * std::f32::consts::TAU;
                    let r0 = hub_r * 1.3;
                    let r1 = hub_r * 2.6;
                    draw_line(
                        cx + a.cos() * r0,
                        cy + a.sin() * r0,
                        cx + a.cos() * r1,
                        cy + a.sin() * r1,
                        1.4,
                        with_alpha(accent, 0.35),
                    );
                }
            }
        }
        BuildingKind::OreNode => {
            // Square chassis + concentric bore blades (seen from above).
            let chassis = Color::from_rgba(55, 95, 140, 255);
            let dark = Color::from_rgba(30, 50, 75, 255);
            let blade = Color::from_rgba(90, 140, 185, 255);
            draw_rectangle(sx, sy, w, h, chassis);
            draw_rectangle(sx + w * 0.06, sy + h * 0.06, w * 0.88, h * 0.88, dark);
            draw_rectangle_lines(sx, sy, w, h, edge, border);
            let r = w.min(h) * 0.38;
            draw_circle(cx, cy, r * 1.05, Color::from_rgba(20, 30, 42, 255));
            // Spiral blade rings.
            for ring in 0..4 {
                let rr = r * (0.95 - ring as f32 * 0.18);
                draw_circle_lines(cx, cy, rr, edge * 1.1, blade);
                if detail {
                    for k in 0..6 {
                        let a0 = k as f32 / 6.0 * std::f32::consts::TAU + ring as f32 * 0.35;
                        let a1 = a0 + 0.55;
                        draw_line(
                            cx + a0.cos() * rr * 0.55,
                            cy + a0.sin() * rr * 0.55,
                            cx + a1.cos() * rr,
                            cy + a1.sin() * rr,
                            edge,
                            with_alpha(blade, 0.85),
                        );
                    }
                }
            }
            draw_circle(cx, cy, r * 0.16, if lit { accent } else { Color::from_rgba(40, 55, 70, 255) });
            // Facing notch.
            draw_circle(
                cx + fx * w * 0.42,
                cy + fy * h * 0.42,
                w.min(h) * 0.06,
                accent,
            );
        }
        BuildingKind::Smelter => {
            // Heavy chamfered block + molten core + pipe stubs.
            let body = Color::from_rgba(48, 52, 60, 255);
            let inset = w.min(h) * 0.12;
            let pts = [
                (sx + inset, sy),
                (sx + w - inset, sy),
                (sx + w, sy + inset),
                (sx + w, sy + h - inset),
                (sx + w - inset, sy + h),
                (sx + inset, sy + h),
                (sx, sy + h - inset),
                (sx, sy + inset),
            ];
            draw_poly_fill(&pts, body);
            draw_poly_outline(&pts, edge, border);
            let core_r = w.min(h) * 0.22;
            let heat = if lit {
                Color::from_rgba(255, 120, 40, 255)
            } else {
                Color::from_rgba(90, 45, 30, 255)
            };
            draw_circle(cx, cy, core_r * 1.35, Color::from_rgba(255, 80, 20, if lit { 60 } else { 20 }));
            draw_circle(cx, cy, core_r, heat);
            draw_circle(
                cx - core_r * 0.2,
                cy - core_r * 0.25,
                core_r * 0.35,
                Color::from_rgba(255, 220, 140, if lit { 180 } else { 60 }),
            );
            // Pipe stubs (top + side).
            let pipe = Color::from_rgba(90, 95, 105, 255);
            draw_rectangle(cx - w * 0.08, sy - h * 0.02, w * 0.16, h * 0.14, pipe);
            draw_rectangle(sx + w * 0.88, cy - h * 0.08, w * 0.14, h * 0.16, pipe);
            draw_rectangle_lines(cx - w * 0.08, sy - h * 0.02, w * 0.16, h * 0.14, 1.2, border);
            draw_rectangle_lines(sx + w * 0.88, cy - h * 0.08, w * 0.14, h * 0.16, 1.2, border);
        }
        BuildingKind::Assembler => {
            // Squared chassis with arm rails + status light.
            let body = Color::from_rgba(42, 58, 72, 255);
            draw_rectangle(sx, sy, w, h, body);
            draw_rectangle(sx + w * 0.1, sy + h * 0.12, w * 0.8, h * 0.76, Color::from_rgba(30, 42, 54, 255));
            draw_rectangle_lines(sx, sy, w, h, edge, border);
            let rail = Color::from_rgba(90, 130, 160, 255);
            draw_rectangle(sx + w * 0.18, sy + h * 0.22, w * 0.64, h * 0.08, rail);
            draw_rectangle(sx + w * 0.18, sy + h * 0.70, w * 0.64, h * 0.08, rail);
            let arm = if lit {
                Color::from_rgba(120, 200, 255, 255)
            } else {
                Color::from_rgba(70, 100, 120, 255)
            };
            draw_rectangle(cx - w * 0.04, sy + h * 0.28, w * 0.08, h * 0.44, arm);
            draw_circle(cx, cy, w.min(h) * 0.1, arm);
        }
        BuildingKind::Box => {
            // Reinforced crate with corner brackets + X braces.
            let crate_c = Color::from_rgba(55, 60, 70, 255);
            draw_rectangle(sx, sy, w, h, crate_c);
            draw_rectangle(sx + w * 0.08, sy + h * 0.08, w * 0.84, h * 0.84, Color::from_rgba(42, 46, 54, 255));
            draw_rectangle_lines(sx, sy, w, h, edge, border);
            let brace = Color::from_rgba(120, 130, 145, 200);
            draw_line(sx + w * 0.12, sy + h * 0.12, sx + w * 0.88, sy + h * 0.88, edge * 1.2, brace);
            draw_line(sx + w * 0.88, sy + h * 0.12, sx + w * 0.12, sy + h * 0.88, edge * 1.2, brace);
            let corner = w.min(h) * 0.14;
            for (ox, oy) in [
                (sx, sy),
                (sx + w - corner, sy),
                (sx, sy + h - corner),
                (sx + w - corner, sy + h - corner),
            ] {
                draw_rectangle_lines(ox, oy, corner, corner, edge, border);
            }
            if lit {
                draw_rectangle_lines(
                    sx + w * 0.2,
                    sy + h * 0.2,
                    w * 0.6,
                    h * 0.6,
                    1.2,
                    with_alpha(accent, 0.4),
                );
            }
        }
        BuildingKind::Splitter => {
            let along_w = w >= h;
            let (cells, cell_w, cell_h) = if along_w {
                (3, w / 3.0, h)
            } else {
                (3, w, h / 3.0)
            };
            for i in 0..cells {
                let (x0, y0) = if along_w {
                    (sx + cell_w * i as f32, sy)
                } else {
                    (sx, sy + cell_h * i as f32)
                };
                draw_rectangle(x0 + 1.0, y0 + 1.0, cell_w - 2.0, cell_h - 2.0, Color::from_rgba(40, 36, 28, 255));
                draw_rectangle_lines(x0 + 1.0, y0 + 1.0, cell_w - 2.0, cell_h - 2.0, edge, BELT_YELLOW);
                draw_chevron(
                    x0 + cell_w * 0.5,
                    y0 + cell_h * 0.5,
                    fx,
                    fy,
                    cell_w.min(cell_h) * 0.28,
                    BELT_YELLOW,
                );
            }
        }
        BuildingKind::Totem => {
            // Concentric top-down beacon (not a side-view spire).
            let r = w.min(h) * 0.42;
            draw_circle(cx, cy, r, Color::from_rgba(40, 48, 62, 255));
            draw_circle(cx, cy, r * 0.72, Color::from_rgba(55, 70, 95, 255));
            draw_circle_lines(cx, cy, r, edge, border);
            draw_circle(cx, cy, r * 0.28, if lit { accent } else { Color::from_rgba(60, 70, 85, 255) });
            if lit && detail {
                draw_circle(cx, cy, r * 0.95, with_alpha(accent, 0.12));
                draw_circle_lines(cx, cy, r * 1.15, 1.5, with_alpha(accent, 0.35));
            }
        }
        BuildingKind::Turret => {
            // Placeholder until sprites: turret_base (pad) + turret_gun (rotates with facing).
            // Heavy top-down cannon look — white armor, dark barrel, orange/teal accents.
            let (px, py) = (-fy, fx); // perpendicular
            let s = w.min(h);
            let armor = Color::from_rgba(230, 232, 236, 255);
            let armor_dim = Color::from_rgba(195, 198, 205, 255);
            let plate = Color::from_rgba(170, 174, 182, 255);
            let steel = Color::from_rgba(55, 58, 66, 255);
            let steel_hi = Color::from_rgba(90, 94, 104, 255);
            let outline = Color::from_rgba(28, 30, 36, 255);
            let orange = Color::from_rgba(240, 120, 40, if lit { 255 } else { 140 });
            let teal = Color::from_rgba(40, 190, 170, if lit { 230 } else { 110 });

            // --- turret_base: faceted armored octagon (stays put) ---
            let rx = w * 0.46;
            let ry = h * 0.46;
            let base = [
                (cx, cy - ry),
                (cx + rx * 0.55, cy - ry * 0.78),
                (cx + rx, cy - ry * 0.22),
                (cx + rx, cy + ry * 0.22),
                (cx + rx * 0.55, cy + ry * 0.78),
                (cx, cy + ry),
                (cx - rx * 0.55, cy + ry * 0.78),
                (cx - rx, cy + ry * 0.22),
                (cx - rx, cy - ry * 0.22),
                (cx - rx * 0.55, cy - ry * 0.78),
            ];
            draw_poly_fill(&base, armor);
            // Inset bevel ring.
            let inset = 0.14;
            let mut inner = base;
            for p in &mut inner {
                p.0 = cx + (p.0 - cx) * (1.0 - inset);
                p.1 = cy + (p.1 - cy) * (1.0 - inset);
            }
            draw_poly_fill(&inner, armor_dim);
            draw_poly_outline(&base, edge.max(1.8), outline);
            draw_poly_outline(&inner, 1.2, outline);
            // Center mount plate.
            draw_circle(cx, cy, s * 0.16, plate);
            draw_circle_lines(cx, cy, s * 0.16, 1.2, outline);
            if detail {
                for (ox, oy) in [
                    (0.0, -0.72),
                    (0.62, -0.42),
                    (0.62, 0.42),
                    (0.0, 0.72),
                    (-0.62, 0.42),
                    (-0.62, -0.42),
                ] {
                    draw_circle(cx + ox * rx, cy + oy * ry, s * 0.025, outline);
                }
            }

            // --- turret_gun: heavy cannon aimed by facing ---
            let gun_len = s * 0.48;
            let gun_half = s * 0.18;
            let shroud = s * 0.14;
            // Side armor shrouds (white).
            for side in [-1.0, 1.0] {
                let ox = px * side * (gun_half * 0.55);
                let oy = py * side * (gun_half * 0.55);
                let rear = -gun_len * 0.28;
                let tip = gun_len * 0.72;
                let pts = [
                    (cx + ox + fx * rear + px * side * shroud * 0.2, cy + oy + fy * rear + py * side * shroud * 0.2),
                    (cx + ox + fx * tip + px * side * shroud * 0.15, cy + oy + fy * tip + py * side * shroud * 0.15),
                    (cx + ox + fx * tip + px * side * shroud, cy + oy + fy * tip + py * side * shroud),
                    (cx + ox + fx * (rear * 0.2) + px * side * shroud * 1.05, cy + oy + fy * (rear * 0.2) + py * side * shroud * 1.05),
                ];
                draw_poly_fill(&pts, armor);
                draw_poly_outline(&pts, 1.3, outline);
            }
            // Thick dark cannon barrel (ribbed).
            let b_w = (s * 0.13).clamp(4.0, 12.0);
            let rear_x = cx - fx * gun_len * 0.22;
            let rear_y = cy - fy * gun_len * 0.22;
            let tip_x = cx + fx * gun_len * 0.78;
            let tip_y = cy + fy * gun_len * 0.78;
            draw_line(rear_x, rear_y, tip_x, tip_y, b_w, steel);
            if detail {
                for i in 1..5 {
                    let t = i as f32 / 5.0;
                    let bx = rear_x + (tip_x - rear_x) * t;
                    let by = rear_y + (tip_y - rear_y) * t;
                    draw_line(
                        bx - px * b_w * 0.45,
                        by - py * b_w * 0.45,
                        bx + px * b_w * 0.45,
                        by + py * b_w * 0.45,
                        1.4,
                        steel_hi,
                    );
                }
            }
            // Muzzle block.
            draw_rectangle(
                tip_x - b_w * 0.55 - fx * s * 0.02,
                tip_y - b_w * 0.55 - fy * s * 0.02,
                b_w * 1.1,
                b_w * 1.1,
                steel,
            );
            draw_rectangle_lines(
                tip_x - b_w * 0.55 - fx * s * 0.02,
                tip_y - b_w * 0.55 - fy * s * 0.02,
                b_w * 1.1,
                b_w * 1.1,
                1.2,
                outline,
            );
            // Pivot hub (rotation anchor) + red status dot.
            draw_circle(cx, cy, s * 0.11, steel);
            draw_circle_lines(cx, cy, s * 0.11, 1.4, outline);
            draw_circle(cx, cy, s * 0.035, Color::from_rgba(220, 50, 55, 255));
            if detail {
                // Orange lights on shrouds.
                for side in [-1.0, 1.0] {
                    let lx = cx + px * side * gun_half * 0.7 + fx * gun_len * 0.05;
                    let ly = cy + py * side * gun_half * 0.7 + fy * gun_len * 0.05;
                    draw_circle(lx, ly, s * 0.035, orange);
                    draw_circle(
                        lx + fx * s * 0.07,
                        ly + fy * s * 0.07,
                        s * 0.028,
                        orange,
                    );
                    // Teal indicator bars.
                    for k in 0..3 {
                        let t = -0.05 + k as f32 * 0.08;
                        let bx = cx + px * side * (gun_half * 0.95) + fx * gun_len * t;
                        let by = cy + py * side * (gun_half * 0.95) + fy * gun_len * t;
                        draw_rectangle(bx - 2.0, by - 1.2, 4.0, 2.4, teal);
                    }
                }
            }
        }
        BuildingKind::PowerWire | BuildingKind::Conveyor => {
            draw_rectangle(sx, sy, w, h, fill);
            draw_rectangle_lines(sx, sy, w, h, edge, border);
        }
        BuildingKind::SpawnAssault
        | BuildingKind::SpawnHunter
        | BuildingKind::SpawnSaboteur
        | BuildingKind::SpawnFogcaller
        | BuildingKind::SpawnNest => {
            draw_circle(cx, cy, w.min(h) * 0.4, fill);
            draw_circle_lines(cx, cy, w.min(h) * 0.4, edge, border);
        }
        BuildingKind::Machine | BuildingKind::Lab | BuildingKind::NexusSite => {
            draw_rectangle(sx, sy, w, h, fill);
            draw_rectangle_lines(sx, sy, w, h, edge, border);
            draw_rectangle(
                sx + w * 0.15,
                sy + h * 0.2,
                w * 0.7,
                h * 0.6,
                Color::from_rgba(30, 40, 55, 200),
            );
        }
        BuildingKind::FluidTank => {
            draw_rectangle(sx, sy, w, h, fill);
            draw_circle(cx, cy, w.min(h) * 0.38, Color::from_rgba(50, 100, 150, 220));
            draw_circle_lines(cx, cy, w.min(h) * 0.38, edge, border);
        }
        BuildingKind::Pipe => {
            draw_rectangle(sx + w * 0.3, sy, w * 0.4, h, fill);
            draw_rectangle(sx, sy + h * 0.3, w, h * 0.4, fill);
        }
        BuildingKind::Wall | BuildingKind::ReinforcedWall => {
            draw_rectangle(sx, sy, w, h, fill);
            draw_rectangle_lines(sx, sy, w, h, edge * 1.5, border);
        }
        BuildingKind::BallisticTurret | BuildingKind::LaserTurret => {
            draw_rectangle(sx, sy, w, h, fill);
            draw_circle(cx, cy, w.min(h) * 0.28, Color::from_rgba(40, 45, 55, 255));
            draw_circle_lines(cx, cy, w.min(h) * 0.28, edge, border);
        }
        BuildingKind::Nexus => {
            draw_rectangle(sx, sy, w, h, fill);
            draw_circle(cx, cy, w.min(h) * 0.35, Color::from_rgba(255, 200, 80, 180));
            draw_circle_lines(cx, cy, w.min(h) * 0.35, edge, Color::from_rgba(255, 230, 140, 255));
        }
    }
}

fn draw_cannon_fx(app: &App) {
    let Some(mat) = app.cannon_fx.as_ref() else {
        // CPU fallback if shader missing.
        for n in app.world.nodes.values() {
            if n.kind == BuildingKind::Turret {
                draw_turret_charge_fx_cpu(&app.cam, n);
            }
        }
        return;
    };

    let mut charges = [vec4(0.0, 0.0, 0.0, 0.0); CANNON_MAX_CHARGES];
    let mut ci = 0usize;
    for n in app.world.nodes.values() {
        if n.kind != BuildingKind::Turret || ci >= CANNON_MAX_CHARGES {
            continue;
        }
        let (cx, cy) = n.center();
        let (ux, uy) = (n.aim_angle.sin(), -n.aim_angle.cos());
        let mx = cx + ux * 36.0;
        let my = cy + uy * 36.0;
        let charge = if n.charge > 0.02 {
            n.charge
        } else if n.cooldown > 0.0 {
            (n.cooldown / TURRET_FIRE_INTERVAL).clamp(0.0, 1.0) * 0.22
        } else {
            0.0
        };
        if charge < 0.02 {
            continue;
        }
        let rad = 9.0 + charge * 11.0;
        charges[ci] = vec4(mx, my, rad, charge);
        ci += 1;
    }

    let mut beams = [vec4(0.0, 0.0, 0.0, 0.0); CANNON_MAX_BEAMS];
    let mut beam_life = [vec4(0.0, 0.0, 0.0, 0.0); CANNON_MAX_BEAMS];
    let mut bi = 0usize;
    for shot in &app.world.combat_shots {
        if bi >= CANNON_MAX_BEAMS {
            break;
        }
        let life = (shot.life / shot.max_life.max(0.05)).clamp(0.0, 1.0);
        beams[bi] = vec4(shot.x0, shot.y0, shot.x1, shot.y1);
        beam_life[bi] = vec4(life, shot.style as f32, 0.0, 0.0);
        bi += 1;
    }

    if ci == 0 && bi == 0 {
        return;
    }

    mat.set_uniform("ScreenSize", vec2(screen_width(), screen_height()));
    mat.set_uniform("CamPos", vec2(app.cam.x, app.cam.y));
    mat.set_uniform("CamZoom", app.cam.zoom);
    mat.set_uniform("Time", app.storm.time * 2.4);
    mat.set_uniform("PixelSize", FX_PIXEL);
    mat.set_uniform_array("Charges", &charges);
    mat.set_uniform_array("Beams", &beams);
    mat.set_uniform_array("BeamLife", &beam_life);
    gl_use_material(mat);
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), WHITE);
    gl_use_default_material();
}

fn turret_gun_angle(n: &Node) -> f32 {
    n.aim_angle
}

fn draw_turret_charge_fx_cpu(cam: &Cam, n: &Node) {
    if n.charge <= 0.02 && n.cooldown <= 0.0 {
        return;
    }
    let (cx, cy) = n.center();
    let (ux, uy) = (n.aim_angle.sin(), -n.aim_angle.cos());
    let mx = cx + ux * 36.0;
    let my = cy + uy * 36.0;
    let (sx, sy) = cam.world_to_screen(mx, my);
    let z = cam.zoom;
    let c = if n.charge > 0.02 {
        n.charge
    } else {
        (n.cooldown / TURRET_FIRE_INTERVAL).clamp(0.0, 1.0) * 0.3
    };
    let r = (4.0 + c * 10.0) * z;
    draw_fx_pixel_disc(
        cam,
        sx,
        sy,
        r * 1.5,
        Color::from_rgba(255, 110, 40, (28.0 + c * 40.0) as u8),
    );
    draw_fx_pixel_disc(
        cam,
        sx,
        sy,
        r,
        Color::from_rgba(255, 170, 80, (50.0 + c * 80.0) as u8),
    );
    draw_fx_pixel_disc(
        cam,
        sx,
        sy,
        r * 0.32,
        Color::from_rgba(255, 245, 210, (140.0 + c * 100.0) as u8),
    );
}

fn draw_node(
    cam: &Cam,
    n: &Node,
    hovered: bool,
    connect_tool: Option<BuildingKind>,
    wire_from: Option<(u32, usize)>,
    world: &World,
    node_id: u32,
    art: &art::Art,
) {
    let (sx, sy) = cam.world_to_screen(n.x, n.y);
    let w = n.w() * cam.zoom;
    let h = n.h() * cam.zoom;
    let lod = node_lod(cam.zoom);
    let accent = kind_swatch(n.kind);
    let damaged = n.hp < n.max_hp - 0.5;
    let unpowered = n.kind.needs_power() && !n.powered;
    let lit = !unpowered && (n.working || !n.kind.needs_power() || n.powered);

    let fill = if unpowered {
        Color::from_rgba(48, 30, 34, 220)
    } else if lit {
        Color::from_rgba(22, 32, 46, 230)
    } else {
        Color::from_rgba(26, 30, 38, 210)
    };
    let border = if unpowered {
        Color::from_rgba(220, 80, 80, 230)
    } else if damaged {
        Color::from_rgba(230, 170, 60, 230)
    } else if hovered {
        with_alpha(accent, 1.0)
    } else {
        with_alpha(accent, 0.9)
    };

    let detail = lod != NodeLod::Far;
    if n.kind == BuildingKind::Turret {
        let tint = if unpowered {
            Color::from_rgba(255, 140, 140, 230)
        } else if damaged {
            Color::from_rgba(255, 220, 170, 255)
        } else {
            WHITE
        };
        art::draw_turret(art, sx, sy, w, h, turret_gun_angle(n), tint);
        if hovered {
            draw_rectangle_lines(sx, sy, w, h, 1.5, border);
        }
    } else {
        draw_building_silhouette(n.kind, n.facing, sx, sy, w, h, fill, border, accent, lit, detail);
    }

    if lod == NodeLod::Near || hovered {
        let label = if hovered && lod == NodeLod::Near {
            n.kind.label()
        } else {
            n.kind.short()
        };
        let fs = (12.0 * cam.zoom).clamp(9.0, 15.0);
        // Label sits under the silhouette so it doesn't fight the shape.
        draw_text(
            label,
            sx + w * 0.5 - measure_text(label, None, fs as u16, 1.0).width * 0.5,
            sy + h + fs + 2.0,
            fs,
            with_alpha(TEXT, 0.85),
        );
    }

    if lod == NodeLod::Near && hovered {
        let body = match n.kind {
            BuildingKind::Solar => format!("{:+.0} e/s", SOLAR_POWER),
            BuildingKind::PowerPole => {
                if n.working {
                    "Field ON".into()
                } else {
                    "No network".into()
                }
            }
            BuildingKind::OreNode => {
                let res = n
                    .mine_vein
                    .and_then(|vid| world.veins.iter().find(|v| v.id == vid))
                    .map(|v| v.kind.label())
                    .or_else(|| n.mine_item.map(item_label))
                    .unwrap_or("no vent");
                let press = n
                    .mine_vein
                    .and_then(|vid| world.veins.iter().find(|v| v.id == vid))
                    .map(|v| {
                        format!(
                            "+{:.0}% ×{}",
                            v.yield_display(),
                            v.taps.max(1)
                        )
                    })
                    .unwrap_or_else(|| "—".into());
                format!(
                    "{}  {}  out {:.0}  {}",
                    res,
                    press,
                    n.out_ore,
                    if n.powered { "OK" } else { "OFF" }
                )
            }
            BuildingKind::Smelter | BuildingKind::Assembler => {
                let recipe = if let Some(r) = recipes::recipe_by_id(n.craft_recipe) {
                    let era = match r.era {
                        recipes::ScienceEra::Ember => "E0",
                        recipes::ScienceEra::Clearfoundry => "E1",
                    };
                    format!("{} · {era}", r.name)
                } else {
                    "idle".into()
                };
                let outs: f32 = Item::ALL
                    .iter()
                    .filter(|&&it| {
                        recipes::item_is_machine_output(
                            if n.kind == BuildingKind::Smelter {
                                recipes::MachineKind::Smelt
                            } else {
                                recipes::MachineKind::Assemble
                            },
                            it,
                        )
                    })
                    .map(|&it| n.stock(it))
                    .sum();
                let ins: f32 = n.stocks.iter().sum::<f32>() - outs;
                format!("{recipe}  in {:.0}  out {:.0}", ins.max(0.0), outs)
            }
            BuildingKind::Box => {
                let total: f32 = n.stocks.iter().sum();
                format!("stock {:.0}", total)
            }
            BuildingKind::Splitter => String::new(),
            BuildingKind::Totem => {
                if n.powered {
                    "Sheltering".into()
                } else {
                    "No power".into()
                }
            }
            BuildingKind::Turret => {
                if !n.powered {
                    "No power".into()
                } else if n.charge > 0.05 {
                    format!("Charging {:.0}%", n.charge * 100.0)
                } else if n.cooldown > 0.0 {
                    "Cooling".into()
                } else {
                    "Tracking".into()
                }
            }
            BuildingKind::PowerWire | BuildingKind::Conveyor => String::new(),
            BuildingKind::SpawnAssault
            | BuildingKind::SpawnHunter
            | BuildingKind::SpawnSaboteur
            | BuildingKind::SpawnFogcaller
            | BuildingKind::SpawnNest => "debug spawn".into(),
            BuildingKind::Machine | BuildingKind::Lab => {
                if n.era_craft {
                    content::content()
                        .recipe(n.craft_recipe)
                        .map(|r| r.name.as_str())
                        .unwrap_or("idle")
                        .to_string()
                } else {
                    format!("stock {:.0}", n.stocks.iter().sum::<f32>())
                }
            }
            BuildingKind::FluidTank | BuildingKind::Pipe => {
                let total: f32 = n.stocks.iter().sum();
                match n.fluid_filter {
                    Some(f) => format!("{} {:.0}", item_label(f), total),
                    None => format!("fluid {:.0}", total),
                }
            }
            BuildingKind::Wall | BuildingKind::ReinforcedWall => format!("hp {:.0}", n.hp),
            BuildingKind::BallisticTurret => format!("ammo {:.0}", n.ammo),
            BuildingKind::LaserTurret => {
                if n.powered {
                    "laser ready".into()
                } else {
                    "No power".into()
                }
            }
            BuildingKind::NexusSite => format!(
                "nexus {:.0}%",
                world.tech.nexus_progress * 100.0
            ),
            BuildingKind::Nexus => "ERA 1 COMPLETE".into(),
        };
        if !body.is_empty() {
            let fs = (11.0 * cam.zoom).clamp(9.0, 13.0);
            draw_text(
                &body,
                sx + w * 0.5 - measure_text(&body, None, fs as u16, 1.0).width * 0.5,
                sy + h + fs * 2.2,
                fs,
                TEXT_DIM,
            );
        }
    }

    if damaged {
        let bar_w = w * 0.7;
        let bar_h = (3.5 * cam.zoom).max(2.5);
        let bar_x = sx + (w - bar_w) * 0.5;
        let bar_y = sy + h + 2.0;
        let pct = (n.hp / n.max_hp).clamp(0.0, 1.0);
        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::from_rgba(20, 24, 30, 180));
        draw_rectangle(
            bar_x,
            bar_y,
            bar_w * pct,
            bar_h,
            if pct > 0.45 {
                Color::from_rgba(220, 170, 60, 230)
            } else {
                Color::from_rgba(220, 70, 70, 230)
            },
        );
    }

    let show_ports =
        connect_tool.is_some() || lod == NodeLod::Near || (lod == NodeLod::Mid && hovered);
    if show_ports {
        for (pi, p) in n.ports.iter().enumerate() {
            let relevant = match connect_tool {
                Some(BuildingKind::PowerWire) => p.kind.is_energy(),
                _ => true,
            };
            if connect_tool.is_some() && !relevant {
                continue;
            }
            let (px, py) = cam.world_to_screen(n.x + p.ox, n.y + p.oy);
            let r = (7.0 * cam.zoom).clamp(5.0, 11.0);
            let selected = wire_from == Some((node_id, pi));
            let valid_target = if let (Some(from), Some(BuildingKind::PowerWire)) =
                (wire_from, connect_tool)
            {
                from != (node_id, pi) && world.can_connect_power(from, (node_id, pi))
            } else {
                false
            };
            draw_port_icon(
                px,
                py,
                r,
                p,
                n,
                selected,
                valid_target,
                connect_tool.is_some(),
            );
        }
    }
}

/// Outward normal from building center through the port (for chevron direction).
fn port_outward(n: &Node, p: &Port) -> (f32, f32) {
    let dx = p.ox - n.w() * 0.5;
    let dy = p.oy - n.h() * 0.5;
    if dx.abs() >= dy.abs() {
        (if dx >= 0.0 { 1.0 } else { -1.0 }, 0.0)
    } else {
        (0.0, if dy >= 0.0 { 1.0 } else { -1.0 })
    }
}

fn draw_port_icon(
    px: f32,
    py: f32,
    r: f32,
    p: &Port,
    n: &Node,
    selected: bool,
    valid_target: bool,
    connect_mode: bool,
) {
    let (ox, oy) = port_outward(n, p);
    let energy = p.kind.is_energy();
    let base = if energy { POWER_C } else { CYAN };
    let color = if selected {
        Color::from_rgba(255, 255, 200, 255)
    } else if valid_target {
        Color::from_rgba(110, 255, 140, 255)
    } else {
        with_alpha(base, if connect_mode { 0.95 } else { 0.85 })
    };

    if p.kind.is_bidirectional() {
        // Diamond = bidirectional energy socket.
        let d = r * 1.05;
        let pts = [(px, py - d), (px + d, py), (px, py + d), (px - d, py)];
        draw_poly_fill(&pts, Color::from_rgba(20, 26, 34, 230));
        draw_poly_outline(&pts, 2.0, color);
        draw_circle(px, py, r * 0.28, color);
    } else if p.kind.is_output() {
        // Filled chevron pointing OUT of the building.
        draw_circle(px, py, r * 0.55, Color::from_rgba(20, 26, 34, 220));
        draw_chevron(px, py, ox, oy, r * 1.15, color);
        if connect_mode && !energy {
            draw_text("OUT", px - 10.0, py - r - 2.0, 10.0, with_alpha(color, 0.9));
        }
    } else if p.kind.is_input() {
        // Hollow socket = INPUT (items flow in).
        let s = r * 1.05;
        draw_rectangle(
            px - s,
            py - s,
            s * 2.0,
            s * 2.0,
            Color::from_rgba(20, 26, 34, 230),
        );
        draw_rectangle_lines(px - s, py - s, s * 2.0, s * 2.0, 2.2, color);
        draw_chevron(px, py, -ox, -oy, r * 0.7, with_alpha(color, 0.85));
        if connect_mode && !energy {
            draw_text("IN", px - 7.0, py - r - 2.0, 10.0, with_alpha(color, 0.9));
        }
    }

    if selected {
        draw_circle_lines(px, py, r * 1.7, 1.5, Color::from_rgba(255, 255, 180, 200));
    } else if valid_target {
        draw_circle_lines(px, py, r * 1.55, 1.4, Color::from_rgba(110, 255, 140, 180));
    }
}

fn draw_deposits(
    world: &World,
    cam: &Cam,
    storm: &Storm,
    art: &art::Art,
    show_fields: bool,
    zones: &[(f32, f32, f32)],
) {
    for v in &world.veins {
        if v.yield_pct <= 1.0 {
            continue;
        }
        // Hidden in the storm until a clear zone reveals the vein.
        if !storm.in_clear(v.x, v.y, zones) {
            continue;
        }
        if !cam.world_circle_visible(v.x, v.y, v.radius, 80.0) {
            continue;
        }
        if show_fields {
            draw_vein_field(cam, v);
        }
        draw_vein_crack(cam, art, v, show_fields);
    }
}

/// Organic cavern outline — only while the mining drill is selected.
fn draw_vein_field(cam: &Cam, v: &deposits::Vein) {
    let base = v.kind.color();
    let br = (base.r * 255.0) as u8;
    let bg = (base.g * 255.0) as u8;
    let bb = (base.b * 255.0) as u8;
    let outline = v.outline_world();
    let screen: Vec<(f32, f32)> = outline
        .iter()
        .map(|&(wx, wy)| cam.world_to_screen(wx, wy))
        .collect();
    if screen.len() < 3 {
        return;
    }
    draw_poly_fill(&screen, Color::from_rgba(br, bg, bb, 40));
    let edge = Color::from_rgba(br, bg, bb, 150);
    let thick = (1.6 * cam.zoom).clamp(1.2, 2.8);
    for i in 0..screen.len() {
        let a = screen[i];
        let b = screen[(i + 1) % screen.len()];
        draw_line(a.0, a.1, b.0, b.1, thick, edge);
    }
}

/// Push the player out of impassable gas vents — wall slide along crack silhouette.
fn resolve_player_crack_collision(
    player: &mut player::Player,
    world: &World,
    storm: &Storm,
    art: &art::Art,
    zones: &[(f32, f32, f32)],
) {
    let pr = player::BODY_R * 0.85;
    for _ in 0..2 {
        for v in &world.veins {
            if v.yield_pct <= 1.0 {
                continue;
            }
            if !storm.in_clear(v.x, v.y, zones) {
                continue;
            }
            let reach = art::crack_draw_world_size(v) * 0.75 + pr;
            let dx = player.x - v.x;
            let dy = player.y - v.y;
            if dx * dx + dy * dy > reach * reach {
                continue;
            }
            if !art::crack_blocks_point(art, v, player.x, player.y, pr) {
                continue;
            }
            let (nx, ny, ox, oy) =
                art::crack_resolve_wall(art, v, player.x, player.y, pr);
            player.x = nx;
            player.y = ny;
            // Slide along the wall: kill only the inward normal speed.
            if ox * ox + oy * oy > 1e-6 {
                let vn = player.vx * ox + player.vy * oy;
                if vn < 0.0 {
                    player.vx -= vn * ox;
                    player.vy -= vn * oy;
                }
            }
        }
    }
}

fn draw_vein_crack(cam: &Cam, art: &art::Art, v: &deposits::Vein, show_yield: bool) {
    // Black void crack only — gas plumes are a separate storm-quality pass.
    let z = cam.zoom;
    let (ocx, ocy) = cam.world_to_screen(v.x, v.y);

    let world_size = art::crack_draw_world_size(v);
    let size = world_size * z;
    let variant = art::crack_variant(v.seed);
    let rot = art::crack_rotation(v.seed);

    art::draw_crack(
        art,
        variant,
        ocx - size * 0.5,
        ocy - size * 0.5,
        size,
        rot,
        0.94,
    );

    if show_yield && z > 0.35 {
        let label = format!("{}  +{:.0}%", v.kind.short(), v.yield_display());
        let fs = (11.0 * z).clamp(9.0, 13.0);
        let tw = measure_text(&label, None, fs as u16, 1.0).width;
        draw_text(
            &label,
            ocx - tw * 0.5,
            ocy - (size * 0.42).clamp(22.0, 44.0),
            fs,
            Color::from_rgba(220, 228, 235, 210),
        );
    }
}

/// Rising pixelated gas from crack silhouette pixels (sparse, short rise).
fn draw_gas_vents(app: &App, zones: &[(f32, f32, f32)]) {
    if matches!(app.settings.effect_quality, EffectQuality::Low) {
        return;
    }
    let Some(mat) = app.gas_fx.as_ref() else {
        return;
    };
    let mut vents = [vec4(0.0, 0.0, 0.0, 0.0); GAS_MAX_VENTS];
    let mut colors = [vec4(0.0, 0.0, 0.0, 0.0); GAS_MAX_VENTS];
    let mut xforms = [vec4(0.0, 0.0, 0.0, 0.0); GAS_MAX_VENTS];
    let mut n = 0usize;
    for v in &app.world.veins {
        if n >= GAS_MAX_VENTS {
            break;
        }
        if v.yield_pct <= 1.0 {
            continue;
        }
        if !app.storm.in_clear(v.x, v.y, zones) {
            continue;
        }
        let size = art::crack_draw_world_size(v);
        if !app.cam.world_circle_visible(v.x, v.y, size * 0.65, 24.0) {
            continue;
        }
        let (gr, gg, gb, _) = v.kind.gas_rgba();
        let inten = (0.55 + v.freshness01() * 0.25).clamp(0.45, 0.85);
        let rot = art::crack_rotation(v.seed);
        let variant = art::crack_variant(v.seed) as f32;
        vents[n] = vec4(v.x, v.y, size, inten);
        colors[n] = vec4(gr as f32 / 255.0, gg as f32 / 255.0, gb as f32 / 255.0, 1.0);
        xforms[n] = vec4(rot.cos(), rot.sin(), variant, (v.seed % 997) as f32);
        n += 1;
    }
    if n == 0 {
        return;
    }

    mat.set_texture("Crack0", app.art.cracks[0].clone());
    mat.set_texture("Crack1", app.art.cracks[1].clone());
    mat.set_texture("Crack2", app.art.cracks[2].clone());
    mat.set_uniform("ScreenSize", vec2(screen_width(), screen_height()));
    mat.set_uniform("CamPos", vec2(app.cam.x, app.cam.y));
    mat.set_uniform("CamZoom", app.cam.zoom);
    mat.set_uniform("Time", app.storm.time * 1.35);
    mat.set_uniform("PixelSize", GAS_FX_PIXEL);
    mat.set_uniform_array("Vents", &vents);
    mat.set_uniform_array("VentColor", &colors);
    mat.set_uniform_array("VentXform", &xforms);
    gl_use_material(mat);
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), WHITE);
    gl_use_default_material();
}

fn draw_nests_and_raiders(world: &World, cam: &Cam, storm: &Storm, zones: &[(f32, f32, f32)]) {
    for nest in &world.nests {
        // Hidden in the storm until a clear zone reveals them.
        if !storm.in_clear(nest.x, nest.y, zones) {
            continue;
        }
        if !cam.world_circle_visible(nest.x, nest.y, NEST_RADIUS, 32.0) {
            continue;
        }
        let (sx, sy) = cam.world_to_screen(nest.x, nest.y);
        let r = NEST_RADIUS * cam.zoom;
        let fill = if nest.active {
            Color::from_rgba(160, 50, 70, 200)
        } else {
            Color::from_rgba(70, 55, 65, 160)
        };
        draw_circle(sx, sy, r, fill);
        // Wind-up ring: nest is about to launch a swarm.
        if nest.active && nest.wave_cd > 0.0 && nest.wave_cd <= NEST_REVEAL_WINDUP + 0.5 {
            let pulse = 1.0 - (nest.wave_cd / (NEST_REVEAL_WINDUP + 0.5)).clamp(0.0, 1.0);
            draw_circle_lines(
                sx,
                sy,
                r * (1.3 + pulse * 0.8),
                2.0,
                Color::from_rgba(255, 80, 80, (80.0 + pulse * 140.0) as u8),
            );
        }
        draw_circle_lines(
            sx,
            sy,
            r,
            1.5,
            if nest.active {
                Color::from_rgba(255, 90, 110, 220)
            } else {
                Color::from_rgba(120, 100, 110, 140)
            },
        );
        // Simple "nest" lobes.
        draw_circle(sx - r * 0.45, sy - r * 0.2, r * 0.4, fill);
        draw_circle(sx + r * 0.4, sy - r * 0.15, r * 0.35, fill);
        if nest.hp < nest.max_hp - 0.5 {
            let bar_w = r * 1.6;
            let pct = (nest.hp / nest.max_hp).clamp(0.0, 1.0);
            draw_rectangle(
                sx - bar_w * 0.5,
                sy + r + 4.0 * cam.zoom,
                bar_w,
                4.0 * cam.zoom.max(0.7),
                Color::from_rgba(30, 30, 36, 220),
            );
            draw_rectangle(
                sx - bar_w * 0.5,
                sy + r + 4.0 * cam.zoom,
                bar_w * pct,
                4.0 * cam.zoom.max(0.7),
                Color::from_rgba(220, 80, 90, 255),
            );
        }
        if nest.active && cam.zoom > 0.45 {
            draw_text(
                "NEST",
                sx - 16.0 * cam.zoom,
                sy - r - 6.0 * cam.zoom,
                (12.0 * cam.zoom).clamp(9.0, 14.0),
                Color::from_rgba(255, 140, 150, 230),
            );
        }
    }
    for raider in &world.raiders {
        let scale = if raider.role == RaiderRole::Fogcaller {
            1.25
        } else {
            1.0
        };
        let wr = RAIDER_RADIUS * scale;
        if !cam.world_circle_visible(raider.x, raider.y, wr, 24.0) {
            continue;
        }
        let (sx, sy) = cam.world_to_screen(raider.x, raider.y);
        let (fill, rim) = match raider.role {
            RaiderRole::Assault => (
                Color::from_rgba(210, 60, 80, 230),
                Color::from_rgba(255, 120, 130, 255),
            ),
            RaiderRole::Hunter => (
                Color::from_rgba(230, 120, 50, 230),
                Color::from_rgba(255, 180, 90, 255),
            ),
            RaiderRole::Saboteur => (
                Color::from_rgba(160, 80, 220, 230),
                Color::from_rgba(210, 150, 255, 255),
            ),
            RaiderRole::Fogcaller => (
                Color::from_rgba(70, 90, 120, 240),
                Color::from_rgba(140, 180, 220, 255),
            ),
        };
        let r = wr * cam.zoom;
        draw_ellipse(
            sx,
            sy + r * 0.55,
            r * 1.05,
            r * 0.4,
            0.0,
            Color::from_rgba(0, 0, 0, 50),
        );
        draw_circle(sx, sy, r * 1.15, with_alpha(rim, 0.18));
        draw_circle(sx, sy, r, fill);
        draw_circle(sx - r * 0.2, sy - r * 0.25, r * 0.45, with_alpha(rim, 0.35));
        draw_circle_lines(sx, sy, r, 2.0, with_alpha(rim, 0.45));
        draw_circle_lines(sx, sy, r, 1.1, rim);
        draw_circle(sx, sy - r * 0.55, r * 0.35, rim);
        draw_circle(sx - r * 0.08, sy - r * 0.62, r * 0.12, Color::from_rgba(255, 255, 255, 90));
    }
}

fn draw_storm_blots(world: &World, cam: &Cam) {
    for blot in &world.storm_blots {
        if !cam.world_circle_visible(blot.x, blot.y, blot.radius, 16.0) {
            continue;
        }
        let (sx, sy) = cam.world_to_screen(blot.x, blot.y);
        let r = blot.radius * cam.zoom;
        let a = (blot.life / FOG_BLOT_LIFE).clamp(0.15, 1.0);
        draw_circle(
            sx,
            sy,
            r,
            Color::from_rgba(40, 55, 80, (55.0 * a) as u8),
        );
        draw_circle_lines(
            sx,
            sy,
            r,
            2.0,
            Color::from_rgba(120, 160, 220, (160.0 * a) as u8),
        );
        draw_circle(
            sx,
            sy,
            r * 0.35,
            Color::from_rgba(90, 120, 180, (90.0 * a) as u8),
        );
    }
}

fn draw_combat_shots(world: &World, cam: &Cam, lasers_via_shader: bool) {
    for shot in &world.combat_shots {
        // Charge-cannon shader already draws style-1 lasers with the pixel mosaic.
        if shot.style == 1 && lasers_via_shader {
            continue;
        }
        let mid_x = (shot.x0 + shot.x1) * 0.5;
        let mid_y = (shot.y0 + shot.y1) * 0.5;
        let span = ((shot.x1 - shot.x0).hypot(shot.y1 - shot.y0) * 0.5).max(8.0);
        if !cam.world_circle_visible(mid_x, mid_y, span, 24.0) {
            continue;
        }
        let max_life = shot.max_life.max(0.05);
        let life01 = (shot.life / max_life).clamp(0.0, 1.0);
        let a = if life01 > 0.75 {
            1.0
        } else {
            (life01 / 0.75).clamp(0.0, 1.0)
        };
        let age = 1.0 - life01;
        let (sx0, sy0) = cam.world_to_screen(shot.x0, shot.y0);
        let (sx1, sy1) = cam.world_to_screen(shot.x1, shot.y1);
        let z = cam.zoom;
        let dx = sx1 - sx0;
        let dy = sy1 - sy0;

        if shot.style == 1 {
            // CPU fallback laser — same 8px mosaic as storm/cannon.
            let w = (2.4 * z).max(FX_PIXEL);
            draw_fx_pixel_line(
                cam,
                sx0,
                sy0,
                sx1,
                sy1,
                w * 3.0,
                Color::from_rgba(255, 90, 30, (40.0 * a) as u8),
            );
            draw_fx_pixel_line(
                cam,
                sx0,
                sy0,
                sx1,
                sy1,
                w * 1.5,
                Color::from_rgba(255, 170, 80, (140.0 * a) as u8),
            );
            draw_fx_pixel_line(
                cam,
                sx0,
                sy0,
                sx1,
                sy1,
                w * 0.6,
                Color::from_rgba(255, 250, 230, (230.0 * a) as u8),
            );
            let mf = (1.0 - (age / 0.28).min(1.0)).max(0.0);
            if mf > 0.01 {
                draw_fx_pixel_disc(
                    cam,
                    sx0,
                    sy0,
                    (7.0 * z) * mf,
                    Color::from_rgba(255, 140, 50, (120.0 * mf * a) as u8),
                );
            }
            let travel = (age * 2.4).min(1.0);
            draw_fx_pixel_disc(
                cam,
                sx0 + dx * travel,
                sy0 + dy * travel,
                (4.0 * z) * a,
                Color::from_rgba(255, 250, 230, (200.0 * a) as u8),
            );
            if travel >= 0.92 {
                let impact = ((age - 0.35) / 0.65).clamp(0.0, 1.0);
                let fade = (1.0 - impact) * a;
                draw_fx_pixel_disc(
                    cam,
                    sx1,
                    sy1,
                    (5.0 + impact * 16.0) * z,
                    Color::from_rgba(255, 100, 35, (70.0 * fade) as u8),
                );
            }
        } else {
            let w = (1.8 * z).max(FX_PIXEL);
            draw_fx_pixel_line(
                cam,
                sx0,
                sy0,
                sx1,
                sy1,
                w * 2.0,
                Color::from_rgba(255, 180, 80, (50.0 * a) as u8),
            );
            draw_fx_pixel_line(
                cam,
                sx0,
                sy0,
                sx1,
                sy1,
                w,
                Color::from_rgba(255, 240, 200, (200.0 * a) as u8),
            );
        }
    }
}

fn draw_placement_ghost(
    world: &World,
    ui: &Ui,
    cam: &Cam,
    storm: &Storm,
    inv: &Inventory,
    art: &art::Art,
    wx: f32,
    wy: f32,
    creative: bool,
) {
    if ui.build_open || ui.drag_node.is_some() {
        return;
    }
    let Some(entry) = ui.current_entry() else {
        return;
    };
    let kind = entry.kind();
    // Wire / conveyor tools: cursor hint instead of a building ghost.
    if kind.is_cable() {
        if ui.wire_paint.is_some() {
            return; // rubber-band drawn by link renderers
        }
        let (sx, sy) = cam.world_to_screen(wx, wy);
        draw_text(
            "Wire — click ◆ · click corners · click ◆ · RMB undo",
            sx + 14.0,
            sy - 8.0,
            16.0,
            POWER_C,
        );
        return;
    }
    if kind.is_belt_tool() {
        let (sx, sy) = cam.world_to_screen(wx, wy);
        let costs = belt_recipe();
        let afford = creative || inv.can_afford(costs);
        let hint = if afford {
            if creative {
                "Belt — drag to paint · R rotate · RMB erase · free".to_string()
            } else {
                "Belt — drag to paint · R rotate · RMB erase".to_string()
            }
        } else {
            inv.missing_hint(costs)
        };
        draw_text(
            &hint,
            sx + 14.0,
            sy - 8.0,
            16.0,
            if afford {
                BELT_YELLOW
            } else {
                Color::from_rgba(255, 140, 140, 255)
            },
        );
        return;
    }
    if kind.is_debug_tool() {
        let (sx, sy) = cam.world_to_screen(wx, wy);
        let col = entry.swatch();
        let r = if kind == BuildingKind::SpawnNest {
            NEST_RADIUS * cam.zoom
        } else {
            RAIDER_RADIUS * cam.zoom * 1.2
        };
        draw_circle(sx, sy, r * 1.3, with_alpha(col, 0.2));
        draw_circle(sx, sy, r, with_alpha(col, 0.55));
        draw_circle_lines(sx, sy, r, 2.0, col);
        let label = format!("{} — click to spawn (free)", entry.label());
        draw_text(&label, sx + r + 8.0, sy - 4.0, 15.0, col);
        return;
    }
    if ui.wire_paint.is_some() {
        return;
    }
    let mut probe = Node::new(kind, 0.0, 0.0, ui.place_facing);
    if let Some(mid) = entry.machine_id() {
        probe.set_machine_id(Some(mid));
    }
    let (x, y) = snap_building_xy_size(probe.footprint(), ui.place_facing, wx, wy);
    let costs = building_recipe(kind);
    let can_afford = creative || inv.can_afford(costs);
    let unlock = entry.tech_unlock();
    let locked = !creative && !world.tech.machine_unlocked(&unlock);
    let needs_deposit = kind == BuildingKind::OreNode && {
        let zones = storm.clear_zones(world);
        !world.veins.iter().any(|v| {
            v.yield_pct > 1.0
                && v.overlaps_rect(x, y, probe.w(), probe.h())
                && storm.in_clear(v.x, v.y, &zones)
        })
    };
    let blocked =
        world.collides(x, y, probe.w(), probe.h(), None) || !can_afford || needs_deposit || locked;
    let in_storm = storm.point_in_storm(x + probe.w() * 0.5, y + probe.h() * 0.5, world);
    let (sx, sy) = cam.world_to_screen(x, y);
    let w = probe.w() * cam.zoom;
    let h = probe.h() * cam.zoom;
    let fill = if blocked {
        Color::from_rgba(180, 50, 50, 90)
    } else if in_storm {
        Color::from_rgba(255, 160, 60, 70)
    } else {
        Color::from_rgba(30, 50, 60, 100)
    };
    let outline = if blocked {
        Color::from_rgba(220, 80, 80, 200)
    } else if in_storm {
        Color::from_rgba(255, 170, 70, 220)
    } else {
        CYAN
    };
    let accent = entry.swatch();
    if kind == BuildingKind::Turret {
        let tint = if blocked {
            Color::from_rgba(255, 120, 120, 180)
        } else if in_storm {
            Color::from_rgba(255, 200, 120, 200)
        } else {
            Color::from_rgba(255, 255, 255, 200)
        };
        art::draw_turret(
            art,
            sx,
            sy,
            w,
            h,
            facing_aim_angle(ui.place_facing),
            tint,
        );
        draw_rectangle_lines(sx, sy, w, h, 1.5, outline);
    } else {
        draw_building_silhouette(
            kind,
            ui.place_facing,
            sx,
            sy,
            w,
            h,
            fill,
            outline,
            accent,
            !blocked,
            true,
        );
    }
    let label = if locked {
        format!("Locked — {}", unlock)
    } else if !can_afford {
        inv.missing_hint(costs)
    } else if needs_deposit {
        "Place on a revealed gas vent".into()
    } else if world.collides(x, y, probe.w(), probe.h(), None) {
        entry.label()
    } else if in_storm {
        format!("{} · storm!", entry.short())
    } else {
        entry.label()
    };
    draw_text(
        &label,
        sx + w * 0.5 - measure_text(&label, None, ((14.0 * cam.zoom).clamp(10.0, 16.0)) as u16, 1.0).width * 0.5,
        sy + h + 16.0,
        (14.0 * cam.zoom).clamp(10.0, 16.0),
        if blocked {
            Color::from_rgba(255, 160, 160, 255)
        } else if in_storm {
            ACCENT
        } else {
            CYAN
        },
    );
    if kind == BuildingKind::PowerPole {
        let (cx, cy) = cam.world_to_screen(x + probe.w() * 0.5, y + probe.h() * 0.5);
        draw_circle_lines(cx, cy, POLE_RADIUS * cam.zoom, 1.0, POWER_DIM);
    }
    if kind == BuildingKind::Totem {
        let (cx, cy) = cam.world_to_screen(x + probe.w() * 0.5, y + probe.h() * 0.5);
        draw_circle_lines(
            cx,
            cy,
            TOTEM_CLEAR_RADIUS * STORM_HARD_CLEAR_SCALE * cam.zoom,
            1.2,
            Color::from_rgba(140, 100, 220, 120),
        );
    }
    if kind == BuildingKind::Turret {
        let (cx, cy) = cam.world_to_screen(x + probe.w() * 0.5, y + probe.h() * 0.5);
        draw_circle_lines(cx, cy, TURRET_RANGE * cam.zoom, 1.0, Color::from_rgba(200, 90, 90, 100));
    }
}

fn draw_peer_ghost(cam: &Cam, kind: BuildingKind, wx: f32, wy: f32, facing: Facing, color: Color) {
    let probe = Node::new(kind, 0.0, 0.0, facing);
    let x = wx - probe.w() * 0.5;
    let y = wy - probe.h() * 0.5;
    let (sx, sy) = cam.world_to_screen(x, y);
    let w = probe.w() * cam.zoom;
    let h = probe.h() * cam.zoom;
    let mut fill = color;
    fill.a = 0.2;
    let mut outline = color;
    outline.a = 0.7;
    draw_rectangle(sx, sy, w, h, fill);
    draw_rectangle_lines(sx, sy, w, h, 1.5, outline);
}

fn draw_peer_cursors(app: &App) {
    for peer in app.peers.values() {
        let color = peer_color(peer.id);
        // Remote drone (other players only — never the local one).
        player::draw_drone_remote(
            &peer.drone,
            app.cam.x,
            app.cam.y,
            app.cam.zoom,
            color,
            Some(&peer_label(peer.id)),
        );
        // Mouse cursor marker matching drone color.
        let (sx, sy) = app.cam.world_to_screen(peer.x, peer.y);
        let size = 12.0;
        draw_triangle(
            Vec2::new(sx, sy),
            Vec2::new(sx + size, sy + size * 0.7),
            Vec2::new(sx + size * 0.25, sy + size),
            color,
        );
        if let Some(kind) = peer.selected {
            draw_peer_ghost(&app.cam, kind, peer.x, peer.y, peer.facing, color);
        }
    }
}

fn draw_hotbar(ui: &Ui, mouse: (f32, f32)) {
    let (bar_x, bar_y, slot, gap) = hotbar_geom();
    let width = HOTBAR_SLOTS as f32 * slot + (HOTBAR_SLOTS - 1) as f32 * gap;
    let pad = s(10.0);

    ui_chrome::floating_bar(
        bar_x - pad,
        bar_y - pad,
        width + pad * 2.0,
        slot + pad * 2.0,
    );

    for i in 0..HOTBAR_SLOTS {
        let x = bar_x + i as f32 * (slot + gap);
        let selected = i == ui.hotbar_index
            && ui.selected.is_some()
            && ui.hotbar_entry(i) == ui.current_entry();
        let indexed = i == ui.hotbar_index;
        let hovered = mouse.0 >= x
            && mouse.0 <= x + slot
            && mouse.1 >= bar_y
            && mouse.1 <= bar_y + slot;
        let drop_target = ui.palette_drag.is_some() && hovered;

        ui_chrome::slot_frame(x, bar_y, slot, hovered || indexed, selected, drop_target);

        draw_text(
            &(i + 1).to_string(),
            x + s(5.0),
            bar_y + s(14.0),
            s(12.0),
            UI_TEXT_DIM,
        );

        if let Some(entry) = ui.hotbar_entry(i) {
            let dim = ui.hotbar_drag_from == Some(i);
            let mut swatch = entry.swatch();
            if dim {
                swatch.a = 0.35;
            }
            let chip_w = slot - s(18.0);
            let chip_h = s(10.0);
            draw_rectangle(
                x + (slot - chip_w) * 0.5,
                bar_y + s(20.0),
                chip_w,
                chip_h,
                swatch,
            );
            let fs = s(13.0);
            let label = entry.short();
            let tw = measure_text(&label, None, fs as u16, 1.0).width;
            draw_text(
                &label,
                x + (slot - tw) * 0.5,
                bar_y + slot - s(8.0),
                fs,
                if dim { TEXT_DIM } else { TEXT },
            );
        }
    }
}

fn draw_controls_chip(app: &App) {
    let label = format!(
        "{} · {} · WASD · Tab inv · B build · Q cancel · RMB remove",
        app.game_mode.label(),
        app.player.cam_mode.label()
    );
    ui_chrome::chip(&label, s(16.0), screen_height() - s(78.0));
}

fn inventory_panel_rect() -> Rect {
    let slot = s(52.0);
    let gap = s(6.0);
    let pad = s(18.0);
    let w = pad * 2.0 + INV_COLS as f32 * slot + (INV_COLS - 1) as f32 * gap;
    let h = s(78.0) + inventory::INV_ROWS as f32 * slot + (inventory::INV_ROWS - 1) as f32 * gap + pad;
    Rect {
        x: (screen_width() - w) * 0.5,
        y: (screen_height() - h) * 0.5 - s(30.0),
        w,
        h,
    }
}

fn draw_item_icon(art: &art::Art, item: Item, cx: f32, cy: f32, size: f32) {
    if item == Item::CrudeOil {
        let (fill, dark, _) = item_chip_colors(item);
        draw_ellipse(cx, cy + size * 0.05, size * 0.34, size * 0.26, 0.0, dark);
        draw_ellipse(cx - size * 0.04, cy - size * 0.04, size * 0.26, size * 0.18, 0.0, fill);
        return;
    }
    if item_is_ore(item) {
        art::draw_tinted_item(&art.ore, cx, cy, size * 0.72, item_tint(item));
        return;
    }
    if item_is_ingot(item) {
        art::draw_tinted_item(&art.ingot, cx, cy, size * 0.72, item_tint(item));
        return;
    }
    let (fill, dark, vein) = item_chip_colors(item);
    draw_circle(cx, cy, size * 0.38, fill);
    draw_circle(cx - size * 0.12, cy - size * 0.1, size * 0.16, dark);
    draw_circle(cx + size * 0.14, cy + size * 0.08, size * 0.14, vein);
}

fn draw_and_handle_inventory(app: &mut App, mouse: (f32, f32)) {
    ui_chrome::scrim(150);
    let r = inventory_panel_rect();
    ui_chrome::panel(r.x, r.y, r.w, r.h);
    ui_chrome::panel_header(
        r.x,
        r.y + s(6.0),
        r.w,
        "Inventory",
        Some("Tab / E / Esc close"),
    );
    draw_text(
        &format!("Gas {}   ·   Ingot {}", app.inventory.ore(), app.inventory.ingot()),
        r.x + s(18.0),
        r.y + s(52.0),
        s(14.0),
        UI_AMBER,
    );

    let slot = s(52.0);
    let gap = s(6.0);
    let pad = s(18.0);
    let origin_y = r.y + s(68.0);
    let mut hover_label: Option<String> = None;

    for i in 0..INV_SLOTS {
        let col = i % INV_COLS;
        let row = i / INV_COLS;
        let x = r.x + pad + col as f32 * (slot + gap);
        let y = origin_y + row as f32 * (slot + gap);
        let hovered = mouse.0 >= x
            && mouse.0 <= x + slot
            && mouse.1 >= y
            && mouse.1 <= y + slot;
        ui_chrome::slot_frame(x, y, slot, hovered, false, false);
        let cell = app.inventory.slots[i];
        if let Some(item) = cell.item {
            draw_item_icon(&app.art, item, x + slot * 0.5, y + slot * 0.42, slot);
            let count = cell.count.to_string();
            let fs = s(13.0);
            let tw = measure_text(&count, None, fs as u16, 1.0).width;
            draw_text(
                &count,
                x + slot - tw - s(4.0),
                y + slot - s(6.0),
                fs,
                UI_TEXT,
            );
            if hovered {
                hover_label = Some(format!("{} × {}", item_label(item), cell.count));
            }
        }
    }

    if let Some(label) = hover_label {
        ui_chrome::tooltip(&label, mouse.0, mouse.1);
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        let inside = mouse.0 >= r.x
            && mouse.0 <= r.x + r.w
            && mouse.1 >= r.y
            && mouse.1 <= r.y + r.h;
        if !inside {
            app.ui.close_inventory();
        }
    }
}

fn draw_tool_dock(app: &App, mouse: (f32, f32)) {
    let top = tool_button_rect(0);
    let bot = tool_button_rect(3);
    let rail_pad = s(10.0);
    ui_chrome::floating_bar(
        top.x - rail_pad,
        top.y - rail_pad,
        top.w + rail_pad * 2.0,
        (bot.y + bot.h) - top.y + rail_pad * 2.0,
    );

    for (i, tool) in CornerTool::ALL.iter().enumerate() {
        let r = tool_button_rect(i);
        let active = match *tool {
            CornerTool::Build => app.ui.build_open,
            other => app.ui.overlay == Some(other),
        };
        let hovered = mouse.0 >= r.x
            && mouse.0 <= r.x + r.w
            && mouse.1 >= r.y
            && mouse.1 <= r.y + r.h;

        let cx = r.x + r.w * 0.5;
        let cy = r.y + r.h * 0.5;
        let radius = r.w * 0.46;

        draw_circle(
            cx,
            cy,
            radius,
            if active {
                Color::from_rgba(28, 62, 56, 245)
            } else if hovered {
                Color::from_rgba(36, 46, 60, 245)
            } else {
                Color::from_rgba(20, 26, 34, 230)
            },
        );
        draw_circle_lines(
            cx,
            cy,
            radius,
            if active || hovered { 2.2 } else { 1.2 },
            if active {
                UI_CYAN
            } else if hovered {
                UI_AMBER
            } else {
                UI_EDGE
            },
        );

        let accent = if active || hovered { UI_CYAN } else { UI_TEXT };
        let icon_size = radius * 1.15;
        match *tool {
            CornerTool::Build => {
                if let Some(tex) = app.art.icon_hammer.as_ref() {
                    art::draw_dock_icon(tex, cx, cy, icon_size, WHITE);
                } else {
                    draw_build_icon(cx, cy, accent);
                }
            }
            CornerTool::Recipes => draw_recipe_icon(cx, cy, accent),
            CornerTool::TechTree => {
                if let Some(tex) = app.art.icon_tech.as_ref() {
                    art::draw_dock_icon(tex, cx, cy, icon_size, WHITE);
                } else {
                    draw_tech_icon(cx, cy, accent);
                }
            }
            CornerTool::Map => {
                if let Some(tex) = app.art.icon_map.as_ref() {
                    art::draw_dock_icon(tex, cx, cy, icon_size, WHITE);
                } else {
                    draw_map_icon(cx, cy, accent);
                }
            }
            CornerTool::NodeChart => draw_nodes_icon(cx, cy, accent),
        }

        if hovered || active {
            let label = tool.label();
            let fs = s(14.0);
            let tw = measure_text(label, None, fs as u16, 1.0).width;
            let lx = r.x - s(14.0) - tw;
            let ly = cy + fs * 0.35;
            draw_rectangle(
                lx - s(10.0),
                cy - s(13.0),
                tw + s(18.0),
                s(26.0),
                Color::from_rgba(12, 16, 22, 220),
            );
            draw_rectangle(lx - s(10.0), cy - s(13.0), 3.0, s(26.0), UI_CYAN);
            draw_text(label, lx, ly, fs, if active { UI_CYAN } else { UI_TEXT });
        }
    }
}

fn draw_corner_overlay(tool: CornerTool, world: &World, mouse: (f32, f32), ui: &mut Ui) {
    ui_chrome::scrim(150);
    let (w, h, x, y) = if tool == CornerTool::Recipes {
        // Nearly full-viewport — recipe progression needs room.
        let w = (screen_width() - s(48.0)).clamp(s(900.0), screen_width() - s(24.0));
        let h = (screen_height() - s(56.0)).clamp(s(560.0), screen_height() - s(28.0));
        let x = (screen_width() - w) * 0.5;
        let y = (screen_height() - h) * 0.5;
        (w, h, x, y)
    } else {
        let w = s(720.0);
        let h = s(480.0);
        let x = (screen_width() - w) * 0.5;
        let y = (screen_height() - h) * 0.5 - s(40.0);
        (w, h, x, y)
    };
    ui_chrome::panel(x, y, w, h);

    let title = match tool {
        CornerTool::Recipes => "Production Tree",
        CornerTool::TechTree => "Tech / Route",
        CornerTool::Map => "Map",
        CornerTool::NodeChart => "Factory Graph",
        CornerTool::Build => "Build",
    };
    let blurb = match tool {
        CornerTool::Recipes => {
            "Helmod-style nested chain — every ingredient expands to its producer · Tab web · search product"
        }
        CornerTool::TechTree => "Era 1 technology routes — prereqs, science cost, unlocks.",
        CornerTool::Map => "Storm pocket, nests, and expansion frontiers.",
        CornerTool::NodeChart => "Power nets, belt throughput, and machine status.",
        CornerTool::Build => "",
    };
    let tip = match tool {
        CornerTool::Recipes => {
            let s = &content::content().stats;
            if ui.recipe_view_web {
                format!(
                    "{} recipes · web zoom {:.0}% · Tab tree · F fit",
                    s.recipes,
                    ui.recipe_zoom * 100.0
                )
            } else {
                format!(
                    "{} recipes · nested production · Tab web · click row to refocus",
                    s.recipes
                )
            }
        }
        CornerTool::TechTree => {
            let s = &content::content().stats;
            format!(
                "Pack: {} items · {} fluids · {} recipes · {} machines · {} techs · researched {}",
                s.items,
                s.fluids,
                s.recipes,
                s.machines,
                s.technologies,
                world.tech.researched.len()
            )
        }
        CornerTool::Map => "Planned: resource patches · remote view · ping markers".to_string(),
        CornerTool::NodeChart => format!(
            "Now: {} buildings · {} belts · {} power links",
            world.nodes.len(),
            world.belt_tiles.len(),
            world.links.len()
        ),
        CornerTool::Build => String::new(),
    };
    ui_chrome::panel_header(x, y + s(8.0), w, title, None);
    draw_text(blurb, x + s(24.0), y + s(64.0), s(15.0), UI_TEXT_DIM);

    match tool {
        CornerTool::Recipes => {
            let graph_y = y + s(78.0);
            let graph_h = h - s(120.0);
            let gx = x + s(16.0);
            let gw = w - s(32.0);
            let over = mouse.0 >= gx
                && mouse.0 <= gx + gw
                && mouse.1 >= graph_y
                && mouse.1 <= graph_y + graph_h;
            if is_key_pressed(KeyCode::Tab) {
                ui.recipe_view_web = !ui.recipe_view_web;
                ui.recipe_fit_pending = true;
            }
            if ui.recipe_view_web {
                if over {
                    if is_key_pressed(KeyCode::F) {
                        ui.recipe_fit_pending = true;
                    }
                    let (wheel_x, wheel_y) = mouse_wheel();
                    let wheel = if wheel_y != 0.0 { wheel_y } else { wheel_x };
                    if wheel != 0.0 {
                        let before = screen_to_recipe_graph(ui, gx, graph_y, gw, graph_h, mouse);
                        let zoom = (ui.recipe_zoom * (1.0 + wheel * 0.12)).clamp(0.08, 6.0);
                        ui.recipe_zoom = zoom;
                        let after = screen_to_recipe_graph(ui, gx, graph_y, gw, graph_h, mouse);
                        ui.recipe_cam_x += before.0 - after.0;
                        ui.recipe_cam_y += before.1 - after.1;
                    }
                    let pan_btn = is_mouse_button_down(MouseButton::Middle)
                        || is_mouse_button_down(MouseButton::Right);
                    if pan_btn {
                        if !ui.recipe_panning {
                            ui.recipe_panning = true;
                            ui.recipe_pan_last = mouse;
                        } else {
                            let dx = mouse.0 - ui.recipe_pan_last.0;
                            let dy = mouse.1 - ui.recipe_pan_last.1;
                            ui.recipe_cam_x -= dx / ui.recipe_zoom;
                            ui.recipe_cam_y -= dy / ui.recipe_zoom;
                            ui.recipe_pan_last = mouse;
                        }
                    } else {
                        ui.recipe_panning = false;
                    }
                } else {
                    ui.recipe_panning = false;
                }
                draw_full_recipe_tree(world, ui, mouse, gx, graph_y, gw, graph_h);
            } else {
                draw_helmod_production_tree(world, ui, mouse, gx, graph_y, gw, graph_h);
            }
            draw_text(&tip, x + s(24.0), y + h - s(28.0), s(14.0), UI_AMBER);
        }
        CornerTool::TechTree => {
            draw_tech_route(world, x + s(16.0), y + s(88.0), w - s(32.0), h - s(140.0));
            draw_text(&tip, x + s(24.0), y + h - s(36.0), s(15.0), UI_AMBER);
        }
        CornerTool::Map => {
            draw_map_icon(x + w * 0.5, y + h * 0.52, UI_CYAN);
            let cx = x + w * 0.5;
            let cy = y + h * 0.52;
            draw_circle_lines(cx, cy, s(70.0), 1.5, Color::from_rgba(120, 160, 200, 100));
            draw_circle(cx, cy, s(6.0), UI_CYAN);
            draw_text(&tip, x + s(24.0), y + h - s(36.0), s(15.0), UI_AMBER);
        }
        CornerTool::NodeChart => {
            draw_nodes_icon(x + w * 0.5, y + h * 0.48, UI_CYAN);
            draw_text(&tip, x + s(24.0), y + h - s(36.0), s(15.0), UI_AMBER);
        }
        CornerTool::Build => {}
    }
}

fn screen_to_recipe_graph(
    ui: &Ui,
    vx: f32,
    vy: f32,
    vw: f32,
    vh: f32,
    mouse: (f32, f32),
) -> (f32, f32) {
    let lx = mouse.0 - (vx + vw * 0.5);
    let ly = mouse.1 - (vy + vh * 0.5);
    (
        ui.recipe_cam_x + lx / ui.recipe_zoom,
        ui.recipe_cam_y + ly / ui.recipe_zoom,
    )
}

fn recipe_graph_to_screen(
    ui: &Ui,
    vx: f32,
    vy: f32,
    vw: f32,
    vh: f32,
    gx: f32,
    gy: f32,
) -> (f32, f32) {
    (
        vx + vw * 0.5 + (gx - ui.recipe_cam_x) * ui.recipe_zoom,
        vy + vh * 0.5 + (gy - ui.recipe_cam_y) * ui.recipe_zoom,
    )
}

fn recipe_node_color(category: &str, unlocked: bool) -> Color {
    let a = if unlocked { 230 } else { 120 };
    let (r, g, b) = match category {
        "mining" | "extraction" | "atmosphere" => (140, 150, 160),
        "crushing" | "drying" | "purification" => (180, 140, 90),
        "smelting" | "metallurgy" => (210, 120, 70),
        "assembly" | "manufacturing" => (90, 160, 200),
        "water" | "water_purification" | "chemical" => (70, 150, 220),
        "military" | "ammunition" | "defense" => (200, 80, 90),
        "research" => (140, 100, 220),
        "recovery" => (120, 110, 90),
        _ => (100, 170, 160),
    };
    Color::from_rgba(r, g, b, a)
}

/// One continuous DAG of every Era 1 recipe — no family filters.
fn item_chip_color(item_id: &str, waste: bool) -> Color {
    if waste || item_id.contains("_waste_") {
        return Color::from_rgba(160, 55, 55, 230);
    }
    if item_id.starts_with("era1_fluid_") || item_id.starts_with("era1_gas_") {
        return Color::from_rgba(45, 110, 150, 230);
    }
    if item_id.starts_with("era1_raw_") {
        return Color::from_rgba(90, 120, 70, 230);
    }
    if item_id.contains("circuit") || item_id.contains("wire") || item_id.contains("electronics") {
        return Color::from_rgba(50, 140, 70, 230);
    }
    Color::from_rgba(55, 70, 90, 230)
}

fn draw_recipe_chip(x: f32, y: f32, w: f32, h: f32, label: &str, fill: Color) {
    draw_rectangle(x, y, w, h, fill);
    draw_rectangle_lines(x, y, w, h, 1.0, Color::from_rgba(20, 24, 30, 200));
    let fs = (h * 0.55).clamp(9.0, 13.0);
    let tw = measure_text(label, None, fs as u16, 1.0).width;
    draw_text(
        label,
        x + (w - tw) * 0.5,
        y + h * 0.72,
        fs,
        Color::from_rgba(235, 240, 245, 255),
    );
}

fn short_item_label(name: &str) -> String {
    if name.len() <= 10 {
        name.to_string()
    } else {
        format!("{}…", &name[..9])
    }
}

fn handle_recipe_search_chars(ui: &mut Ui) {
    if !ui.recipe_search_focus {
        return;
    }
    if is_key_pressed(KeyCode::Backspace) {
        ui.recipe_search.pop();
    }
    while let Some(c) = get_char_pressed() {
        if !c.is_control() && ui.recipe_search.len() < 48 {
            ui.recipe_search.push(c);
        }
    }
}

fn draw_helmod_production_tree(
    world: &World,
    ui: &mut Ui,
    mouse: (f32, f32),
    vx: f32,
    vy: f32,
    vw: f32,
    vh: f32,
) {
    let reg = content::content();
    handle_recipe_search_chars(ui);

    // Search + summary strip
    let search_h = s(34.0);
    let summary_h = s(52.0);
    let search = Rect {
        x: vx,
        y: vy,
        w: vw * 0.42,
        h: search_h,
    };
    let search_hovered = mouse.0 >= search.x
        && mouse.0 <= search.x + search.w
        && mouse.1 >= search.y
        && mouse.1 <= search.y + search.h;
    if search_hovered && is_mouse_button_pressed(MouseButton::Left) {
        ui.recipe_search_focus = true;
    }
    ui_chrome::text_field_frame(
        search.x,
        search.y,
        search.w,
        search.h,
        ui.recipe_search_focus,
    );
    let search_label = if ui.recipe_search.is_empty() {
        "Search product (e.g. green wire, circuit)…"
    } else {
        ui.recipe_search.as_str()
    };
    draw_text(
        search_label,
        search.x + s(10.0),
        search.y + s(22.0),
        s(14.0),
        if ui.recipe_search.is_empty() {
            TEXT_DIM
        } else {
            TEXT
        },
    );

    // Apply search → pick best matching item that has a recipe.
    if !ui.recipe_search.trim().is_empty() {
        let q = ui.recipe_search.trim().to_ascii_lowercase();
        // Exact recipe / item id hit via registry lookups.
        if let Some(r) = reg.recipe_by_str(ui.recipe_search.trim()) {
            if let Some(out) = r.outputs.first() {
                ui.recipe_root_item = Some(out.item);
            }
        } else if let Some(it) = reg.item_by_str(ui.recipe_search.trim()) {
            ui.recipe_root_item = Some(it.index);
        }
        let mut best: Option<(i32, u16)> = None;
        for item in &reg.items {
            let name = item.name.to_ascii_lowercase();
            let id = item.id.to_ascii_lowercase();
            let cat = item.category.to_ascii_lowercase();
            if !(name.contains(&q) || id.contains(&q) || cat.contains(&q)) {
                continue;
            }
            if reg.best_recipe_for_output(item.index).is_none() && !item.id.starts_with("era1_raw_")
            {
                continue;
            }
            let mut score = 0i32;
            if name.starts_with(&q) || id.ends_with(&q) {
                score += 50;
            }
            if name.contains("green") && q.contains("green") {
                score += 20;
            }
            score -= name.len() as i32;
            if best.map(|(s, _)| score > s).unwrap_or(true) {
                best = Some((score, item.index));
            }
        }
        if let Some((_, idx)) = best {
            if ui.recipe_root_item != Some(idx) {
                ui.recipe_root_item = Some(idx);
                ui.recipe_scroll = 0.0;
            }
        }
    }

    let root = ui.recipe_root_item.or_else(|| {
        reg.item_index("era1_logistics_green_wire")
            .or_else(|| reg.item_index("era1_component_basic_circuit"))
    });
    let Some(root) = root else {
        draw_text(
            "No recipes loaded.",
            vx + s(12.0),
            vy + s(60.0),
            s(16.0),
            UI_TEXT_DIM,
        );
        return;
    };
    ui.recipe_root_item = Some(root);

    let rows = reg.production_tree(root, 400);
    // Summarize leaf ingredients + waste across the tree.
    let mut ingredient_tot: HashMap<u16, f32> = HashMap::new();
    let mut byproduct_tot: HashMap<u16, f32> = HashMap::new();
    for row in &rows {
        let Some(rid) = row.recipe else {
            if row.depth > 0 {
                *ingredient_tot.entry(row.item).or_default() += 1.0;
            }
            continue;
        };
        let Some(r) = reg.recipe(rid) else {
            continue;
        };
        for w in &r.waste {
            *byproduct_tot.entry(w.item).or_default() += w.amount;
        }
        // Leaves: inputs with no further producer expansion in this walk show as depth children;
        // also count extract/raw recipe outputs' absence of inputs.
        if r.inputs.is_empty() {
            *ingredient_tot.entry(row.item).or_default() += 1.0;
        }
    }
    for row in &rows {
        if row.recipe.is_none() && !row.cyclic {
            *ingredient_tot.entry(row.item).or_default() += 1.0;
        }
    }

    // Summary chips
    let sum_y = vy + search_h + s(6.0);
    draw_rectangle(vx, sum_y, vw, summary_h, Color::from_rgba(14, 18, 24, 255));
    draw_rectangle_lines(vx, sum_y, vw, summary_h, 1.0, UI_EDGE);
    if let Some(root_item) = reg.item(root) {
        draw_text("Product", vx + s(8.0), sum_y + s(16.0), s(11.0), UI_TEXT_DIM);
        draw_recipe_chip(
            vx + s(8.0),
            sum_y + s(20.0),
            s(88.0),
            s(24.0),
            &reg.short(Item::from_u16(root_item.index)),
            item_chip_color(&root_item.id, false),
        );
        let meta = format!(
            "E{} · {}{}",
            root_item.era,
            if root_item.category.is_empty() {
                "item"
            } else {
                root_item.category.as_str()
            },
            root_item
                .state
                .as_ref()
                .map(|s| format!(" · {s}"))
                .unwrap_or_default()
        );
        draw_text(&meta, vx + s(8.0), sum_y + s(52.0), s(11.0), UI_TEXT_DIM);
        if !root_item.description.is_empty() {
            let d = if root_item.description.len() > 48 {
                format!("{}…", &root_item.description[..46])
            } else {
                root_item.description.clone()
            };
            draw_text(&d, vx + s(100.0), sum_y + s(52.0), s(11.0), UI_TEXT_DIM);
        }
    }
    draw_text(
        "Byproducts",
        vx + s(110.0),
        sum_y + s(16.0),
        s(11.0),
        Color::from_rgba(220, 120, 120, 220),
    );
    let mut bx = vx + s(110.0);
    let mut shown = 0;
    for (&item, _) in byproduct_tot.iter().take(10) {
        if let Some(it) = reg.item(item) {
            draw_recipe_chip(
                bx,
                sum_y + s(20.0),
                s(72.0),
                s(24.0),
                &short_item_label(&it.name),
                item_chip_color(&it.id, true),
            );
            bx += s(76.0);
            shown += 1;
            if bx > vx + vw - s(200.0) {
                break;
            }
        }
    }
    if shown == 0 {
        draw_text("—", vx + s(110.0), sum_y + s(38.0), s(13.0), TEXT_DIM);
    }
    draw_text(
        "Ingredients",
        vx + vw * 0.62,
        sum_y + s(16.0),
        s(11.0),
        Color::from_rgba(120, 200, 130, 220),
    );
    let mut ix = vx + vw * 0.62;
    for (&item, _) in ingredient_tot.iter().take(8) {
        if let Some(it) = reg.item(item) {
            draw_recipe_chip(
                ix,
                sum_y + s(20.0),
                s(72.0),
                s(24.0),
                &short_item_label(&it.name),
                Color::from_rgba(40, 110, 55, 230),
            );
            ix += s(76.0);
            if ix > vx + vw - s(20.0) {
                break;
            }
        }
    }

    // Row list
    let list_y = sum_y + summary_h + s(8.0);
    let list_h = vh - (list_y - vy);
    draw_rectangle(vx, list_y, vw, list_h, Color::from_rgba(5, 8, 12, 250));
    draw_rectangle_lines(vx, list_y, vw, list_h, 1.2, UI_EDGE);

    let row_h = s(30.0);
    let header_h = s(22.0);
    // Column headers
    draw_text("Recipe", vx + s(10.0), list_y + s(16.0), s(12.0), UI_TEXT_DIM);
    draw_text(
        "Machine",
        vx + vw * 0.38,
        list_y + s(16.0),
        s(12.0),
        UI_TEXT_DIM,
    );
    draw_text(
        "Products",
        vx + vw * 0.55,
        list_y + s(16.0),
        s(12.0),
        UI_TEXT_DIM,
    );
    draw_text(
        "Byproducts",
        vx + vw * 0.70,
        list_y + s(16.0),
        s(12.0),
        Color::from_rgba(220, 120, 120, 200),
    );
    draw_text(
        "Ingredients",
        vx + vw * 0.84,
        list_y + s(16.0),
        s(12.0),
        Color::from_rgba(120, 200, 130, 200),
    );

    let body_y = list_y + header_h;
    let body_h = list_h - header_h;
    let content_h = rows.len() as f32 * row_h;
    let max_scroll = (content_h - body_h).max(0.0);
    let over_list = mouse.0 >= vx
        && mouse.0 <= vx + vw
        && mouse.1 >= body_y
        && mouse.1 <= body_y + body_h;
    if over_list {
        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            ui.recipe_scroll = (ui.recipe_scroll - wheel * row_h * 2.0).clamp(0.0, max_scroll);
        }
    }
    ui.recipe_scroll = ui.recipe_scroll.clamp(0.0, max_scroll);

    let first = (ui.recipe_scroll / row_h).floor().max(0.0) as usize;
    let visible = ((body_h / row_h).ceil() as usize) + 2;
    let last = (first + visible).min(rows.len());

    for i in first..last {
        let row = &rows[i];
        let y = body_y + i as f32 * row_h - ui.recipe_scroll;
        if y + row_h < body_y || y > body_y + body_h {
            continue;
        }
        let hovered = over_list
            && mouse.1 >= y
            && mouse.1 <= y + row_h
            && mouse.0 >= vx
            && mouse.0 <= vx + vw;
        if hovered {
            draw_rectangle(
                vx + 1.0,
                y,
                vw - 2.0,
                row_h,
                Color::from_rgba(28, 36, 48, 255),
            );
        } else if i % 2 == 0 {
            draw_rectangle(
                vx + 1.0,
                y,
                vw - 2.0,
                row_h,
                Color::from_rgba(10, 14, 20, 180),
            );
        }

        // Tree connectors (Helmod L-shape)
        let indent = s(14.0);
        let tree_x0 = vx + s(8.0);
        for (di, &open) in row.ancestor_open.iter().enumerate() {
            if open {
                let tx = tree_x0 + di as f32 * indent + s(6.0);
                draw_line(tx, y, tx, y + row_h, 1.0, Color::from_rgba(80, 95, 110, 180));
            }
        }
        if row.depth > 0 {
            let tx = tree_x0 + (row.depth as f32 - 1.0) * indent + s(6.0);
            let mid = y + row_h * 0.5;
            draw_line(tx, y, tx, mid, 1.0, Color::from_rgba(120, 140, 160, 200));
            draw_line(
                tx,
                mid,
                tx + indent - s(2.0),
                mid,
                1.0,
                Color::from_rgba(120, 140, 160, 200),
            );
            if !row.is_last {
                draw_line(
                    tx,
                    mid,
                    tx,
                    y + row_h,
                    1.0,
                    Color::from_rgba(120, 140, 160, 200),
                );
            }
        }

        let label_x = tree_x0 + row.depth as f32 * indent + s(4.0);
        let item = reg.item(row.item);
        let item_name = item.map(|i| i.name.as_str()).unwrap_or("?");
        let item_id = item.map(|i| i.id.as_str()).unwrap_or("");
        let unlocked = row
            .recipe
            .and_then(|rid| reg.recipe(rid))
            .map(|r| world.tech.recipe_unlocked(&r.technology_unlock))
            .unwrap_or(true);

        draw_recipe_chip(
            label_x,
            y + s(4.0),
            s(86.0),
            s(22.0),
            &short_item_label(item_name),
            item_chip_color(item_id, false),
        );
        if row.cyclic {
            draw_text(
                "cycle",
                label_x + s(90.0),
                y + s(20.0),
                s(11.0),
                Color::from_rgba(220, 160, 80, 220),
            );
        }

        // Machine
        if let Some(rid) = row.recipe {
            if let Some(r) = reg.recipe(rid) {
                let mname = r
                    .machine
                    .strip_prefix("era1_machine_")
                    .unwrap_or(r.machine.as_str())
                    .replace('_', " ");
                let mshort = if mname.len() > 18 {
                    format!("{}…", &mname[..16])
                } else {
                    mname
                };
                draw_text(
                    &mshort,
                    vx + vw * 0.38,
                    y + s(20.0),
                    s(12.0),
                    if unlocked { UI_TEXT } else { TEXT_DIM },
                );

                // Products
                let mut px = vx + vw * 0.55;
                for (oi, out) in r.outputs.iter().enumerate() {
                    if let Some(it) = reg.item(out.item) {
                        draw_recipe_chip(
                            px,
                            y + s(4.0),
                            s(64.0),
                            s(22.0),
                            &format!("{:.0}", out.amount),
                            item_chip_color(&it.id, oi > 0),
                        );
                        px += s(68.0);
                        if px > vx + vw * 0.68 {
                            break;
                        }
                    }
                }
                // Byproducts
                let mut wx = vx + vw * 0.70;
                for w in r.waste.iter().chain(r.outputs.iter().skip(1)) {
                    if let Some(it) = reg.item(w.item) {
                        draw_recipe_chip(
                            wx,
                            y + s(4.0),
                            s(58.0),
                            s(22.0),
                            &short_item_label(&it.name),
                            item_chip_color(&it.id, true),
                        );
                        wx += s(62.0);
                        if wx > vx + vw * 0.82 {
                            break;
                        }
                    }
                }
                // Ingredients
                let mut gx = vx + vw * 0.84;
                for inp in &r.inputs {
                    if let Some(it) = reg.item(inp.item) {
                        draw_recipe_chip(
                            gx,
                            y + s(4.0),
                            s(58.0),
                            s(22.0),
                            &short_item_label(&it.name),
                            Color::from_rgba(40, 110, 55, 230),
                        );
                        gx += s(62.0);
                        if gx > vx + vw - s(8.0) {
                            break;
                        }
                    }
                }
            }
        } else {
            draw_text(
                "raw / external",
                vx + vw * 0.38,
                y + s(20.0),
                s(12.0),
                TEXT_DIM,
            );
        }

        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            ui.recipe_root_item = Some(row.item);
            ui.recipe_search = item_name.to_string();
            ui.recipe_scroll = 0.0;
        }
    }

    if max_scroll > 1.0 {
        let bar_w = s(5.0);
        let track_h = body_h - s(8.0);
        let thumb_h = (track_h * (body_h / content_h.max(1.0))).clamp(s(24.0), track_h);
        let thumb_y = body_y + s(4.0) + (track_h - thumb_h) * (ui.recipe_scroll / max_scroll);
        draw_rectangle(
            vx + vw - bar_w - s(4.0),
            thumb_y,
            bar_w,
            thumb_h,
            Color::from_rgba(70, 82, 98, 220),
        );
    }

    draw_text(
        &format!("{} steps · scroll · click row to refocus · Tab full web", rows.len()),
        vx + s(8.0),
        list_y + list_h - s(6.0),
        s(12.0),
        UI_TEXT_DIM,
    );
}

fn draw_full_recipe_tree(
    world: &World,
    ui: &mut Ui,
    mouse: (f32, f32),
    vx: f32,
    vy: f32,
    vw: f32,
    vh: f32,
) {
    draw_rectangle(vx, vy, vw, vh, Color::from_rgba(5, 8, 12, 250));
    draw_rectangle_lines(vx, vy, vw, vh, 1.2, UI_EDGE);

    let reg = content::content();
    let depths = reg.recipe_depths();
    let edges = reg.recipe_dependency_edges();

    if depths.is_empty() {
        draw_text(
            "No recipes loaded.",
            vx + s(20.0),
            vy + s(40.0),
            s(18.0),
            UI_TEXT_DIM,
        );
        return;
    }

    let max_depth = depths.iter().map(|(_, d)| *d).max().unwrap_or(1).max(1);
    let mut columns: Vec<Vec<u16>> = vec![Vec::new(); (max_depth as usize) + 1];
    for &(rid, d) in &depths {
        columns[d as usize].push(rid);
    }
    for col in &mut columns {
        col.sort_by(|&a, &b| {
            let na = reg.recipe(a).map(|r| r.name.as_str()).unwrap_or("");
            let nb = reg.recipe(b).map(|r| r.name.as_str()).unwrap_or("");
            na.cmp(nb)
        });
    }

    // Dense packing — matches the Pyanodons-style full web.
    let col_gap = 56.0;
    let row_gap = 8.0;
    let mut pos: HashMap<u16, (f32, f32)> = HashMap::new();
    let mut max_h = 0.0_f32;
    for (ci, col) in columns.iter().enumerate() {
        let n = col.len().max(1) as f32;
        let height = (n - 1.0) * row_gap;
        max_h = max_h.max(height);
        let x0 = ci as f32 * col_gap;
        for (ri, &rid) in col.iter().enumerate() {
            let y0 = ri as f32 * row_gap - height * 0.5;
            pos.insert(rid, (x0, y0));
        }
    }
    let graph_w = (columns.len().saturating_sub(1) as f32) * col_gap;
    let graph_h = max_h;

    if ui.recipe_fit_pending {
        ui.recipe_cam_x = graph_w * 0.5;
        ui.recipe_cam_y = 0.0;
        let zx = if graph_w > 1.0 {
            (vw * 0.94) / graph_w
        } else {
            1.0
        };
        let zy = if graph_h > 1.0 {
            (vh * 0.90) / graph_h.max(row_gap * 12.0)
        } else {
            1.0
        };
        ui.recipe_zoom = zx.min(zy).clamp(0.08, 2.2);
        ui.recipe_fit_pending = false;
    }

    let zoom = ui.recipe_zoom;
    let show_labels = zoom >= 1.6;
    let show_names_mid = zoom >= 1.05;
    let node_half = (2.4 * zoom).clamp(1.2, 6.5);
    let edge_w = (0.55 * zoom).clamp(0.25, 1.35);

    let pad = 24.0;
    let in_view = |sx: f32, sy: f32| {
        sx >= vx - pad && sx <= vx + vw + pad && sy >= vy - pad && sy <= vy + vh + pad
    };

    // Edges: producer recipe → consumer recipe.
    for &(from, to) in &edges {
        let Some(&(gx0, gy0)) = pos.get(&from) else {
            continue;
        };
        let Some(&(gx1, gy1)) = pos.get(&to) else {
            continue;
        };
        let (x0, y0) = recipe_graph_to_screen(ui, vx, vy, vw, vh, gx0, gy0);
        let (x1, y1) = recipe_graph_to_screen(ui, vx, vy, vw, vh, gx1, gy1);
        if !in_view(x0, y0) && !in_view(x1, y1) {
            continue;
        }
        let span = (x1 - x0).abs() + (y1 - y0).abs();
        if zoom < 0.35 && span > vw * 0.9 {
            continue;
        }
        let unlocked = reg
            .recipe(to)
            .map(|r| world.tech.recipe_unlocked(&r.technology_unlock))
            .unwrap_or(true);
        let cat = reg.recipe(to).map(|r| r.category.as_str()).unwrap_or("");
        let mut c = recipe_node_color(cat, unlocked);
        c.a = if unlocked { 0.38 } else { 0.15 };
        let mx = (x0 + x1) * 0.5;
        draw_line(x0, y0, mx, y0, edge_w, c);
        draw_line(mx, y0, mx, y1, edge_w, c);
        draw_line(mx, y1, x1, y1, edge_w, c);
    }

    let mut hover: Option<u16> = None;
    let mut hover_screen = (0.0_f32, 0.0_f32);
    for (&rid, &(gx, gy)) in &pos {
        let (sx, sy) = recipe_graph_to_screen(ui, vx, vy, vw, vh, gx, gy);
        if !in_view(sx, sy) {
            continue;
        }
        let Some(r) = reg.recipe(rid) else {
            continue;
        };
        let unlocked = world.tech.recipe_unlocked(&r.technology_unlock);
        let fill = recipe_node_color(&r.category, unlocked);
        let hs = node_half;
        draw_rectangle(sx - hs, sy - hs, hs * 2.0, hs * 2.0, fill);
        if zoom >= 0.85 {
            draw_rectangle_lines(
                sx - hs,
                sy - hs,
                hs * 2.0,
                hs * 2.0,
                1.0,
                Color::from_rgba(230, 235, 245, if unlocked { 120 } else { 50 }),
            );
        }

        if show_names_mid {
            let label = if show_labels {
                r.name.clone()
            } else if r.name.len() > 12 {
                format!("{}…", &r.name[..10])
            } else {
                r.name.clone()
            };
            let fs = (9.0 * zoom).clamp(7.0, 12.0);
            let tw = measure_text(&label, None, fs as u16, 1.0).width;
            draw_text(
                &label,
                sx - tw * 0.5,
                sy + hs + fs + 1.0,
                fs,
                if unlocked {
                    Color::from_rgba(210, 220, 230, 210)
                } else {
                    Color::from_rgba(120, 130, 145, 150)
                },
            );
        }

        let hit_r = hs + 3.5;
        if (mouse.0 - sx).abs() <= hit_r && (mouse.1 - sy).abs() <= hit_r {
            hover = Some(rid);
            hover_screen = (sx, sy);
        }
    }

    if let Some(rid) = hover {
        let (sx, sy) = hover_screen;
        let hs = node_half + 2.0;
        draw_rectangle_lines(sx - hs, sy - hs, hs * 2.0, hs * 2.0, 2.0, UI_CYAN);
        if let Some(r) = reg.recipe(rid) {
            let unlocked = world.tech.recipe_unlocked(&r.technology_unlock);
            let mut lines = vec![r.name.clone(), r.id.clone()];
            lines.push(format!(
                "{} · {:.1}s · {}",
                r.machine,
                r.processing_time,
                if unlocked { "unlocked" } else { "locked" }
            ));
            if !r.description.is_empty() {
                let d = if r.description.len() > 64 {
                    format!("{}…", &r.description[..62])
                } else {
                    r.description.clone()
                };
                lines.push(d);
            }
            if !r.grade_effect.is_empty() {
                lines.push(format!("Grade: {}", r.grade_effect));
            }
            if !unlocked {
                lines.push(format!("Tech: {}", r.technology_unlock));
            }
            if !r.inputs.is_empty() {
                let ins: Vec<String> = r
                    .inputs
                    .iter()
                    .filter_map(|io| {
                        reg.item(io.item)
                            .map(|i| format!("{}×{}", io.amount as i32, i.name))
                    })
                    .collect();
                lines.push(format!("In: {}", ins.join(", ")));
            }
            let outs: Vec<String> = r
                .outputs
                .iter()
                .filter_map(|io| {
                    reg.item(io.item)
                        .map(|i| format!("{}×{}", io.amount as i32, i.name))
                })
                .collect();
            if !outs.is_empty() {
                lines.push(format!("Out: {}", outs.join(", ")));
            }
            draw_recipe_hover_card(&lines, mouse.0, mouse.1);
        }
    }

    draw_text(
        &format!("{} recipes", pos.len()),
        vx + s(10.0),
        vy + vh - s(10.0),
        s(12.0),
        UI_TEXT_DIM,
    );
}

fn draw_recipe_hover_card(lines: &[String], mx: f32, my: f32) {
    let fs = s(14.0);
    let line_h = s(18.0);
    let pad_x = s(12.0);
    let pad_y = s(10.0);
    let tw = lines
        .iter()
        .map(|l| measure_text(l, None, fs as u16, 1.0).width)
        .fold(0.0_f32, f32::max);
    let w = tw + pad_x * 2.0;
    let h = pad_y * 2.0 + line_h * lines.len() as f32;
    let x = (mx + s(16.0)).min(screen_width() - w - s(8.0)).max(s(8.0));
    let y = (my - h - s(12.0)).max(s(8.0));
    draw_rectangle(x + 2.0, y + 2.0, w, h, Color::from_rgba(0, 0, 0, 90));
    draw_rectangle(x, y, w, h, Color::from_rgba(12, 16, 22, 240));
    draw_rectangle(x, y, w, 2.0, UI_CYAN);
    draw_rectangle_lines(x, y, w, h, 1.0, UI_EDGE);
    for (i, line) in lines.iter().enumerate() {
        let color = if i == 0 {
            UI_TEXT
        } else if i == 1 {
            UI_CYAN
        } else {
            UI_TEXT_DIM
        };
        draw_text(
            line,
            x + pad_x,
            y + pad_y + line_h * (i as f32 + 0.75),
            fs,
            color,
        );
    }
}

/// Era 1 tech route list — tier columns with prereq / cost / status.
fn draw_tech_route(world: &World, x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, Color::from_rgba(10, 14, 20, 200));
    draw_rectangle_lines(x, y, w, h, 1.0, UI_EDGE);

    let techs = &content::content().techs;
    let mut by_tier: HashMap<u8, Vec<&content::RuntimeTech>> = HashMap::new();
    for t in techs {
        by_tier.entry(t.tier).or_default().push(t);
    }
    let mut tiers: Vec<u8> = by_tier.keys().copied().collect();
    tiers.sort_unstable();
    if tiers.is_empty() {
        draw_text("No technologies loaded.", x + s(12.0), y + s(24.0), s(16.0), UI_TEXT_DIM);
        return;
    }

    let col_w = w / tiers.len() as f32;
    for (ci, tier) in tiers.iter().enumerate() {
        let list = by_tier.get(tier).map(|v| v.as_slice()).unwrap_or(&[]);
        let cx = x + col_w * (ci as f32) + s(8.0);
        draw_text(
            &format!("T{tier}"),
            cx,
            y + s(18.0),
            s(14.0),
            UI_AMBER,
        );
        let mut yy = y + s(36.0);
        for t in list.iter().take(12) {
            let done = world.tech.is_researched(&t.id);
            let ready = world.tech.can_start(&t.id);
            let active = world.tech.active.as_deref() == Some(t.id.as_str());
            let color = if done {
                Color::from_rgba(90, 200, 120, 255)
            } else if active {
                UI_CYAN
            } else if ready {
                UI_TEXT
            } else {
                UI_TEXT_DIM
            };
            let name = if t.name.len() > 22 {
                format!("{}…", &t.name[..20])
            } else {
                t.name.clone()
            };
            draw_text(&format!("#{} {name}", t.index), cx, yy, s(13.0), color);
            yy += s(16.0);
            let blurb = if !t.purpose.is_empty() {
                t.purpose.as_str()
            } else {
                t.description.as_str()
            };
            if !blurb.is_empty() {
                let short = if blurb.len() > 28 {
                    format!("{}…", &blurb[..26])
                } else {
                    blurb.to_string()
                };
                draw_text(&short, cx, yy, s(11.0), UI_TEXT_DIM);
                yy += s(14.0);
            }
            if !t.unlocks.is_empty() {
                draw_text(
                    &format!("E{} · unlocks {}", t.era, t.unlocks.len()),
                    cx,
                    yy,
                    s(11.0),
                    UI_TEXT_DIM,
                );
                yy += s(14.0);
            } else {
                draw_text(&format!("E{}", t.era), cx, yy, s(11.0), UI_TEXT_DIM);
                yy += s(14.0);
            }
            yy += s(4.0);
            if yy > y + h - s(24.0) {
                break;
            }
        }
    }

    if world.era1_complete || world.tech.nexus_complete {
        draw_text(
            "NEXUS ONLINE — Era 1 complete. Era 2 unlocked.",
            x + s(12.0),
            y + h - s(14.0),
            s(14.0),
            Color::from_rgba(255, 200, 90, 255),
        );
    } else if let Some(active) = world.tech.active.as_deref() {
        let bank: f32 = world.tech.science.values().sum();
        draw_text(
            &format!(
                "Researching {} ({:.0}%) · bank {:.0} packs",
                active,
                world.tech.research_progress01() * 100.0,
                bank
            ),
            x + s(12.0),
            y + h - s(14.0),
            s(13.0),
            UI_CYAN,
        );
    } else {
        let bank: f32 = world.tech.science.values().sum();
        if bank > 0.05 {
            draw_text(
                &format!("Science bank: {:.0} packs (labs pull local first)", bank),
                x + s(12.0),
                y + h - s(14.0),
                s(12.0),
                UI_TEXT_DIM,
            );
        }
    }
}

fn draw_drag_ghost(ui: &Ui, mouse: (f32, f32)) {
    let entry = if let Some(entry) = ui.palette_entry() {
        let dx = mouse.0 - ui.palette_drag_origin.0;
        let dy = mouse.1 - ui.palette_drag_origin.1;
        if dx * dx + dy * dy > 36.0 {
            Some(entry)
        } else {
            None
        }
    } else if let Some(i) = ui.hotbar_drag_from {
        let dx = mouse.0 - ui.hotbar_drag_origin.0;
        let dy = mouse.1 - ui.hotbar_drag_origin.1;
        if dx * dx + dy * dy > 36.0 {
            ui.hotbar_entry(i)
        } else {
            None
        }
    } else {
        None
    };
    let Some(entry) = entry else {
        return;
    };
    let size = 48.0;
    let x = mouse.0 - size * 0.5;
    let y = mouse.1 - size * 0.5;
    draw_rectangle(x, y, size, size, Color::from_rgba(20, 24, 30, 220));
    draw_rectangle_lines(x, y, size, size, 2.0, CYAN);
    draw_rectangle(x + 10.0, y + 10.0, size - 20.0, 12.0, entry.swatch());
    draw_text(&entry.short(), x + 6.0, y + 38.0, 14.0, TEXT);
}

fn draw_context_menu(ui: &Ui, mouse: (f32, f32)) {
    let Some(menu) = ui.context_menu.as_ref() else {
        return;
    };
    let r = context_menu_rect(menu);
    let items = context_items(menu.target);
    ui_chrome::panel(r.x, r.y, r.w, r.h);
    for (i, (label, _)) in items.iter().enumerate() {
        let y = r.y + 8.0 + i as f32 * 34.0;
        let hovered = mouse.0 >= r.x
            && mouse.0 <= r.x + r.w
            && mouse.1 >= y
            && mouse.1 <= y + 30.0;
        if hovered {
            draw_rectangle(
                r.x + 4.0,
                y,
                r.w - 8.0,
                30.0,
                Color::from_rgba(36, 52, 58, 255),
            );
            draw_rectangle(r.x + 4.0, y, 3.0, 30.0, UI_CYAN);
        }
        draw_text(
            label,
            r.x + 16.0,
            y + 21.0,
            17.0,
            if hovered { UI_CYAN } else { UI_TEXT },
        );
    }
}

fn draw_and_handle_build_menu(app: &mut App, mouse: (f32, f32)) {
    ui_chrome::scrim(155);

    let r = build_menu_rect();
    let pad = s(14.0);
    let sidebar_w = s(148.0);
    let detail_h = s(64.0);
    let search_h = s(36.0);
    let header_h = s(56.0);

    ui_chrome::panel(r.x, r.y, r.w, r.h);
    ui_chrome::panel_header(
        r.x,
        r.y + s(6.0),
        r.w,
        "Build",
        Some("Click equip · drag to hotbar · 1–9 pin"),
    );

    // --- Left category sidebar ---
    let side_x = r.x + pad;
    let side_y = r.y + header_h;
    let side_h = r.h - header_h - pad;
    draw_rectangle(
        side_x,
        side_y,
        sidebar_w,
        side_h,
        UI_PANEL_INNER,
    );
    draw_rectangle_lines(side_x, side_y, sidebar_w, side_h, 1.0, UI_EDGE);

    let cat_row_h = s(36.0);
    // All + existing categories
    let all_label = "All";
    let categories: Vec<BuildCategory> = BuildCategory::ALL
        .into_iter()
        .filter(|c| app.is_creative() || *c != BuildCategory::Debug)
        .collect();
    let cat_count = 1 + categories.len();
    for i in 0..cat_count {
        let (cat, label): (Option<BuildCategory>, &str) = if i == 0 {
            (None, all_label)
        } else {
            let c = categories[i - 1];
            (Some(c), c.label())
        };
        let y = side_y + s(8.0) + i as f32 * (cat_row_h + s(4.0));
        let row = Rect {
            x: side_x + s(6.0),
            y,
            w: sidebar_w - s(12.0),
            h: cat_row_h,
        };
        let active = app.ui.build_category == cat;
        let hovered = mouse.0 >= row.x
            && mouse.0 <= row.x + row.w
            && mouse.1 >= row.y
            && mouse.1 <= row.y + row.h;
        ui_chrome::sidebar_row(row.x, row.y, row.w, row.h, label, active, hovered);
        if hovered && is_mouse_button_pressed(MouseButton::Left) && app.ui.palette_drag.is_none() {
            app.ui.build_category = cat;
            app.ui.build_scroll = 0.0;
        }
    }

    // --- Content area (search + grid + detail) ---
    let content_x = side_x + sidebar_w + pad;
    let content_w = r.w - sidebar_w - pad * 3.0;
    let content_top = side_y;

    // Search field
    let search = Rect {
        x: content_x,
        y: content_top,
        w: content_w,
        h: search_h,
    };
    let search_hovered = mouse.0 >= search.x
        && mouse.0 <= search.x + search.w
        && mouse.1 >= search.y
        && mouse.1 <= search.y + search.h;
    if search_hovered && is_mouse_button_pressed(MouseButton::Left) {
        app.ui.build_search_focus = true;
    }
    ui_chrome::text_field_frame(search.x, search.y, search.w, search.h, app.ui.build_search_focus);
    let search_label = if app.ui.build_search.is_empty() {
        if app.ui.build_search_focus {
            "Type to filter…"
        } else {
            "Search buildings…"
        }
    } else {
        app.ui.build_search.as_str()
    };
    draw_text(
        search_label,
        search.x + s(12.0),
        search.y + s(24.0),
        s(16.0),
        if app.ui.build_search.is_empty() {
            TEXT_DIM
        } else {
            TEXT
        },
    );

    // Detail strip at bottom of content
    let detail = Rect {
        x: content_x,
        y: r.y + r.h - pad - detail_h,
        w: content_w,
        h: detail_h,
    };

    // Icon grid between search and detail
    let grid = Rect {
        x: content_x,
        y: search.y + search.h + s(10.0),
        w: content_w,
        h: detail.y - (search.y + search.h + s(10.0)) - s(8.0),
    };
    draw_rectangle(grid.x, grid.y, grid.w, grid.h, UI_SLOT);
    draw_rectangle_lines(grid.x, grid.y, grid.w, grid.h, 1.0, UI_EDGE);

    let cell = s(72.0);
    let gap = s(8.0);
    let cols = ((grid.w - gap) / (cell + gap)).floor().max(1.0) as usize;
    let row_stride = cell + gap;
    let items = app.ui.filtered_entries(&app.world.tech, app.is_creative());
    let rows = if items.is_empty() {
        0
    } else {
        (items.len() + cols - 1) / cols
    };
    let content_h = rows as f32 * row_stride + gap;
    let max_scroll = (content_h - grid.h).max(0.0);

    let over_grid = mouse.0 >= grid.x
        && mouse.0 <= grid.x + grid.w
        && mouse.1 >= grid.y
        && mouse.1 <= grid.y + grid.h;
    if over_grid {
        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            app.ui.build_scroll = (app.ui.build_scroll - wheel * row_stride).clamp(0.0, max_scroll);
        }
    }
    app.ui.build_scroll = app.ui.build_scroll.clamp(0.0, max_scroll);

    let mut detail_entry: Option<BuildEntry> = app
        .ui
        .palette_entry()
        .or_else(|| app.ui.current_entry())
        .filter(|e| items.contains(e));

    if items.is_empty() {
        draw_text(
            "No buildings match",
            grid.x + s(16.0),
            grid.y + s(32.0),
            s(18.0),
            TEXT_DIM,
        );
    } else {
        let first_row = (app.ui.build_scroll / row_stride).floor().max(0.0) as usize;
        let visible_rows = ((grid.h / row_stride).ceil() as usize) + 1;
        let last_row = (first_row + visible_rows).min(rows);

        for row in first_row..last_row {
            for col in 0..cols {
                let idx = row * cols + col;
                if idx >= items.len() {
                    break;
                }
                let entry = items[idx];
                let x = grid.x + gap + col as f32 * (cell + gap);
                let y = grid.y + gap + row as f32 * row_stride - app.ui.build_scroll;
                if y + cell < grid.y || y > grid.y + grid.h {
                    continue;
                }
                let hovered = mouse.0 >= x
                    && mouse.0 <= x + cell
                    && mouse.1 >= y
                    && mouse.1 <= y + cell
                    && over_grid;
                let selected = app.ui.current_entry() == Some(entry)
                    || app.ui.palette_entry() == Some(entry);
                if hovered {
                    detail_entry = Some(entry);
                }
                let unlock = entry.tech_unlock();
                let locked = !app.is_creative() && !app.world.tech.machine_unlocked(&unlock);
                // Clip-ish: only draw if mostly inside grid
                draw_rectangle(
                    x,
                    y,
                    cell,
                    cell,
                    if hovered || selected {
                        Color::from_rgba(44, 54, 68, 255)
                    } else {
                        Color::from_rgba(28, 32, 40, 255)
                    },
                );
                draw_rectangle_lines(
                    x,
                    y,
                    cell,
                    cell,
                    1.2,
                    if selected {
                        CYAN
                    } else if hovered {
                        Color::from_rgba(90, 110, 130, 255)
                    } else {
                        NODE_BORDER
                    },
                );
                let sw = s(28.0);
                let mut swatch = entry.swatch();
                if locked {
                    swatch.a = 0.45;
                }
                draw_rectangle(
                    x + (cell - sw) * 0.5,
                    y + s(12.0),
                    sw,
                    s(20.0),
                    swatch,
                );
                let short = entry.short();
                let tw = measure_text(&short, None, s(14.0) as u16, 1.0).width;
                draw_text(
                    &short,
                    x + (cell - tw) * 0.5,
                    y + cell - s(12.0),
                    s(14.0),
                    if locked { TEXT_DIM } else { TEXT },
                );
                // Locked entries are filtered out in Survival; Creative never locks.
                if hovered
                    && is_mouse_button_pressed(MouseButton::Left)
                    && app.ui.palette_drag.is_none()
                {
                    app.ui.set_palette_entry(Some(entry));
                    app.ui.palette_drag_origin = mouse;
                    app.ui.select_entry(entry);
                    app.ui.context_menu = None;
                }
            }
        }

        // Scrollbar when needed
        if max_scroll > 1.0 {
            let bar_w = s(5.0);
            let track_h = grid.h - s(8.0);
            let thumb_h = (track_h * (grid.h / content_h)).clamp(s(24.0), track_h);
            let thumb_y = grid.y
                + s(4.0)
                + (track_h - thumb_h) * (app.ui.build_scroll / max_scroll);
            draw_rectangle(
                grid.x + grid.w - bar_w - s(4.0),
                thumb_y,
                bar_w,
                thumb_h,
                Color::from_rgba(70, 82, 98, 220),
            );
        }
    }

    // Detail strip
    draw_rectangle(
        detail.x,
        detail.y,
        detail.w,
        detail.h,
        Color::from_rgba(24, 28, 36, 255),
    );
    draw_rectangle_lines(detail.x, detail.y, detail.w, detail.h, 1.0, NODE_BORDER);
    if let Some(entry) = detail_entry {
        let unlock = entry.tech_unlock();
        let locked = !app.is_creative() && !app.world.tech.machine_unlocked(&unlock);
        draw_rectangle(
            detail.x + s(14.0),
            detail.y + s(18.0),
            s(28.0),
            s(20.0),
            entry.swatch(),
        );
        draw_text(
            &entry.label(),
            detail.x + s(54.0),
            detail.y + s(24.0),
            s(18.0),
            TEXT,
        );
        let hint = if locked {
            format!("Locked — research {unlock}")
        } else {
            entry.hint()
        };
        draw_text(
            &hint,
            detail.x + s(54.0),
            detail.y + s(46.0),
            s(14.0),
            TEXT_DIM,
        );
    } else {
        draw_text(
            "Select a building",
            detail.x + s(16.0),
            detail.y + s(36.0),
            s(16.0),
            TEXT_DIM,
        );
    }

    // Click dimmer outside panel closes menu (unless dragging).
    if app.ui.palette_drag.is_none()
        && is_mouse_button_pressed(MouseButton::Left)
        && !(mouse.0 >= r.x
            && mouse.0 <= r.x + r.w
            && mouse.1 >= r.y
            && mouse.1 <= r.y + r.h)
        && !point_in_hud_chrome(mouse.0, mouse.1)
    {
        app.ui.close_build();
    }
}
