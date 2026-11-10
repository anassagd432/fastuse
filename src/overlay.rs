//! What the user sees while the agent works: its own cursor, a colored glow around the screen edge
//! and a small "controlled by fastuse" banner at the top. Shared by every platform.
//!
//! This file owns the look (pure pixel math, no OS calls) and the motion (the glide, the click
//! ripple, fade in/out, the slow breathing of the glow). A platform only supplies a `Surface`:
//! a few primitives to show pixels in click-through, always-on-top windows.
//!
//! FASTUSE_CURSOR=0       disable the whole overlay
//! FASTUSE_GLOW=0         no screen-edge glow and banner, cursor only
//! FASTUSE_COLOR=#RRGGBB  brand color (default blue #2F6BFF)
//! FASTUSE_BANNER=text    banner text ("0" hides the banner)
//! FASTUSE_RECORDABLE=1   let screen captures see the glow and banner (default: hidden from them,
//!                        so the model's screenshots show the real screen); use it to record demos
//! FASTUSE_CURSOR_SYNC=1  make each action wait (max ~0.8 s) until the cursor arrives, so every
//!                        stop is visible; default is async so the overlay never slows an action

use std::sync::{mpsc, OnceLock};
use std::time::{Duration, Instant};

/// Logical size of the cursor sprite canvas and the offset of the arrow tip inside it.
pub const CANVAS: f32 = 64.0;
pub const TIP: f32 = 20.0;
pub const RIPPLE_FRAMES: usize = 7;
const K: f32 = 1.12; // arrow scale (logical): about the size of the macOS pointer
const CURSOR_IDLE: Duration = Duration::from_secs(8);
const DEFAULT_FRAME_IDLE: Duration = Duration::from_secs(30);
const DEFAULT_COLOR: (f32, f32, f32) = (47.0, 107.0, 255.0);

pub fn parse_hex(s: &str) -> Option<(f32, f32, f32)> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(s, 16).ok()?;
    Some((((n >> 16) & 255) as f32, ((n >> 8) & 255) as f32, (n & 255) as f32))
}

pub fn color() -> (f32, f32, f32) {
    static C: OnceLock<(f32, f32, f32)> = OnceLock::new();
    *C.get_or_init(|| std::env::var("FASTUSE_COLOR").ok().and_then(|v| parse_hex(&v)).unwrap_or(DEFAULT_COLOR))
}

fn env_off(name: &str) -> bool {
    std::env::var(name).map(|v| v == "0").unwrap_or(false)
}

pub fn banner_text() -> Option<String> {
    match std::env::var("FASTUSE_BANNER") {
        Ok(v) if v == "0" => None,
        Ok(v) if !v.trim().is_empty() => Some(v),
        _ => Some("This computer is being controlled by fastuse".into()),
    }
}

/// Banner size in pixels for `text` at scale `s` (platforms with real text metrics may measure instead).
#[cfg_attr(not(any(windows, target_os = "macos")), allow(dead_code))]
pub fn banner_size(text: &str, s: f32) -> (i32, i32) {
    let w = (text.chars().count() as f32 * 7.4 + 52.0) * s;
    (w.ceil() as i32, (34.0 * s).ceil() as i32)
}

// ---------------------------------------------------------------------------------------------
// Pixels. All buffers are premultiplied 0xAARRGGBB, row-major, top-left origin.
// ---------------------------------------------------------------------------------------------

/// Premultiplied "over": paint `c` with coverage `a` on top of `dst` (r, g, b, a all 0..255 premultiplied).
fn over(dst: &mut [f32; 4], c: (f32, f32, f32), a: f32) {
    let k = 1.0 - a;
    dst[0] = c.0 * a + dst[0] * k;
    dst[1] = c.1 * a + dst[1] * k;
    dst[2] = c.2 * a + dst[2] * k;
    dst[3] = 255.0 * a + dst[3] * k;
}

