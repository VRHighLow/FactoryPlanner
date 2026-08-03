//! Hover drone player: detailed hull, layered jet trails, Follow/Free cam.

use macroquad::prelude::*;

pub const PLAYER_SPEED: f32 = 160.0;
pub const CAM_LEASH_MARGIN: f32 = 72.0;

const BODY_R: f32 = 15.0;
const TRAIL_CAP: usize = 120;
const TRAIL_SPAWN_RATE: f32 = 90.0;
const TRAIL_LIFE: f32 = 0.65;

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

#[derive(Clone, Copy, Debug)]
struct TrailPuff {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    age: f32,
    life: f32,
    size: f32,
    /// 0 cool / 1 hot core
    warm: f32,
    /// Elongation along velocity for jet streaks.
    stretch: f32,
    side: f32,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub facing: f32,
    pub cam_mode: CamMode,
    bob: f32,
    trail: Vec<TrailPuff>,
    spawn_acc: f32,
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
            bob: 0.0,
            trail: Vec::with_capacity(TRAIL_CAP),
            spawn_acc: 0.0,
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

        self.bob += dt * (2.4 + speed * 0.01);
        update_trails(
            &mut self.trail,
            &mut self.spawn_acc,
            dt,
            self.x,
            self.y,
            self.vx,
            self.vy,
            self.facing,
            speed,
            wish.length() > 0.1,
        );
    }
}

/// Networked peer drone: smooth chase + local thruster FX (effects aren't synced).
#[derive(Clone, Debug)]
pub struct RemoteDrone {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub facing: f32,
    bob: f32,
    trail: Vec<TrailPuff>,
    spawn_acc: f32,
    target_x: f32,
    target_y: f32,
    target_facing: f32,
}

impl RemoteDrone {
    pub fn new(x: f32, y: f32, facing: f32) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            facing,
            bob: 0.0,
            trail: Vec::with_capacity(TRAIL_CAP),
            spawn_acc: 0.0,
            target_x: x,
            target_y: y,
            target_facing: facing,
        }
    }

    pub fn set_target(&mut self, x: f32, y: f32, facing: f32) {
        self.target_x = x;
        self.target_y = y;
        self.target_facing = facing;
    }

    pub fn tick(&mut self, dt: f32) {
        // Critically-damped-ish chase so packets don't snap the hull.
        let follow = 1.0 - (-16.0 * dt).exp();
        let nx = self.x + (self.target_x - self.x) * follow;
        let ny = self.y + (self.target_y - self.y) * follow;
        if dt > 1e-6 {
            // Blend measured velocity so trail kick stays stable.
            let mvx = (nx - self.x) / dt;
            let mvy = (ny - self.y) / dt;
            self.vx = self.vx * 0.35 + mvx * 0.65;
            self.vy = self.vy * 0.35 + mvy * 0.65;
        }
        self.x = nx;
        self.y = ny;

        let speed = vec2(self.vx, self.vy).length();
        let face_t = if speed > 12.0 { 14.0 } else { 8.0 };
        self.facing = lerp_angle(
            self.facing,
            self.target_facing,
            (face_t * dt).min(1.0),
        );

        self.bob += dt * (2.4 + speed * 0.01);
        let thrusting = speed > 18.0;
        update_trails(
            &mut self.trail,
            &mut self.spawn_acc,
            dt,
            self.x,
            self.y,
            self.vx,
            self.vy,
            self.facing,
            speed,
            thrusting,
        );
    }
}

