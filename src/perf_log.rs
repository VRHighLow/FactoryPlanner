//! Append-only performance diagnostics for player support.
//!
//! Written next to settings as `userdata/perf_log.txt` so users can send the
//! file when reporting low FPS. Simulation UPS is logged separately from render FPS.

use crate::save::{EffectQuality, FpsLimit, Settings};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_LOG_BYTES: u64 = 1_500_000;
const KEEP_TAIL_BYTES: u64 = 512_000;
const SAMPLE_EVERY: Duration = Duration::from_secs(2);
const DIP_COOLDOWN: Duration = Duration::from_secs(5);

pub fn log_path() -> PathBuf {
    PathBuf::from("userdata").join("perf_log.txt")
}

pub struct PerfLog {
    file: Option<File>,
    session_start: Instant,
    last_sample: Instant,
    last_dip: Instant,
    samples: u32,
    fps_sum: f64,
    fps_min: f32,
    fps_max: f32,
    frame_ms_sum: f64,
    frame_ms_max: f32,
    dip_count: u32,
    rolling_fps: f32,
    screen: String,
    finished: bool,
}

impl PerfLog {
    pub fn start(settings: &Settings) -> Self {
        let _ = fs::create_dir_all("userdata");
        rotate_if_huge();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path())
            .ok();

        let mut log = Self {
            file: None,
            session_start: Instant::now(),
            last_sample: Instant::now(),
            last_dip: Instant::now()
                .checked_sub(DIP_COOLDOWN)
                .unwrap_or_else(Instant::now),
            samples: 0,
            fps_sum: 0.0,
            fps_min: f32::MAX,
            fps_max: 0.0,
            frame_ms_sum: 0.0,
            frame_ms_max: 0.0,
            dip_count: 0,
            rolling_fps: 0.0,
            screen: "boot".into(),
            finished: false,
        };