fn pack(p: &[f32; 4]) -> u32 {
    (p[3].round() as u32) << 24 | (p[0].round() as u32) << 16 | (p[1].round() as u32) << 8 | p[2].round() as u32
}

fn mix(a: (f32, f32, f32), b: (f32, f32, f32), t: f32) -> (f32, f32, f32) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, a.2 + (b.2 - a.2) * t)
}

/// 0xAARRGGBB pixels -> R,G,B,A bytes (what CoreGraphics / AppKit bitmaps expect).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn to_rgba_bytes(px: &[u32]) -> Vec<u8> {
    let mut v = Vec::with_capacity(px.len() * 4);
    for p in px {
        v.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, *p as u8, (p >> 24) as u8]);
    }
    v
}

const ARROW: [(f32, f32); 7] = [(0.0, 0.0), (0.0, 17.0), (4.0, 13.0), (7.0, 20.0), (10.0, 19.0), (7.0, 12.0), (12.0, 12.0)];

fn pts(s: f32) -> [(f32, f32); 7] {
    let mut o = [(0.0, 0.0); 7];
    for (i, p) in ARROW.iter().enumerate() {
        o[i] = (TIP * s + p.0 * K * s, TIP * s + p.1 * K * s);
    }
    o
}

fn inside(poly: &[(f32, f32); 7], px: f32, py: f32) -> bool {
    let mut c = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        if (yi > py) != (yj > py) && px < (xj - xi) * (py - yi) / (yj - yi) + xi {
            c = !c;
        }
        j = i;
    }
    c
}

