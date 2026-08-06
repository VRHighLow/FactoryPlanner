//! Shared UI chrome — industrial factory look (panels, buttons, slots, tooltips).

use macroquad::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

pub const UI_PANEL: Color = Color::from_rgba(16, 20, 28, 250);
pub const UI_PANEL_INNER: Color = Color::from_rgba(12, 15, 20, 255);
pub const UI_SLOT: Color = Color::from_rgba(22, 27, 35, 255);
pub const UI_SLOT_HOVER: Color = Color::from_rgba(32, 40, 52, 255);
pub const UI_EDGE: Color = Color::from_rgba(70, 88, 108, 160);
pub const UI_EDGE_BRIGHT: Color = Color::from_rgba(120, 150, 175, 210);
pub const UI_CYAN: Color = Color::from_rgba(72, 220, 205, 255);
pub const UI_CYAN_DIM: Color = Color::from_rgba(48, 140, 130, 255);
pub const UI_AMBER: Color = Color::from_rgba(255, 168, 72, 255);
pub const UI_TEXT: Color = Color::from_rgba(232, 238, 246, 255);
pub const UI_TEXT_DIM: Color = Color::from_rgba(148, 162, 178, 255);
pub const UI_DANGER: Color = Color::from_rgba(220, 90, 90, 255);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

/// Draw / layout size — same coordinate space as `mouse_position()`.
#[inline]
pub fn ui_width() -> f32 {
    screen_width()
}

#[inline]
pub fn ui_height() -> f32 {
    screen_height()
}

/// Pointer in screen/layout space. Do not remap — that drifts vs drawn hitboxes.
#[inline]
pub fn pointer() -> (f32, f32) {
    mouse_position()
}

#[inline]
pub fn point_in(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    mx >= x && mx < x + w && my >= y && my < y + h
}

pub fn copy_to_clipboard(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    miniquad::window::clipboard_set(text);
    true
}

fn anim_t(key: u64, target: f32) -> f32 {
    thread_local! {
        static ANIM: RefCell<HashMap<u64, f32>> = RefCell::new(HashMap::new());
    }
    let dt = get_frame_time().clamp(0.0, 0.05);
    let speed = 14.0;
    ANIM.with(|cell| {
        let mut map = cell.borrow_mut();
        let t = map.entry(key).or_insert(0.0);
        *t += (target - *t) * (1.0 - (-speed * dt).exp());
        if *t < 0.002 && target == 0.0 {
            *t = 0.0;
        }
        *t
    })
}

fn rect_key(x: f32, y: f32, w: f32, h: f32) -> u64 {
    let bits = |v: f32| v.to_bits() as u64;
    bits(x) ^ bits(y).rotate_left(16) ^ bits(w).rotate_left(32) ^ bits(h).rotate_left(48)
}

pub fn scrim(alpha: u8) {
    draw_rectangle(
        0.0,
        0.0,
        ui_width(),
        ui_height(),
        Color::from_rgba(4, 6, 10, alpha),
    );
}

/// Layered panel with top accent bar + soft inner frame.
pub fn panel(x: f32, y: f32, w: f32, h: f32) {
    // Soft multi-layer shadow
    draw_rectangle(
        x + 8.0,
        y + 12.0,
        w,
        h,
        Color::from_rgba(0, 0, 0, 50),
    );
    draw_rectangle(
        x + 3.0,
        y + 5.0,
        w,
        h,
        Color::from_rgba(0, 0, 0, 80),
    );
    draw_rectangle(x, y, w, h, UI_PANEL);
    // Top accent strip
    draw_rectangle(x, y, w, 3.0, UI_CYAN);
    draw_rectangle(x, y + 3.0, w, 1.0, Color::from_rgba(120, 200, 190, 50));
    // Outer edge
    draw_rectangle_lines(x, y, w, h, 1.5, UI_EDGE_BRIGHT);
    // Inner inset
    draw_rectangle_lines(
        x + 4.0,
        y + 6.0,
        w - 8.0,
        h - 10.0,
        1.0,
        Color::from_rgba(40, 50, 64, 110),
    );
}

pub fn panel_header(x: f32, y: f32, _w: f32, title: &str, subtitle: Option<&str>) {
    draw_text(title, x + 18.0, y + 30.0, 24.0, UI_TEXT);
    let tw = measure_text(title, None, 24, 1.0).width;
    draw_rectangle(x + 18.0, y + 36.0, tw.max(40.0) * 0.55, 2.0, UI_CYAN);
    if let Some(sub) = subtitle {
        draw_text(sub, x + 18.0 + tw + 14.0, y + 28.0, 14.0, UI_TEXT_DIM);
    }
}

