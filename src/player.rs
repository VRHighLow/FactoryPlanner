//! Simple circle player placeholder — Follow/Free cam, no particle FX.

use macroquad::prelude::*;
use crate::ui_chrome;

pub const PLAYER_SPEED: f32 = 180.0; // ~4.5 tiles/s (TILE=40)
pub const CAM_LEASH_MARGIN: f32 = 72.0;
pub const BODY_R: f32 = 14.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CamMode {
    Follow,
    Free,
}

impl CamMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Follow => Self::Free,
            Self::Free => Self::Follow,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Follow => "Cam · Follow",
            Self::Free => "Cam · Free",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub facing: f32,
    pub cam_mode: CamMode,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            facing: -std::f32::consts::FRAC_PI_2,
            cam_mode: CamMode::Follow,
        }
    }

    pub fn tick(&mut self, dt: f32, wish: Vec2) {
        let wish = if wish.length_squared() > 1e-6 {
            wish.normalize()
        } else {
            Vec2::ZERO
        };
        let target = wish * PLAYER_SPEED;
        let accel = 1100.0;
        let to = target - vec2(self.vx, self.vy);
        let max_dv = accel * dt;
        if to.length() <= max_dv {
            self.vx = target.x;
            self.vy = target.y;
        } else {
            let d = to.normalize() * max_dv;
            self.vx += d.x;
            self.vy += d.y;
        }

        self.x += self.vx * dt;
        self.y += self.vy * dt;

        let speed = vec2(self.vx, self.vy).length();
        if speed > 10.0 {
            self.facing = lerp_angle(self.facing, self.vy.atan2(self.vx), (12.0 * dt).min(1.0));
        }
    }
}

/// Networked peer: authoritative UPS samples + dead-reckon between them.
#[derive(Clone, Debug)]
pub struct RemoteDrone {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub facing: f32,
}

impl RemoteDrone {
    pub fn new(x: f32, y: f32, facing: f32) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            facing,
        }
    }

    /// Snap to what the peer simulated this UPS tick (no soft chase).
    pub fn apply_net(&mut self, x: f32, y: f32, vx: f32, vy: f32, facing: f32) {
        self.x = x;
        self.y = y;
        self.vx = vx;
        self.vy = vy;
        self.facing = facing;
    }

    pub fn tick(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut d = b - a;
    while d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    }
    while d < -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    a + d * t
}

fn to_screen(wx: f32, wy: f32, cam_x: f32, cam_y: f32, zoom: f32) -> (f32, f32) {
    (
        ((wx - cam_x) * zoom + ui_chrome::ui_width() * 0.5).round(),
        ((wy - cam_y) * zoom + ui_chrome::ui_height() * 0.5).round(),
    )
}

fn draw_player_circle(
    x: f32,
    y: f32,
    cam_x: f32,
    cam_y: f32,
    zoom: f32,
    accent: Color,
    name: Option<&str>,
) {
    let (sx, sy) = to_screen(x, y, cam_x, cam_y, zoom);
    let r = BODY_R * zoom;
    draw_circle(sx, sy + 3.0 * zoom, r * 0.85, Color::from_rgba(0, 0, 0, 50));
    draw_circle(sx, sy, r, accent);
    draw_circle_lines(sx, sy, r, (2.0 * zoom).max(1.5), Color::from_rgba(255, 255, 255, 200));
    draw_circle(sx, sy, r * 0.35, Color::from_rgba(255, 255, 255, 90));

    if let Some(label) = name {
        let fs = (13.0 * zoom).clamp(11.0, 16.0);
        let tw = measure_text(label, None, fs as u16, 1.0).width;
        draw_text(
            label,
            sx - tw * 0.5,
            sy + r * 1.35 + fs,
            fs,
            Color {
                r: accent.r,
                g: accent.g,
                b: accent.b,
                a: 0.95,
            },
        );
    }
}

pub fn draw_player(
    player: &Player,
    cam_x: f32,
    cam_y: f32,
    zoom: f32,
    accent: Color,
    name: Option<&str>,
) {
    draw_player_circle(player.x, player.y, cam_x, cam_y, zoom, accent, name);
}

/// Remote peer circle + nameplate.
pub fn draw_drone_remote(
    drone: &RemoteDrone,
    cam_x: f32,
    cam_y: f32,
    zoom: f32,
    accent: Color,
    name: Option<&str>,
) {
    draw_player_circle(drone.x, drone.y, cam_x, cam_y, zoom, accent, name);
}

pub fn movement_wish() -> Vec2 {
    let mut w = Vec2::ZERO;
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        w.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        w.y += 1.0;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        w.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        w.x += 1.0;
    }
    w
}

pub fn clamp_cam_to_player(cam_x: &mut f32, cam_y: &mut f32, zoom: f32, px: f32, py: f32) {
    let sx = (px - *cam_x) * zoom + ui_chrome::ui_width() * 0.5;
    let sy = (py - *cam_y) * zoom + ui_chrome::ui_height() * 0.5;
    // Soft leash — start pulling before the edge so Free→Follow doesn't yank.
    let w = ui_chrome::ui_width();
    let h = ui_chrome::ui_height();
    let m = CAM_LEASH_MARGIN;
    if sx < m {
        *cam_x -= (m - sx) / zoom;
    } else if sx > w - m {
        *cam_x += (sx - (w - m)) / zoom;
    }
    if sy < m {
        *cam_y -= (m - sy) / zoom;
    } else if sy > h - m {
        *cam_y += (sy - (h - m)) / zoom;
    }
}

pub fn snap_cam_to_player(cam_x: &mut f32, cam_y: &mut f32, px: f32, py: f32) {
    *cam_x = px;
    *cam_y = py;
}