fn edge_dist(poly: &[(f32, f32); 7], px: f32, py: f32) -> f32 {
    let mut best = f32::MAX;
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let t = (((px - a.0) * dx + (py - a.1) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        let (qx, qy) = (a.0 + t * dx, a.1 + t * dy);
        best = best.min(((px - qx).powi(2) + (py - qy).powi(2)).sqrt());
    }
    best
}

/// The cursor sprite, `size`x`size` pixels at scale `s` (pixels per logical unit): a macOS-style
/// arrow with a graded brand fill, white outline, a soft brand-colored halo and a drop shadow.
/// `ring` = (radius in logical units, alpha) adds a click ripple around the tip.
pub fn render_cursor(size: i32, s: f32, ring: Option<(f32, f32)>) -> Vec<u32> {
    let col = color();
    let top = mix(col, (255.0, 255.0, 255.0), 0.15);
    let bottom = (col.0 * 0.93, col.1 * 0.93, col.2 * 0.93);
    let poly = pts(s);
    let rf = 0.8 * s; // grows the fill a little, which rounds the corners
    let ow = 1.15 * s; // white outline width
    let halo = 6.5 * s; // brand-colored glow around the arrow
    let tip = TIP * s;
    let height = 20.0 * K * s;
    let (shx, shy, sblur) = (0.8 * s, 1.7 * s, 3.2 * s);
    let sdf = |px: f32, py: f32| {
        let d = edge_dist(&poly, px, py);
        if inside(&poly, px, py) {
            -d
        } else {
            d
        }
    };
    let mut out = vec![0u32; (size * size) as usize];
    for y in 0..size {
        for x in 0..size {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let mut p = [0f32; 4];
            let d = sdf(px, py) - rf;
            // soft drop shadow
            let sd = sdf(px - shx, py - shy) - rf;
            let t = (sd / sblur).clamp(0.0, 1.0);
            over(&mut p, (0.0, 0.0, 0.0), 0.28 * (1.0 - t) * (1.0 - t));
            // brand halo
            if d > 0.0 {
                let t = (d / halo).clamp(0.0, 1.0);
                over(&mut p, col, 0.60 * (1.0 - t) * (1.0 - t));
            }
            // click ripple
            if let Some((r, a)) = ring {
                let rr = ((px - tip).powi(2) + (py - tip).powi(2)).sqrt();
                let e = (1.0 - ((rr - r * s).abs() / (1.6 * s)).min(1.0)).max(0.0);
                if e > 0.0 {
                    over(&mut p, col, a * e);
                }
            }
            // white outline, then the graded fill, both analytically anti-aliased
            over(&mut p, (255.0, 255.0, 255.0), (0.5 - (d - ow)).clamp(0.0, 1.0));
            let g = ((py - tip) / height).clamp(0.0, 1.0);
            over(&mut p, mix(top, bottom, g), (0.5 - d).clamp(0.0, 1.0));
            out[(y * size + x) as usize] = pack(&p);
        }
    }
    out
}

/// The screen-edge glow for a `w`x`h` pixel screen: a crisp light stroke on the very edge and a
/// soft brand-colored falloff towards the middle (rounded corners). The middle stays transparent.
pub fn render_glow(w: i32, h: i32, s: f32) -> Vec<u32> {
    let col = color();
    let stroke_col = mix(col, (255.0, 255.0, 255.0), 0.35);
    let band = (48.0 * s).ceil() as i32; // how far the glow reaches into the screen
    let sw = 2.4 * s; // stroke width
    let radius = 14.0 * s;
    let (wf, hf) = (w as f32, h as f32);
    // alpha profile by depth from the edge, one entry per 1/4 pixel
    let lut: Vec<[f32; 4]> = (0..=(band * 4))
        .map(|i| {
            let d = i as f32 / 4.0;
            let mut p = [0f32; 4];
            let t = (d / band as f32).clamp(0.0, 1.0);
            over(&mut p, col, 0.62 * (1.0 - t).powf(2.4));
            over(&mut p, stroke_col, (sw - d + 0.5).clamp(0.0, 1.0) * 0.95);
            p
        })
        .collect();
    let depth = |x: i32, y: i32| -> f32 {
        let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
        let (dx, dy) = (px.min(wf - px), py.min(hf - py));
        if dx < radius && dy < radius {
            (radius - ((radius - dx).powi(2) + (radius - dy).powi(2)).sqrt()).max(0.0)
        } else {
            dx.min(dy)
        }
    };
    let mut out = vec![0u32; (w * h) as usize];
    let mut put = |x: i32, y: i32| {
        let d = depth(x, y);
        if d < band as f32 {
            let p = lut[((d * 4.0) as usize).min(lut.len() - 1)];
            out[(y * w + x) as usize] = pack(&p);
        }
    };
    for y in 0..h {
        if y < band || y >= h - band {
            for x in 0..w {
                put(x, y);
            }
        } else {
            for x in 0..band.min(w) {
                put(x, y);
            }
            for x in (w - band).max(band)..w {
                put(x, y);
            }
        }
    }
    out
}

/// The banner pill: brand-colored, fully rounded, with a faint light border. Text is composited
/// on top by the platform (`composite_text`).
#[cfg_attr(not(windows), allow(dead_code))]
pub fn render_pill(w: i32, h: i32, s: f32) -> Vec<u32> {
    let col = color();
    let fill = (col.0 * 0.62, col.1 * 0.62, col.2 * 0.62);
    let border = mix(col, (255.0, 255.0, 255.0), 0.45);
    let r = h as f32 / 2.0;
    let mut out = vec![0u32; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            // distance to the capsule's core segment, minus its radius: negative inside
            let cx = px.clamp(r, w as f32 - r);
            let d = ((px - cx).powi(2) + (py - r).powi(2)).sqrt() - r;
            let mut p = [0f32; 4];
            over(&mut p, border, (0.5 - d).clamp(0.0, 1.0) * 0.9);
            over(&mut p, fill, (0.5 - (d + 1.2 * s)).clamp(0.0, 1.0) * 0.94);
            out[(y * w + x) as usize] = pack(&p);
        }
    }
    out
}