pub fn floating_bar(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(
        x + 2.0,
        y + 3.0,
        w,
        h,
        Color::from_rgba(0, 0, 0, 70),
    );
    draw_rectangle(x, y, w, h, Color::from_rgba(16, 20, 26, 210));
    draw_rectangle(x, y, w, 2.0, Color::from_rgba(72, 220, 205, 90));
    draw_rectangle_lines(x, y, w, h, 1.2, UI_EDGE);
}

pub fn slot_frame(x: f32, y: f32, size: f32, hovered: bool, selected: bool, drop: bool) {
    let fill = if drop {
        Color::from_rgba(28, 58, 50, 230)
    } else if hovered {
        UI_SLOT_HOVER
    } else {
        UI_SLOT
    };
    draw_rectangle(x, y, size, size, fill);
    draw_line(
        x + 1.0,
        y + 1.0,
        x + size - 1.0,
        y + 1.0,
        1.0,
        Color::from_rgba(255, 255, 255, 18),
    );
    draw_line(
        x + 1.0,
        y + size - 1.0,
        x + size - 1.0,
        y + size - 1.0,
        1.0,
        Color::from_rgba(0, 0, 0, 50),
    );
    let (thick, edge) = if drop {
        (2.2, UI_CYAN)
    } else if selected {
        (2.2, UI_AMBER)
    } else if hovered {
        (1.6, UI_CYAN_DIM)
    } else {
        (1.1, UI_EDGE)
    };
    draw_rectangle_lines(x, y, size, size, thick, edge);
}

pub fn tooltip(label: &str, mx: f32, my: f32) {
    let fs = 15.0;
    let tw = measure_text(label, None, fs as u16, 1.0).width;
    let pad_x = 10.0;
    let pad_y = 7.0;
    let w = tw + pad_x * 2.0;
    let h = fs + pad_y * 2.0;
    let x = (mx + 14.0).min(ui_width() - w - 8.0).max(8.0);
    let y = (my - h - 10.0).max(8.0);
    draw_rectangle(x + 2.0, y + 2.0, w, h, Color::from_rgba(0, 0, 0, 80));
    draw_rectangle(x, y, w, h, Color::from_rgba(12, 16, 22, 235));
    draw_rectangle(x, y, w, 2.0, UI_CYAN);
    draw_rectangle_lines(x, y, w, h, 1.0, UI_EDGE_BRIGHT);
    draw_text(label, x + pad_x, y + pad_y + fs * 0.78, fs, UI_TEXT);
}

pub fn chip(label: &str, x: f32, y: f32) {
    let fs = 13.0;
    let tw = measure_text(label, None, fs as u16, 1.0).width;
    let w = tw + 20.0;
    let h = 26.0;
    draw_rectangle(x, y, w, h, Color::from_rgba(14, 18, 24, 200));
    draw_rectangle_lines(x, y, w, h, 1.0, UI_EDGE);
    draw_rectangle(x, y, 3.0, h, UI_CYAN_DIM);
    draw_text(label, x + 12.0, y + 17.5, fs, UI_TEXT_DIM);
}

pub fn button(label: &str, x: f32, y: f32, w: f32, h: f32, _mouse: (f32, f32)) -> bool {
    button_styled(label, x, y, w, h, _mouse, ButtonStyle::Secondary)
}

