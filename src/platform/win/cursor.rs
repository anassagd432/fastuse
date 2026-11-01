//! The agent's own cursor: a click-through, topmost, non-activating layered window that glides to
//! wherever the agent acts, with a soft glow and a click ripple. It never touches the real mouse
//! pointer or keyboard focus. The look follows the idea of cua-driver's agent cursor (MIT):
//! a colored arrow with a white outline and glow, DPI-scaled, visible for the whole task.
//!
//! FASTUSE_CURSOR=0       disable it
//! FASTUSE_CURSOR_SYNC=1  make each action wait (max ~0.8 s) until the cursor arrives, so every
//!                        stop is visible; default is async so the cursor never slows an action

use std::ffi::c_void;
use std::sync::{mpsc, OnceLock};
use std::time::{Duration, Instant};

use windows::core::w;
use windows::Win32::Foundation::{COLORREF, POINT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, GetDC, SelectObject, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HDC,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::WindowsAndMessaging::*;

struct Msg {
    x: i32,
    y: i32,
    click: bool,
    ack: Option<mpsc::Sender<()>>,
}

static TX: OnceLock<mpsc::Sender<Msg>> = OnceLock::new();
static SYNC: OnceLock<bool> = OnceLock::new();

const IDLE_FADE: Duration = Duration::from_secs(8);
const CANVAS: f32 = 96.0; // logical size of the sprite canvas
const TIP: f32 = 36.0; // logical offset of the arrow tip inside the canvas
const K: f32 = 1.7; // arrow scale (logical)
const RIPPLE_FRAMES: usize = 7;
const COLOR: (f32, f32, f32) = (47.0, 107.0, 255.0);

pub fn init() {
    if std::env::var("FASTUSE_CURSOR").map(|v| v == "0").unwrap_or(false) {
        return;
    }
    let _ = SYNC.set(std::env::var("FASTUSE_CURSOR_SYNC").map(|v| v == "1").unwrap_or(false));
    let (tx, rx) = mpsc::channel();
    if TX.set(tx).is_ok() {
        std::thread::spawn(move || run(rx));
    }
}

/// Glide the agent cursor to screen point (x, y); `click` adds a ripple. In sync mode this waits
/// until the cursor has arrived (bounded), otherwise it returns immediately.
pub fn go(x: i32, y: i32, click: bool) {
    let Some(tx) = TX.get() else { return };
    if SYNC.get().copied().unwrap_or(false) {
        let (atx, arx) = mpsc::channel();
        if tx.send(Msg { x, y, click, ack: Some(atx) }).is_ok() {
            let _ = arx.recv_timeout(Duration::from_millis(800));
        }
    } else {
        let _ = tx.send(Msg { x, y, click, ack: None });
    }
}

/// True when the overlay thread is running (it is skipped when FASTUSE_CURSOR=0).
pub fn enabled() -> bool {
    TX.get().is_some()
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

/// Premultiplied-alpha "over": paint `color` with coverage `a` on top of `dst` (r,g,b,a premultiplied).
fn over(dst: &mut [f32; 4], c: (f32, f32, f32), a: f32) {
    let k = 1.0 - a;
    dst[0] = c.0 * a + dst[0] * k;
    dst[1] = c.1 * a + dst[1] * k;
    dst[2] = c.2 * a + dst[2] * k;
    dst[3] = 255.0 * a + dst[3] * k;
}

/// Premultiplied BGRA sprite. `ring` = (radius in logical units, alpha) draws a click ripple.
fn render(size: i32, s: f32, ring: Option<(f32, f32)>) -> Vec<u32> {
    let poly = pts(s);
    let glow_r = 10.0 * s;
    let outline = 1.7 * s;
    let tip = TIP * s;
    let mut out = vec![0u32; (size * size) as usize];
    for y in 0..size {
        for x in 0..size {
            let mut acc = [0f32; 4];
            for sy in 0..2 {
                for sx in 0..2 {
                    let px = x as f32 + (sx as f32 + 0.5) / 2.0;
                    let py = y as f32 + (sy as f32 + 0.5) / 2.0;
                    let mut p = [0f32; 4];
                    let d = edge_dist(&poly, px, py);
                    let is_in = inside(&poly, px, py);
                    if !is_in && d < glow_r {
                        let g = 1.0 - d / glow_r;
                        over(&mut p, COLOR, 0.55 * g * g);
                    }
                    if let Some((r, a)) = ring {
                        let rr = ((px - tip).powi(2) + (py - tip).powi(2)).sqrt();
                        let w = 2.4 * s;
                        let e = 1.0 - ((rr - r * s).abs() / w).min(1.0);
                        if e > 0.0 {
                            over(&mut p, COLOR, a * e);
                        }
                    }
                    if is_in {
                        over(&mut p, COLOR, 1.0);
                    } else if d <= outline {
                        over(&mut p, (255.0, 255.0, 255.0), 1.0);
                    }
                    for i in 0..4 {
                        acc[i] += p[i];
                    }
                }
            }
            let n = 4.0;
            out[(y * size + x) as usize] = ((acc[3] / n) as u32) << 24
                | ((acc[0] / n) as u32) << 16
                | ((acc[1] / n) as u32) << 8
                | (acc[2] / n) as u32;
        }
    }
    out
}

unsafe fn sprite(screen: HDC, size: i32, px: &[u32]) -> HDC {
    let mdc = CreateCompatibleDC(Some(screen));
    let bi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size,
            biHeight: -size,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits: *mut c_void = std::ptr::null_mut();
    if let Ok(bmp) = CreateDIBSection(Some(screen), &bi, DIB_RGB_COLORS, &mut bits, None, 0) {
        std::ptr::copy_nonoverlapping(px.as_ptr(), bits as *mut u32, px.len());
        SelectObject(mdc, bmp.into());
    }
    mdc
}

unsafe extern "system" fn wndproc(
    h: windows::Win32::Foundation::HWND,
    m: u32,
    w: windows::Win32::Foundation::WPARAM,
    l: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    DefWindowProcW(h, m, w, l)
}

fn run(rx: mpsc::Receiver<Msg>) {
    unsafe {
        let Ok(hinst) = GetModuleHandleW(None) else { return };
        let cls = w!("FastuseAgentCursor");
        let wc = WNDCLASSW { lpfnWndProc: Some(wndproc), hInstance: hinst.into(), lpszClassName: cls, ..Default::default() };
        RegisterClassW(&wc);
        let s = (GetDpiForSystem() as f32 / 96.0).max(1.0);
        let size = (CANVAS * s).ceil() as i32;
        let tip = (TIP * s) as i32;
        let Ok(hwnd) = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            cls,
            w!(""),
            WS_POPUP,
            0,
            0,
            size,
            size,
            None,
            None,
            Some(hinst.into()),
            None,
        ) else {
            return;
        };
        let screen = GetDC(None);
        let normal = sprite(screen, size, &render(size, s, None));
        let ripple: Vec<HDC> = (0..RIPPLE_FRAMES)
            .map(|i| {
                let f = i as f32 / (RIPPLE_FRAMES - 1) as f32;
                sprite(screen, size, &render(size, s, Some((8.0 + 24.0 * f, 0.95 * (1.0 - f)))))
            })
            .collect();

        let skin = |dc: HDC, alpha: u8| {
            let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: alpha, AlphaFormat: AC_SRC_ALPHA as u8 };
            let sz = SIZE { cx: size, cy: size };
            let src = POINT { x: 0, y: 0 };
            let _ = UpdateLayeredWindow(hwnd, None, None, Some(&sz), Some(dc), Some(&src), COLORREF(0), Some(&blend), ULW_ALPHA);
        };
        let place = |x: f32, y: f32| {
            let _ = SetWindowPos(hwnd, None, x as i32 - tip, y as i32 - tip, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER);
        };
        let pump = || {
            let mut m = MSG::default();
            while PeekMessageW(&mut m, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&m);
                DispatchMessageW(&m);
            }
        };

        let mut pos = (0f32, 0f32);
        let mut visible = false;
        let mut last = Instant::now();
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(first) => {
                    // coalesce a burst (a batch of steps): glide to the last target, ripple if any clicked
                    let (mut tx_, mut ty_, mut click) = (first.x, first.y, first.click);
                    let mut acks: Vec<mpsc::Sender<()>> = first.ack.into_iter().collect();
                    while let Ok(n) = rx.try_recv() {
                        tx_ = n.x;
                        ty_ = n.y;
                        click |= n.click;
                        acks.extend(n.ack);
                    }
                    let (tx_, ty_) = (tx_ as f32, ty_ as f32);
                    if !visible {
                        pos = (tx_ - 90.0 * s, ty_ - 60.0 * s); // glide in from up-left
                        skin(normal, 255);
                        place(pos.0, pos.1);
                        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                        visible = true;
                    } else {
                        skin(normal, 255); // undo a fade in progress
                    }
                    // curved glide: quadratic bezier bowed sideways, eased
                    let (dx, dy) = (tx_ - pos.0, ty_ - pos.1);
                    let dist = (dx * dx + dy * dy).sqrt();
                    let ms = (160.0 + dist / s * 0.45).clamp(180.0, 420.0);
                    let steps = (ms / 14.0) as i32;
                    let (cx, cy) = ((pos.0 + tx_) / 2.0 - dy * 0.18, (pos.1 + ty_) / 2.0 + dx * 0.18);
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let e = t * t * (3.0 - 2.0 * t);
                        let u = 1.0 - e;
                        place(u * u * pos.0 + 2.0 * u * e * cx + e * e * tx_, u * u * pos.1 + 2.0 * u * e * cy + e * e * ty_);
                        std::thread::sleep(Duration::from_millis(14));
                    }
                    place(tx_, ty_);
                    pos = (tx_, ty_);
                    for a in acks {
                        let _ = a.send(());
                    }
                    if click {
                        for dc in &ripple {
                            skin(*dc, 255);
                            std::thread::sleep(Duration::from_millis(28));
                        }
                        skin(normal, 255);
                    }
                    last = Instant::now();
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if visible && last.elapsed() > IDLE_FADE {
                        for a in (0..=8).rev() {
                            skin(normal, (a * 31) as u8);
                            std::thread::sleep(Duration::from_millis(30));
                        }
                        let _ = ShowWindow(hwnd, SW_HIDE);
                        visible = false;
                    }
                }
                Err(_) => return,
            }
            pump();
        }
    }
}