/// Paint white text on the pill from a coverage mask (0..255 per pixel, same size as the pill).
#[cfg_attr(not(windows), allow(dead_code))]
pub fn composite_text(pill: &mut [u32], coverage: &[u8]) {
    for (px, cov) in pill.iter_mut().zip(coverage) {
        if *cov == 0 {
            continue;
        }
        let mut p = [((*px >> 16) & 255) as f32, ((*px >> 8) & 255) as f32, (*px & 255) as f32, (*px >> 24) as f32];
        over(&mut p, (255.0, 255.0, 255.0), *cov as f32 / 255.0);
        *px = pack(&p);
    }
}

// ---------------------------------------------------------------------------------------------
// Motion
// ---------------------------------------------------------------------------------------------

/// What a platform has to provide: click-through, always-on-top, non-activating windows.
/// All calls happen on the overlay thread. Cursor coordinates are screen coordinates in the
/// platform's own unit (pixels on Windows, points on macOS); `scale()` converts logical units.
pub trait Surface {
    fn scale(&self) -> f32;
    /// Show sprite `frame` (0 = plain arrow, 1..=RIPPLE_FRAMES = ripple) at opacity 0..255.
    fn cursor_frame(&mut self, frame: usize, alpha: u8);
    /// Put the arrow tip at (x, y).
    fn cursor_place(&mut self, x: f32, y: f32);
    fn cursor_visible(&mut self, on: bool);
    /// Show or hide the screen-edge glow and the banner.
    fn frame_visible(&mut self, on: bool);
    /// Opacity 0..1 of the glow and of the banner.
    fn frame_alpha(&mut self, glow: f32, banner: f32);
    /// Let the OS deliver window messages (Windows needs this; elsewhere nothing to do).
    fn pump(&mut self) {}
}

enum Msg {
    Glide {
        x: i32,
        y: i32,
        click: bool,
        ack: Option<mpsc::Sender<()>>,
    },
    Active(bool),
}

static TX: OnceLock<mpsc::Sender<Msg>> = OnceLock::new();
static SYNC: OnceLock<bool> = OnceLock::new();

/// Start the overlay thread. `make` builds the platform surface on that thread.
pub fn init<S, F>(make: F)
where
    S: Surface,
    F: FnOnce() -> Option<S> + Send + 'static,
{
    if env_off("FASTUSE_CURSOR") {
        return;
    }
    let _ = SYNC.set(std::env::var("FASTUSE_CURSOR_SYNC").map(|v| v == "1").unwrap_or(false));
    let (tx, rx) = mpsc::channel();
    if TX.set(tx).is_ok() {
        std::thread::spawn(move || {
            if let Some(s) = make() {
                run(rx, s);
            }
        });
    }
}

/// Glide the agent cursor to screen point (x, y); `click` adds a ripple. In sync mode this waits
/// until the cursor has arrived (bounded), otherwise it returns immediately.
pub fn go(x: i32, y: i32, click: bool) {
    let Some(tx) = TX.get() else { return };
    if SYNC.get().copied().unwrap_or(false) {
        let (atx, arx) = mpsc::channel();
        if tx.send(Msg::Glide { x, y, click, ack: Some(atx) }).is_ok() {
            let _ = arx.recv_timeout(Duration::from_millis(800));
        }
    } else {
        let _ = tx.send(Msg::Glide { x, y, click, ack: None });
    }
}

/// Keep the screen-edge glow and banner alive while fastuse is actively executing any action.
pub fn set_active(active: bool) {
    if let Some(tx) = TX.get() {
        let _ = tx.send(Msg::Active(active));
    }
}

fn frame_idle() -> Duration {
    std::env::var("FASTUSE_FRAME_IDLE")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_FRAME_IDLE)
}

/// True when the overlay thread was started (it is skipped when FASTUSE_CURSOR=0).
pub fn enabled() -> bool {
    TX.get().is_some()
}

/// Opacity of (glow, banner) `since` the frame appeared and `t` seconds into the run:
/// a quick fade in, then the glow slowly breathes.
fn frame_levels(since: Duration, t: f32) -> (f32, f32) {
    let f = (since.as_secs_f32() / 0.35).min(1.0);
    let fade = f * f * (3.0 - 2.0 * f);
    let breath = 0.84 + 0.16 * (t * std::f32::consts::TAU / 2.8).sin();
    (fade * breath, fade)
}