pub fn button_styled(
    label: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    _mouse: (f32, f32),
    style: ButtonStyle,
) -> bool {
    // Always sample live pointer in draw-space — ignores stale/mismatched mouse args.
    let (mx, my) = pointer();
    let hovered = point_in(mx, my, x, y, w, h);
    let pressed = hovered && is_mouse_button_down(MouseButton::Left);
    let hover_t = anim_t(rect_key(x, y, w, h), if hovered { 1.0 } else { 0.0 });
    let press_t = if pressed { 1.0 } else { 0.0 };

    let (base_fill, hover_fill, edge_idle, edge_hot, text_c) = match style {
        ButtonStyle::Primary => (
            Color::from_rgba(24, 56, 52, 255),
            Color::from_rgba(34, 88, 80, 255),
            UI_CYAN_DIM,
            UI_CYAN,
            UI_TEXT,
        ),
        ButtonStyle::Danger => (
            Color::from_rgba(48, 24, 28, 255),
            Color::from_rgba(78, 34, 38, 255),
            Color::from_rgba(160, 80, 80, 200),
            UI_DANGER,
            UI_TEXT,
        ),
        ButtonStyle::Ghost => (
            Color::from_rgba(20, 24, 32, 120),
            Color::from_rgba(30, 36, 46, 200),
            UI_EDGE,
            UI_EDGE_BRIGHT,
            UI_TEXT_DIM,
        ),
        ButtonStyle::Secondary => (
            Color::from_rgba(28, 34, 46, 255),
            Color::from_rgba(40, 50, 66, 255),
            UI_EDGE_BRIGHT,
            UI_CYAN,
            UI_TEXT,
        ),
    };

    let fill = lerp_color(base_fill, hover_fill, hover_t);
    let edge = lerp_color(edge_idle, edge_hot, hover_t);
    let label_c = if matches!(style, ButtonStyle::Ghost) && hover_t < 0.5 {
        lerp_color(UI_TEXT_DIM, UI_TEXT, hover_t * 2.0)
    } else {
        text_c
    };

    // Draw inset so the full visual stays inside the hit rect (fixes “dead” bottom/edge).
    let inset = press_t * 1.5;
    let bx = x + inset;
    let by = y + inset;
    let bw = (w - inset * 2.0).max(1.0);
    let bh = (h - inset * 2.0).max(1.0);

    // Hover glow (inside hit rect)
    if hover_t > 0.01 {
        let g = (18.0 * hover_t) as u8;
        draw_rectangle(
            bx - 1.0,
            by - 1.0,
            bw + 2.0,
            bh + 2.0,
            Color::from_rgba(72, 220, 205, g),
        );
    }

    // Depth plate
    if press_t < 0.5 {
        draw_rectangle(
            bx + 1.0,
            by + 2.0,
            bw,
            bh,
            Color::from_rgba(0, 0, 0, (50.0 * (1.0 - press_t)) as u8),
        );
    }

    draw_rectangle(bx, by, bw, bh, fill);
    // Animated top sheen
    let sheen_a = (10.0 + 16.0 * hover_t) as u8;
    draw_rectangle(
        bx + 2.0,
        by + 2.0,
        bw - 4.0,
        bh * (0.28 + 0.08 * hover_t),
        Color::from_rgba(255, 255, 255, sheen_a),
    );
    draw_rectangle_lines(bx, by, bw, bh, 1.2 + 0.6 * hover_t, edge);

    // Leading accent bar grows on hover
    let accent_w = 2.0 + 2.0 * hover_t;
    if matches!(style, ButtonStyle::Primary) || hover_t > 0.05 {
        draw_rectangle(bx, by, accent_w, bh, edge);
    }

    let fs = 20.0_f32.min(h * 0.42);
    let tw = measure_text(label, None, fs as u16, 1.0).width;
    // Subtle text lift on hover
    let text_y = by + bh * 0.5 + fs * 0.32 - hover_t * 0.6;
    draw_text(label, bx + (bw - tw) * 0.5, text_y, fs, label_c);

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

pub fn text_field_frame(x: f32, y: f32, w: f32, h: f32, focused: bool) {
    draw_rectangle(x, y, w, h, UI_PANEL_INNER);
    draw_line(
        x + 1.0,
        y + 1.0,
        x + w - 1.0,
        y + 1.0,
        1.0,
        Color::from_rgba(255, 255, 255, 12),
    );
    draw_rectangle_lines(
        x,
        y,
        w,
        h,
        if focused { 1.8 } else { 1.2 },
        if focused { UI_CYAN } else { UI_EDGE },
    );
    if focused {
        draw_rectangle(x, y, 3.0, h, UI_CYAN);
    }
}

pub fn menu_title(title: &str, subtitle: &str, top: f32) {
    let tw = measure_text(title, None, 52, 1.0).width;
    let sx = (ui_width() - tw) * 0.5;
    draw_rectangle(
        sx - 20.0,
        top + 10.0,
        tw + 40.0,
        86.0,
        Color::from_rgba(8, 12, 18, 140),
    );
    draw_text(title, sx, top + 48.0, 52.0, UI_CYAN);
    draw_rectangle(sx, top + 56.0, tw * 0.35, 3.0, UI_AMBER);
    let sw = measure_text(subtitle, None, 18, 1.0).width;
    draw_text(
        subtitle,
        (ui_width() - sw) * 0.5,
        top + 82.0,
        18.0,
        UI_TEXT_DIM,
    );
}

/// Layout for a centered titled menu panel with a right-side category rail.
#[derive(Clone, Copy)]
pub struct MenuShellLayout {
    /// Inner area for page content (excludes footer strip).
    pub content: Rect,
    /// Bottom strip inside the panel for Apply / Back.
    pub footer: Rect,
    /// Top-left of the category rail (to the right of the panel).
    pub cat_origin: (f32, f32),
}

/// Draw page title above a centered panel. Category rail is laid out to the right.
/// `footer_h` reserves space at the bottom of the panel for action buttons.
pub fn menu_shell(title: &str, panel_w: f32, panel_h: f32, footer_h: f32) -> MenuShellLayout {
    let title_gap = 64.0;
    let cat_w = 140.0;
    let cat_gap = 16.0;
    let total_w = panel_w + cat_gap + cat_w;
    let total_h = title_gap + panel_h;
    let ox = (ui_width() - total_w) * 0.5;
    let oy = ((ui_height() - total_h) * 0.5).max(28.0);

    let tw = measure_text(title, None, 44, 1.0).width;
    let tx = ox + (panel_w - tw) * 0.5;
    // Soft title plate
    draw_rectangle(
        tx - 18.0,
        oy + 4.0,
        tw + 36.0,
        48.0,
        Color::from_rgba(8, 12, 18, 150),
    );
    draw_text(title, tx, oy + 38.0, 44.0, UI_CYAN);
    draw_rectangle(tx, oy + 46.0, tw * 0.32, 3.0, UI_AMBER);

    let panel_rect = Rect {
        x: ox,
        y: oy + title_gap,
        w: panel_w,
        h: panel_h,
    };
    panel(panel_rect.x, panel_rect.y, panel_rect.w, panel_rect.h);

    let pad = 22.0;
    let footer = Rect {
        x: panel_rect.x + pad,
        y: panel_rect.y + panel_rect.h - pad - footer_h,
        w: panel_rect.w - pad * 2.0,
        h: footer_h,
    };
    let content = Rect {
        x: panel_rect.x + pad,
        y: panel_rect.y + pad + 4.0,
        w: panel_rect.w - pad * 2.0,
        h: (footer.y - pad * 0.5) - (panel_rect.y + pad + 4.0),
    };

    MenuShellLayout {
        content,
        footer,
        cat_origin: (panel_rect.x + panel_rect.w + cat_gap, panel_rect.y + 4.0),
    }
}

/// Right-side category tabs. Returns the index clicked this frame, if any.
pub fn category_rail(
    labels: &[&str],
    active: usize,
    origin: (f32, f32),
    _mouse: (f32, f32),
) -> Option<usize> {
    let (mx, my) = pointer();
    let w = 140.0;
    let h = 44.0;
    let gap = 10.0;
    let mut clicked = None;

    // Sliding active indicator behind the active row
    let active_y = origin.1 + active as f32 * (h + gap);
    let slide_y = {
        thread_local! {
            static SLIDE_Y: RefCell<Option<f32>> = const { RefCell::new(None) };
        }
        let dt = get_frame_time().clamp(0.0, 0.05);
        SLIDE_Y.with(|cell| {
            let mut slot = cell.borrow_mut();
            let y = slot.get_or_insert(active_y);
            *y += (active_y - *y) * (1.0 - (-16.0 * dt).exp());
            *y
        })
    };
    draw_rectangle(
        origin.0 - 4.0,
        slide_y - 2.0,
        w + 8.0,
        h + 4.0,
        Color::from_rgba(72, 220, 205, 28),
    );

    for (i, label) in labels.iter().enumerate() {
        let y = origin.1 + i as f32 * (h + gap);
        let hovered = point_in(mx, my, origin.0, y, w, h);
        let is_active = i == active;
        let hover_t = anim_t(rect_key(origin.0, y, w, h), if hovered || is_active { 1.0 } else { 0.0 });

        let fill = if is_active {
            Color::from_rgba(32, 48, 54, 255)
        } else {
            lerp_color(
                Color::from_rgba(18, 22, 30, 230),
                Color::from_rgba(28, 36, 46, 255),
                hover_t,
            )
        };
        draw_rectangle(origin.0, y, w, h, fill);
        draw_rectangle_lines(
            origin.0,
            y,
            w,
            h,
            1.2 + 0.4 * hover_t,
            if is_active {
                UI_CYAN
            } else if hovered {
                UI_CYAN_DIM
            } else {
                UI_EDGE
            },
        );
        let accent = if is_active { 3.0 } else { 2.0 * hover_t };
        if accent > 0.05 {
            draw_rectangle(origin.0, y, accent, h, UI_CYAN);
        }
        draw_text(
            label,
            origin.0 + 16.0,
            y + h * 0.5 + 5.5,
            17.0,
            if is_active {
                UI_CYAN
            } else if hovered {
                UI_TEXT
            } else {
                UI_TEXT_DIM
            },
        );
        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            clicked = Some(i);
        }
    }
    clicked
}