        if let Some(f) = file.as_mut() {
            let _ = writeln!(f);
            let _ = writeln!(f, "========== SESSION START {} ==========", utc_stamp());
            let _ = writeln!(
                f,
                "version={}  os={}  arch={}",
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS,
                std::env::consts::ARCH
            );
            let _ = writeln!(
                f,
                "settings: display={}  vsync={}  fps_limit={}  effect_quality={}  window={}x{}  show_fps={}",
                settings.display_mode.label(),
                settings.vsync,
                settings.fps_limit.label(),
                settings.effect_quality.label(),
                settings.window_w,
                settings.window_h,
                settings.show_fps
            );
            let _ = writeln!(
                f,
                "note: player/factory speed uses fixed 60 UPS; FPS is render-only"
            );
            let _ = writeln!(
                f,
                "columns: time_s | screen | fps | avg_fps | frame_ms | ups | nodes | belts | peers | note"
            );
            let _ = f.flush();
        }
        log.file = file;
        log
    }

    pub fn set_screen(&mut self, screen: &str) {
        if self.screen != screen {
            self.write_line(&format!(
                "{:>7.1} | {:<12} | screen -> {}",
                self.session_start.elapsed().as_secs_f32(),
                self.screen,
                screen
            ));
            self.screen = screen.to_string();
        }
    }

    pub fn note(&mut self, msg: &str) {
        self.write_line(&format!(
            "{:>7.1} | {:<12} | NOTE {}",
            self.session_start.elapsed().as_secs_f32(),
            self.screen,
            msg
        ));
    }

    /// Call once per rendered frame with measured frame time and live counters.
    pub fn frame(
        &mut self,
        frame_ms: f32,
        fps: f32,
        ups: f32,
        nodes: usize,
        belts: usize,
        peers: usize,
        effect_quality: EffectQuality,
        fps_limit: FpsLimit,
    ) {
        let now = Instant::now();
        if now.duration_since(self.last_sample) < SAMPLE_EVERY {
            // Still catch severe hitches between samples.
            if frame_ms >= 50.0 && now.duration_since(self.last_dip) >= DIP_COOLDOWN {
                self.last_dip = now;
                self.dip_count += 1;
                self.write_sample(
                    fps,
                    ups,
                    frame_ms,
                    nodes,
                    belts,
                    peers,
                    &format!("HITCH frame_ms={frame_ms:.1}"),
                );
            }
            return;
        }
        self.last_sample = now;

        let fps = fps.max(0.0);
        let frame_ms = frame_ms.max(0.0);
        self.samples += 1;
        self.fps_sum += fps as f64;
        self.frame_ms_sum += frame_ms as f64;
        self.fps_min = self.fps_min.min(fps);
        self.fps_max = self.fps_max.max(fps);
        self.frame_ms_max = self.frame_ms_max.max(frame_ms);

        let avg = if self.samples > 0 {
            (self.fps_sum / self.samples as f64) as f32
        } else {
            fps
        };
        // EMA so short dips compare against recent average, not the whole session.
        self.rolling_fps = if self.rolling_fps <= 1.0 {
            fps
        } else {
            self.rolling_fps * 0.85 + fps * 0.15
        };

        let mut note = String::new();
        let target = match fps_limit {
            FpsLimit::Unlimited => self.rolling_fps.max(60.0),
            FpsLimit::Fps30 => 30.0,
            FpsLimit::Fps60 => 60.0,
            FpsLimit::Fps120 => 120.0,
            FpsLimit::Fps144 => 144.0,
            FpsLimit::Fps240 => 240.0,
        };

        if fps < 25.0 {
            note = format!("CRITICAL fps<{fps:.0} (eq={})", effect_quality.label());
        } else if fps < 40.0 {
            note = format!("LOW fps (eq={})", effect_quality.label());
        } else if self.rolling_fps > 50.0 && fps < self.rolling_fps * 0.55 {
            note = format!(
                "DIP {:.0}% of rolling {:.0}",
                (fps / self.rolling_fps) * 100.0,
                self.rolling_fps
            );
        } else if fps + 8.0 < target * 0.7 && !matches!(fps_limit, FpsLimit::Unlimited) {
            note = format!("below limit target ~{target:.0}");
        }

        if !note.is_empty() {
            if now.duration_since(self.last_dip) >= DIP_COOLDOWN {
                self.last_dip = now;
                self.dip_count += 1;
                self.write_sample(fps, ups, frame_ms, nodes, belts, peers, &note);
            }
        } else {
            // Periodic heartbeat so healthy sessions still leave a trail.
            self.write_sample(fps, ups, frame_ms, nodes, belts, peers, "ok");
        }

        let _ = avg;
    }

    pub fn finish(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        let elapsed = self.session_start.elapsed().as_secs_f32();
        let avg_fps = if self.samples > 0 {
            self.fps_sum / self.samples as f64
        } else {
            0.0
        };
        let avg_ms = if self.samples > 0 {
            self.frame_ms_sum / self.samples as f64
        } else {
            0.0
        };
        let min_fps = if self.fps_min.is_finite() && self.fps_min < f32::MAX / 2.0 {
            self.fps_min
        } else {
            0.0
        };
        self.write_line(&format!(
            "---------- SESSION END {} ({elapsed:.0}s) ----------",
            utc_stamp()
        ));
        self.write_line(&format!(
            "summary: samples={}  avg_fps={avg_fps:.1}  min_fps={min_fps:.1}  max_fps={:.1}  avg_frame_ms={avg_ms:.2}  max_frame_ms={:.1}  dips={}",
            self.samples, self.fps_max, self.frame_ms_max, self.dip_count
        ));
        if let Some(f) = self.file.as_mut() {
            let _ = f.flush();
        }
    }

    fn write_sample(
        &mut self,
        fps: f32,
        ups: f32,
        frame_ms: f32,
        nodes: usize,
        belts: usize,
        peers: usize,
        note: &str,
    ) {
        let avg = if self.samples > 0 {
            self.fps_sum / self.samples as f64
        } else {
            fps as f64
        };
        self.write_line(&format!(
            "{:>7.1} | {:<12} | {:>5.1} | {:>6.1} | {:>7.2} | {:>4.0} | {:>5} | {:>5} | {:>5} | {}",
            self.session_start.elapsed().as_secs_f32(),
            truncate(&self.screen, 12),
            fps,
            avg,
            frame_ms,
            ups,
            nodes,
            belts,
            peers,
            note
        ));
    }

    fn write_line(&mut self, line: &str) {
        if let Some(f) = self.file.as_mut() {
            let _ = writeln!(f, "{line}");
            let _ = f.flush();
        }
    }
}

/// Best-effort line when the process exits without dropping `PerfLog` (e.g. Exit button).
pub fn append_shutdown_note(msg: &str) {
    let _ = fs::create_dir_all("userdata");
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
    {
        let _ = writeln!(
            f,
            "---------- SESSION END {} ({} ) ----------",
            utc_stamp(),
            msg
        );
        let _ = f.flush();
    }
}

impl Drop for PerfLog {
    fn drop(&mut self) {
        self.finish();
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

fn utc_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Keep it simple / no chrono dep: unix seconds + local note.
    format!("unix={secs}")
}

fn rotate_if_huge() {
    let path = log_path();
    let Ok(meta) = fs::metadata(&path) else {
        return;
    };
    if meta.len() <= MAX_LOG_BYTES {
        return;
    }
    let Ok(mut f) = File::open(&path) else {
        return;
    };
    let start = meta.len().saturating_sub(KEEP_TAIL_BYTES);
    if f.seek(SeekFrom::Start(start)).is_err() {
        return;
    }
    let mut buf = Vec::new();
    if f.read_to_end(&mut buf).is_err() {
        return;
    };
    // Drop partial first line.
    if let Some(i) = buf.iter().position(|&b| b == b'\n') {
        buf = buf[i + 1..].to_vec();
    }
    let mut out = Vec::new();
    out.extend_from_slice(b"[log truncated to recent tail]\n");
    out.extend_from_slice(&buf);
    let _ = fs::write(&path, out);
}