fn run<S: Surface>(rx: mpsc::Receiver<Msg>, mut sf: S) {
    let sc = sf.scale();
    let glow = !env_off("FASTUSE_GLOW");
    let t0 = Instant::now();
    let mut pos = (0f32, 0f32);
    let mut cursor_on = false;
    let mut frame_on = false;
    let mut frame_since = Instant::now();
    let mut last = Instant::now();
    let mut last_frame = Instant::now();
    let mut active = false;

    macro_rules! tick_frame {
        () => {
            if frame_on {
                let (g, b) = frame_levels(frame_since.elapsed(), t0.elapsed().as_secs_f32());
                sf.frame_alpha(g, b);
                last_frame = Instant::now();
            }
        };
    }

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(first) => {
                let mut current = Some(first);
                while let Some(msg) = current.take() {
                    match msg {
                        Msg::Active(on) => {
                            active = on;
                            last = Instant::now();
                            if on && glow && !frame_on {
                                frame_since = Instant::now();
                                frame_on = true;
                                sf.frame_alpha(0.0, 0.0);
                                sf.frame_visible(true);
                            }
                            tick_frame!();
                        }
                        Msg::Glide { x, y, click, ack } => {
                            let (mut tx_, mut ty_, mut click_) = (x, y, click);
                            let mut acks: Vec<mpsc::Sender<()>> = ack.into_iter().collect();
                            while let Ok(next) = rx.try_recv() {
                                match next {
                                    Msg::Glide { x: nx, y: ny, click: nc, ack: na } => {
                                        tx_ = nx;
                                        ty_ = ny;
                                        click_ |= nc;
                                        acks.extend(na);
                                    }
                                    Msg::Active(on) => {
                                        current = Some(Msg::Active(on));
                                        break;
                                    }
                                }
                            }
                            let (tx_, ty_) = (tx_ as f32, ty_ as f32);
                            if glow && !frame_on {
                                frame_since = Instant::now();
                                frame_on = true;
                                sf.frame_alpha(0.0, 0.0);
                                sf.frame_visible(true);
                            }
                            if !cursor_on {
                                pos = (tx_ - 90.0 * sc, ty_ - 60.0 * sc); // glide in from up-left
                                sf.cursor_frame(0, 255);
                                sf.cursor_place(pos.0, pos.1);
                                sf.cursor_visible(true);
                                cursor_on = true;
                            } else {
                                sf.cursor_frame(0, 255); // undo a fade in progress
                            }
                            // curved glide: quadratic bezier bowed sideways, eased
                            let (dx, dy) = (tx_ - pos.0, ty_ - pos.1);
                            let dist = (dx * dx + dy * dy).sqrt();
                            let ms = (160.0 + dist / sc * 0.45).clamp(180.0, 420.0);
                            let steps = (ms / 14.0) as i32;
                            let (cx, cy) = ((pos.0 + tx_) / 2.0 - dy * 0.18, (pos.1 + ty_) / 2.0 + dx * 0.18);
                            for i in 1..=steps {
                                let t = i as f32 / steps as f32;
                                let e = t * t * t * (t * (t * 6.0 - 15.0) + 10.0); // smootherstep: soft start and stop
                                let u = 1.0 - e;
                                sf.cursor_place(u * u * pos.0 + 2.0 * u * e * cx + e * e * tx_, u * u * pos.1 + 2.0 * u * e * cy + e * e * ty_);
                                if last_frame.elapsed() >= Duration::from_millis(50) {
                                    tick_frame!();
                                }
                                std::thread::sleep(Duration::from_millis(14));
                            }
                            sf.cursor_place(tx_, ty_);
                            pos = (tx_, ty_);
                            for a in acks {
                                let _ = a.send(());
                            }
                            if click_ {
                                for f in 1..=RIPPLE_FRAMES {
                                    sf.cursor_frame(f, 255);
                                    tick_frame!();
                                    std::thread::sleep(Duration::from_millis(28));
                                }
                                sf.cursor_frame(0, 255);
                            }
                            tick_frame!();
                            last = Instant::now();
                        }
                    }
                    if current.is_none() {
                        if let Ok(n) = rx.try_recv() {
                            current = Some(n);
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if active {
                    last = Instant::now();
                    tick_frame!();
                } else {
                    if cursor_on && last.elapsed() > CURSOR_IDLE {
                        for a in (0..=8).rev() {
                            sf.cursor_frame(0, (a * 31) as u8);
                            std::thread::sleep(Duration::from_millis(30));
                        }
                        sf.cursor_visible(false);
                        cursor_on = false;
                    }
                    if frame_on && last.elapsed() > frame_idle() {
                        let (g, b) = frame_levels(frame_since.elapsed(), t0.elapsed().as_secs_f32());
                        for a in (0..=10).rev() {
                            let k = a as f32 / 10.0;
                            sf.frame_alpha(g * k, b * k);
                            std::thread::sleep(Duration::from_millis(30));
                        }
                        sf.frame_visible(false);
                        frame_on = false;
                    } else {
                        tick_frame!();
                    }
                }
            }
            Err(_) => return,
        }
        sf.pump();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha(p: u32) -> u32 {
        p >> 24
    }

    #[test]
    fn hex_colors() {
        assert_eq!(parse_hex("#2F6BFF"), Some((47.0, 107.0, 255.0)));
        assert_eq!(parse_hex("ff7a00"), Some((255.0, 122.0, 0.0)));
        assert_eq!(parse_hex("blue"), None);
        assert_eq!(parse_hex("#fff"), None);
    }

    #[test]
    fn cursor_is_visible_and_premultiplied() {
        let s = 2.0;
        let size = (CANVAS * s) as i32;
        let px = render_cursor(size, s, None);
        let tip = (TIP * s) as i32;
        // a point just inside the arrow is opaque, the far corner is empty
        assert!(alpha(px[((tip + 10) * size + tip + 4) as usize]) > 240);
        assert_eq!(alpha(px[0]), 0);
        // premultiplied: no channel may exceed alpha
        for p in &px {
            let a = alpha(*p);
            assert!(((p >> 16) & 255) <= a + 1 && ((p >> 8) & 255) <= a + 1 && (p & 255) <= a + 1);
        }
        // the ripple frame draws something the plain one does not
        assert_ne!(px, render_cursor(size, s, Some((10.0, 0.7))));
    }

    #[test]
    fn glow_hugs_the_edge_and_leaves_the_middle_clear() {
        let (w, h) = (400, 300);
        let g = render_glow(w, h, 1.0);
        let at = |x: i32, y: i32| alpha(g[(y * w + x) as usize]);
        assert!(at(200, 0) > 200); // crisp stroke on the edge
        assert!(at(200, 20) > 0 && at(200, 20) < at(200, 2)); // falls off inwards
        assert_eq!(at(200, 150), 0); // center is untouched
        assert_eq!(at(w / 2, h - 1), at(w / 2, 0)); // symmetric
    }

    #[test]
    fn pill_is_rounded() {
        let (w, h) = banner_size("hello", 1.0);
        let p = render_pill(w, h, 1.0);
        assert_eq!(alpha(p[0]), 0); // corner is cut
        assert!(alpha(p[((h / 2) * w + w / 2) as usize]) > 200);
    }

    #[test]
    fn rgba_bytes_order() {
        assert_eq!(to_rgba_bytes(&[0x80_10_20_30]), vec![0x10, 0x20, 0x30, 0x80]);
    }

    #[test]
    fn frame_idle_default_and_env() {
        assert_eq!(DEFAULT_FRAME_IDLE, Duration::from_secs(30));
        assert_eq!(frame_idle(), Duration::from_secs(30));
    }
}