pub fn sidebar_row(x: f32, y: f32, w: f32, h: f32, label: &str, active: bool, hovered: bool) {
    let fill = if active {
        Color::from_rgba(36, 52, 58, 255)
    } else if hovered {
        Color::from_rgba(30, 38, 48, 255)
    } else {
        Color::from_rgba(20, 25, 32, 255)
    };
    draw_rectangle(x, y, w, h, fill);
    if active {
        draw_rectangle(x, y, 3.0, h, UI_CYAN);
    } else if hovered {
        draw_rectangle(x, y, 2.0, h, UI_CYAN_DIM);
    }
    draw_text(
        label,
        x + 14.0,
        y + h * 0.5 + 5.5,
        16.0,
        if active {
            UI_CYAN
        } else if hovered {
            UI_TEXT
        } else {
            UI_TEXT_DIM
        },
    );
}

pub fn toast_bar(label: &str) {
    if label.is_empty() {
        return;
    }
    let fs = 17.0;
    let tw = measure_text(label, None, fs as u16, 1.0).width;
    let w = tw + 28.0;
    let h = 34.0;
    let x = 16.0;
    let y = ui_height() - 36.0 - h;
    draw_rectangle(x + 2.0, y + 2.0, w, h, Color::from_rgba(0, 0, 0, 70));
    draw_rectangle(x, y, w, h, Color::from_rgba(18, 24, 30, 230));
    draw_rectangle(x, y, 3.0, h, UI_AMBER);
    draw_rectangle_lines(x, y, w, h, 1.0, UI_EDGE);
    draw_text(label, x + 14.0, y + 22.5, fs, UI_AMBER);
}