fn update_trails(
    trail: &mut Vec<TrailPuff>,
    spawn_acc: &mut f32,
    dt: f32,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    facing: f32,
    speed: f32,
    thrusting: bool,
) {
    for p in trail.iter_mut() {
        p.age += dt;
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        // Curl slightly outward so the wake feels turbulent.
        let ox = -p.vy * 0.35 * p.side * dt;
        let oy = p.vx * 0.35 * p.side * dt;
        p.vx += ox;
        p.vy += oy;
        p.vx *= 1.0 - 2.2 * dt;
        p.vy *= 1.0 - 2.2 * dt;
        p.size *= 1.0 + 0.55 * dt;
        p.warm *= 1.0 - 1.1 * dt;
    }
    trail.retain(|p| p.age < p.life);

    let throttle = (speed / PLAYER_SPEED).clamp(0.0, 1.0);
    let rate = if thrusting {
        TRAIL_SPAWN_RATE * (0.4 + 0.6 * throttle)
    } else {
        14.0
    };
    *spawn_acc += rate * dt;

    let fx = facing.cos();
    let fy = facing.sin();
    let px = -fy;
    let py = fx;

    while *spawn_acc >= 1.0 && trail.len() < TRAIL_CAP {
        *spawn_acc -= 1.0;
        let side = if rand_f() < 0.5 { -1.0 } else { 1.0 };
        // Twin nozzles at the stern.
        let along = -11.0 + (rand_f() - 0.5) * 2.0;
        let lateral = side * (5.2 + rand_f() * 1.2);
        let ox = fx * along + px * lateral;
        let oy = fy * along + py * lateral;

        let kick = if thrusting {
            36.0 + throttle * 55.0
        } else {
            14.0
        };
        let jx = (rand_f() - 0.5) * 12.0;
        let jy = (rand_f() - 0.5) * 12.0;
        let tvx = -fx * kick + jx - vx * 0.2;
        let tvy = -fy * kick + jy - vy * 0.2;

        // Mix of hot cores and cooler smoke shells.
        let layer = rand_f();
        let (warm, size, life, stretch) = if layer < 0.35 {
            (
                0.85 + rand_f() * 0.15,
                2.0 + throttle * 1.5,
                TRAIL_LIFE * 0.45,
                2.8 + throttle * 2.0,
            )
        } else if layer < 0.7 {
            (
                0.45 + rand_f() * 0.25,
                3.2 + throttle * 2.2,
                TRAIL_LIFE * 0.75,
                2.0 + throttle,
            )
        } else {
            (
                0.08 + rand_f() * 0.12,
                4.5 + throttle * 3.0,
                TRAIL_LIFE * (0.9 + rand_f() * 0.3),
                1.2,
            )
        };

        trail.push(TrailPuff {
            x: x + ox,
            y: y + oy,
            vx: tvx,
            vy: tvy,
            age: 0.0,
            life: life * (0.85 + rand_f() * 0.3),
            size: if thrusting { size } else { size * 0.65 },
            warm: if thrusting { warm } else { warm * 0.25 },
            stretch: if thrusting { stretch } else { 1.0 },
            side,
        });
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

fn rand_f() -> f32 {
    macroquad::rand::gen_range(0.0, 1.0)
}

fn mix_c(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
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
    draw_drone_full(
        player.x,
        player.y,
        player.facing,
        player.bob,
        player.vx,
        player.vy,
        cam_x,
        cam_y,
        zoom,
        accent,
        Some(&player.trail),
        name,
    );
}

/// Remote peer drone with local thruster FX + nameplate.
pub fn draw_drone_remote(
    drone: &RemoteDrone,
    cam_x: f32,
    cam_y: f32,
    zoom: f32,
    accent: Color,
    name: Option<&str>,
) {
    draw_drone_full(
        drone.x,
        drone.y,
        drone.facing,
        drone.bob,
        drone.vx,
        drone.vy,
        cam_x,
        cam_y,
        zoom,
        accent,
        Some(&drone.trail),
        name,
    );
}

fn draw_drone_full(
    x: f32,
    y: f32,
    facing: f32,
    bob_phase: f32,
    vx: f32,
    vy: f32,
    cam_x: f32,
    cam_y: f32,
    zoom: f32,
    accent: Color,
    trail: Option<&[TrailPuff]>,
    name: Option<&str>,
) {
    let to_screen = |wx: f32, wy: f32| -> (f32, f32) {
        (
            ((wx - cam_x) * zoom + screen_width() * 0.5).round(),
            ((wy - cam_y) * zoom + screen_height() * 0.5).round(),
        )
    };

    let bob = bob_phase.sin() * 1.4;
    let body_y = y + bob;
    let fx = facing.cos();
    let fy = facing.sin();
    let px = -fy;
    let py = fx;

    if let Some(trail) = trail {
        for p in trail {
            let t = (p.age / p.life).clamp(0.0, 1.0);
            let fade = (1.0 - t).powf(1.25);
            let (sx, sy) = to_screen(p.x, p.y + bob * 0.2);
            let spd = (p.vx * p.vx + p.vy * p.vy).sqrt().max(1.0);
            let ux = p.vx / spd;
            let uy = p.vy / spd;
            let len = p.size * p.stretch * zoom * (0.8 + 0.4 * fade);
            let wid = p.size * zoom * (0.55 + 0.25 * (1.0 - p.warm)) * fade.max(0.15);

            let hot = Color::from_rgba(255, 230, 190, (210.0 * fade * p.warm) as u8);
            let mid = Color {
                r: accent.r * 0.55 + 0.45,
                g: accent.g * 0.35 + 0.25,
                b: accent.b * 0.2 + 0.1,
                a: 0.55 * fade,
            };
            let cool = Color {
                r: accent.r * 0.35,
                g: accent.g * 0.55,
                b: accent.b * 0.7,
                a: 0.28 * fade * (1.0 - p.warm * 0.5),
            };
            let c = mix_c(cool, mix_c(mid, hot, p.warm), p.warm);

            draw_circle(
                sx,
                sy,
                wid * 2.2,
                Color {
                    r: c.r,
                    g: c.g,
                    b: c.b,
                    a: c.a * 0.25,
                },
            );
            draw_line(
                sx - ux * len * 0.15,
                sy - uy * len * 0.15,
                sx + ux * len,
                sy + uy * len,
                wid * 1.6,
                Color {
                    r: c.r,
                    g: c.g,
                    b: c.b,
                    a: c.a * 0.35,
                },
            );
            draw_line(sx, sy, sx + ux * len * 0.85, sy + uy * len * 0.85, wid, c);
            if p.warm > 0.55 {
                draw_circle(
                    sx,
                    sy,
                    wid * 0.45,
                    Color::from_rgba(255, 250, 230, (180.0 * fade) as u8),
                );
            }
        }
    }

    let (bx, by) = to_screen(x, body_y);
    let r = BODY_R * zoom;

    let (shx, shy) = to_screen(x, y);
    draw_ellipse(
        shx,
        shy + 5.5 * zoom,
        r * 0.9,
        r * 0.28,
        0.0,
        Color::from_rgba(0, 0, 0, 45),
    );

    let speed = (vx * vx + vy * vy).sqrt();
    let throttle = (speed / PLAYER_SPEED).clamp(0.0, 1.0);

    for side in [-1.0, 1.0] {
        let nx = x - fx * 10.0 + px * 5.4 * side;
        let ny = body_y - fy * 10.0 + py * 5.4 * side;
        let (sx, sy) = to_screen(nx, ny);
        draw_circle(
            sx + fx * 2.0 * zoom,
            sy + fy * 2.0 * zoom,
            3.2 * zoom,
            Color::from_rgba(40, 50, 60, 255),
        );
        draw_circle_lines(
            sx + fx * 2.0 * zoom,
            sy + fy * 2.0 * zoom,
            3.2 * zoom,
            1.2,
            Color {
                r: accent.r,
                g: accent.g,
                b: accent.b,
                a: 0.75,
            },
        );
        let ga = 0.25 + throttle * 0.45;
        draw_circle(
            sx - fx * 2.0 * zoom,
            sy - fy * 2.0 * zoom,
            (5.0 + throttle * 4.0) * zoom,
            Color {
                r: accent.r,
                g: accent.g * 0.55,
                b: accent.b * 0.35,
                a: ga * 0.35,
            },
        );
        draw_circle(
            sx,
            sy,
            (1.5 + throttle * 1.5) * zoom,
            Color::from_rgba(255, 250, 220, (ga * 255.0) as u8),
        );
    }

    let armor = Color::from_rgba(44, 62, 76, 255);
    let armor_hi = mix_c(armor, accent, 0.22);
    let armor_lo = Color::from_rgba(28, 38, 48, 255);
    let rim = mix_c(Color::from_rgba(130, 180, 205, 255), accent, 0.45);
    let panel = mix_c(Color::from_rgba(55, 78, 94, 255), accent, 0.15);

    let nose = vec2(bx + fx * r * 1.15, by + fy * r * 1.15);
    let tail = vec2(bx - fx * r * 0.95, by - fy * r * 0.95);
    let left = vec2(bx + px * r * 0.78, by + py * r * 0.78);
    let right = vec2(bx - px * r * 0.78, by - py * r * 0.78);

    draw_triangle(nose, left, tail, armor_hi);
    draw_triangle(nose, tail, right, armor_lo);
    let spine_l = vec2(
        bx + px * r * 0.22 - fx * r * 0.15,
        by + py * r * 0.22 - fy * r * 0.15,
    );
    let spine_r = vec2(
        bx - px * r * 0.22 - fx * r * 0.15,
        by - py * r * 0.22 - fy * r * 0.15,
    );
    draw_triangle(nose, spine_l, spine_r, panel);

    for side in [-1.0, 1.0] {
        let s = if side > 0.0 { left } else { right };
        let mid = vec2(
            bx + px * r * 0.42 * side - fx * r * 0.05,
            by + py * r * 0.42 * side - fy * r * 0.05,
        );
        let aft = vec2(
            bx + px * r * 0.5 * side - fx * r * 0.55,
            by + py * r * 0.5 * side - fy * r * 0.55,
        );
        draw_triangle(s, mid, aft, armor);
        draw_line(
            s.x,
            s.y,
            aft.x,
            aft.y,
            (1.4 * zoom).max(1.0),
            Color {
                r: rim.r,
                g: rim.g,
                b: rim.b,
                a: 0.55,
            },
        );
    }

    for (a, b) in [(nose, left), (left, tail), (tail, right), (right, nose)] {
        draw_line(
            a.x,
            a.y,
            b.x,
            b.y,
            (3.2 * zoom).max(1.8),
            Color {
                r: accent.r,
                g: accent.g,
                b: accent.b,
                a: 0.22,
            },
        );
        draw_line(a.x, a.y, b.x, b.y, (1.5 * zoom).max(1.0), rim);
    }

    for side in [-1.0, 1.0] {
        let root = vec2(
            bx + px * r * 0.58 * side - fx * r * 0.05,
            by + py * r * 0.58 * side - fy * r * 0.05,
        );
        let tip = vec2(
            bx + px * r * 1.25 * side - fx * r * 0.45,
            by + py * r * 1.25 * side - fy * r * 0.45,
        );
        let tip2 = vec2(
            bx + px * r * 1.05 * side - fx * r * 0.2,
            by + py * r * 1.05 * side - fy * r * 0.2,
        );
        draw_triangle(root, tip, tip2, armor_lo);
        draw_line(root.x, root.y, tip.x, tip.y, (2.0 * zoom).max(1.2), rim);
        draw_circle(tip.x, tip.y, 1.6 * zoom, accent);
    }

    let cx = bx + fx * r * 0.18;
    let cy = by + fy * r * 0.18;
    draw_circle(cx, cy, r * 0.5, Color::from_rgba(12, 20, 28, 255));
    draw_circle(cx, cy, r * 0.38, mix_c(Color::from_rgba(30, 70, 80, 255), accent, 0.35));
    draw_circle(cx, cy, r * 0.32, accent);
    draw_circle(
        cx - px * r * 0.08 - fx * r * 0.06,
        cy - py * r * 0.08 - fy * r * 0.06,
        r * 0.12,
        Color::from_rgba(210, 255, 250, 140),
    );
    draw_circle(
        cx + fx * r * 0.08,
        cy + fy * r * 0.08,
        r * 0.1,
        Color::from_rgba(14, 28, 36, 230),
    );
    draw_circle_lines(cx, cy, r * 0.5, 1.3, rim);

    let ax0 = bx + fx * r * 0.55 + px * r * 0.35;
    let ay0 = by + fy * r * 0.55 + py * r * 0.35;
    let ax1 = bx + fx * r * 0.55 - px * r * 0.35;
    let ay1 = by + fy * r * 0.55 - py * r * 0.35;
    draw_line(ax0, ay0, ax1, ay1, (2.0 * zoom).max(1.1), Color::from_rgba(40, 50, 58, 255));
    draw_line(ax0, ay0, ax1, ay1, (1.0 * zoom).max(0.8), rim);
    let ant = vec2(bx + fx * r * 0.85, by + fy * r * 0.85);
    draw_line(cx, cy, ant.x, ant.y, (1.4 * zoom).max(1.0), rim);
    draw_circle(ant.x, ant.y, 2.0 * zoom, accent);
    draw_circle(
        ant.x,
        ant.y,
        3.4 * zoom,
        Color {
            r: accent.r,
            g: accent.g,
            b: accent.b,
            a: 0.2,
        },
    );

    for i in 0..4 {
        let a = facing + i as f32 * std::f32::consts::FRAC_PI_2 + 0.4;
        let rr = r * 0.55;
        draw_circle(
            bx + a.cos() * rr,
            by + a.sin() * rr,
            1.3 * zoom,
            Color::from_rgba(160, 190, 210, 200),
        );
    }

    if let Some(label) = name {
        let fs = (13.0 * zoom).clamp(11.0, 16.0);
        let tw = measure_text(label, None, fs as u16, 1.0).width;
        let (nx, ny) = to_screen(x, y);
        draw_text(
            label,
            nx - tw * 0.5,
            ny + r * 1.35 + fs,
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
    let sx = (px - *cam_x) * zoom + screen_width() * 0.5;
    let sy = (py - *cam_y) * zoom + screen_height() * 0.5;
    let m = CAM_LEASH_MARGIN;
    let w = screen_width();
    let h = screen_height();
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