/// Selectable save / list row. Returns true if clicked.
pub fn list_row(
    label: &str,
    meta: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    selected: bool,
    mouse: (f32, f32),
) -> bool {
    let (mx, my) = pointer();
    let _ = mouse;
    let hovered = point_in(mx, my, x, y, w, h);
    let fill = if selected {
        Color::from_rgba(48, 72, 70, 255)
    } else if hovered {
        Color::from_rgba(32, 40, 52, 255)
    } else {
        Color::from_rgba(20, 24, 32, 230)
    };
    draw_rectangle(x, y, w, h, fill);
    if selected {
        draw_rectangle(x, y, 3.0, h, UI_AMBER);
        draw_rectangle_lines(x, y, w, h, 1.4, UI_AMBER);
    } else {
        draw_rectangle_lines(x, y, w, h, 1.0, if hovered { UI_CYAN_DIM } else { UI_EDGE });
    }
    let fs = 16.0;
    draw_text(
        label,
        x + 12.0,
        y + h * 0.5 + 5.0,
        fs,
        if selected { UI_TEXT } else { UI_TEXT },
    );
    let mw = measure_text(meta, None, 13, 1.0).width;
    draw_text(meta, x + w - mw - 12.0, y + h * 0.5 + 4.5, 13.0, UI_TEXT_DIM);
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

pub fn stat_line(label: &str, value: &str, x: f32, y: f32, w: f32) {
    draw_text(label, x, y, 15.0, UI_TEXT_DIM);
    let vw = measure_text(value, None, 15, 1.0).width;
    draw_text(value, x + w - vw, y, 15.0, UI_TEXT);
}

/// Checkbox-style toggle row. Returns true if toggled this frame.
pub fn checkbox_row(label: &str, on: bool, x: f32, y: f32, w: f32, h: f32, mouse: (f32, f32)) -> bool {
    let (mx, my) = pointer();
    let _ = mouse;
    let hovered = point_in(mx, my, x, y, w, h);
    let fill = if hovered {
        Color::from_rgba(30, 38, 48, 255)
    } else {
        Color::from_rgba(22, 28, 36, 255)
    };
    draw_rectangle(x, y, w, h, fill);
    draw_rectangle_lines(x, y, w, h, 1.0, if hovered { UI_CYAN_DIM } else { UI_EDGE });

    let box_s = 18.0;
    let bx = x + 12.0;
    let by = y + (h - box_s) * 0.5;
    draw_rectangle(bx, by, box_s, box_s, Color::from_rgba(12, 16, 22, 255));
    draw_rectangle_lines(bx, by, box_s, box_s, 1.2, if on { UI_CYAN } else { UI_EDGE });
    if on {
        draw_rectangle(bx + 4.0, by + 4.0, box_s - 8.0, box_s - 8.0, UI_CYAN);
    }
    draw_text(label, bx + box_s + 12.0, y + h * 0.5 + 5.0, 16.0, UI_TEXT);
    hovered && is_mouse_button_pressed(MouseButton::Left)
}
