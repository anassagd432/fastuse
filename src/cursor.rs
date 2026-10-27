//! The agent's own cursor: a click-through, topmost, non-activating layered window that glides to
//! wherever an action lands. It never touches the real mouse pointer or keyboard focus.
//! Disable with FASTUSE_CURSOR=0.

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
use windows::Win32::UI::WindowsAndMessaging::*;

const SZ: i32 = 40; // sprite size
const TIP: i32 = 10; // arrow tip offset inside the sprite

static TX: OnceLock<mpsc::Sender<(i32, i32, bool)>> = OnceLock::new();

pub fn init() {
    if std::env::var("FASTUSE_CURSOR").map(|v| v == "0").unwrap_or(false) {
        return;
    }
    let (tx, rx) = mpsc::channel();
    if TX.set(tx).is_ok() {
        std::thread::spawn(move || run(rx));
    }
}

/// Glide the agent cursor to screen point (x, y); `click` adds a press pulse.
pub fn go(x: i32, y: i32, click: bool) {
    if let Some(tx) = TX.get() {
        let _ = tx.send((x, y, click));
    }
}

const ARROW: [(f32, f32); 7] = [(0.0, 0.0), (0.0, 17.0), (4.0, 13.0), (7.0, 20.0), (10.0, 19.0), (7.0, 12.0), (12.0, 12.0)];

fn inside(px: f32, py: f32) -> bool {
    let mut c = false;
    let mut j = ARROW.len() - 1;
    for i in 0..ARROW.len() {
        let (xi, yi) = (ARROW[i].0 + TIP as f32, ARROW[i].1 + TIP as f32);
        let (xj, yj) = (ARROW[j].0 + TIP as f32, ARROW[j].1 + TIP as f32);
        if (yi > py) != (yj > py) && px < (xj - xi) * (py - yi) / (yj - yi) + xi {
            c = !c;
        }
        j = i;
    }
    c
}

fn edge_dist(px: f32, py: f32) -> f32 {
    let mut best = f32::MAX;
    for i in 0..ARROW.len() {
        let a = (ARROW[i].0 + TIP as f32, ARROW[i].1 + TIP as f32);
        let b = (ARROW[(i + 1) % ARROW.len()].0 + TIP as f32, ARROW[(i + 1) % ARROW.len()].1 + TIP as f32);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let t = (((px - a.0) * dx + (py - a.1) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        let (qx, qy) = (a.0 + t * dx, a.1 + t * dy);
        best = best.min(((px - qx).powi(2) + (py - qy).powi(2)).sqrt());
    }
    best
}

/// Premultiplied BGRA sprite. `ring` adds the click pulse around the tip.
fn render(ring: bool) -> Vec<u32> {
    let mut out = vec![0u32; (SZ * SZ) as usize];
    for y in 0..SZ {
        for x in 0..SZ {
            let (mut r, mut g, mut b, mut a) = (0f32, 0f32, 0f32, 0f32);
            for sy in 0..4 {
                for sx in 0..4 {
                    let px = x as f32 + (sx as f32 + 0.5) / 4.0;
                    let py = y as f32 + (sy as f32 + 0.5) / 4.0;
                    let (cr, cg, cb, ca) = if inside(px, py) {
                        (47.0, 107.0, 255.0, 255.0)
                    } else if edge_dist(px, py) <= 1.4 {
                        (255.0, 255.0, 255.0, 255.0)
                    } else if ring && (((px - TIP as f32).powi(2) + (py - TIP as f32).powi(2)).sqrt() - 9.0).abs() <= 1.2 {
                        (47.0, 107.0, 255.0, 200.0)
                    } else {
                        (0.0, 0.0, 0.0, 0.0)
                    };
                    let af = ca / 255.0;
                    r += cr * af;
                    g += cg * af;
                    b += cb * af;
                    a += ca;
                }
            }
            let n = 16.0;
            out[(y * SZ + x) as usize] =
                ((a / n) as u32) << 24 | ((r / n) as u32) << 16 | ((g / n) as u32) << 8 | (b / n) as u32;
        }
    }
    out
}

unsafe fn sprite(screen: HDC, ring: bool) -> HDC {
    let mdc = CreateCompatibleDC(Some(screen));
    let bi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: SZ,
            biHeight: -SZ,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits: *mut c_void = std::ptr::null_mut();
    if let Ok(bmp) = CreateDIBSection(Some(screen), &bi, DIB_RGB_COLORS, &mut bits, None, 0) {
        let px = render(ring);
        std::ptr::copy_nonoverlapping(px.as_ptr(), bits as *mut u32, px.len());
        SelectObject(mdc, bmp.into());
    }
    mdc
}

unsafe extern "system" fn wndproc(h: windows::Win32::Foundation::HWND, m: u32, w: windows::Win32::Foundation::WPARAM, l: windows::Win32::Foundation::LPARAM) -> windows::Win32::Foundation::LRESULT {
    DefWindowProcW(h, m, w, l)
}

fn run(rx: mpsc::Receiver<(i32, i32, bool)>) {
    unsafe {
        let Ok(hinst) = GetModuleHandleW(None) else { return };
        let cls = w!("FastuseAgentCursor");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: hinst.into(),
            lpszClassName: cls,
            ..Default::default()
        };
        RegisterClassW(&wc);
        let Ok(hwnd) = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            cls,
            w!(""),
            WS_POPUP,
            0,
            0,
            SZ,
            SZ,
            None,
            None,
            Some(hinst.into()),
            None,
        ) else {
            return;
        };
        let screen = GetDC(None);
        let normal = sprite(screen, false);
        let pressed = sprite(screen, true);
        let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
        let put = |dc: HDC, x: i32, y: i32| {
            let pos = POINT { x: x - TIP, y: y - TIP };
            let size = SIZE { cx: SZ, cy: SZ };
            let src = POINT { x: 0, y: 0 };
            let _ = UpdateLayeredWindow(hwnd, Some(screen), Some(&pos), Some(&size), Some(dc), Some(&src), COLORREF(0), Some(&blend), ULW_ALPHA);
        };
        let pump = || {
            let mut m = MSG::default();
            while PeekMessageW(&mut m, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&m);
                DispatchMessageW(&m);
            }
        };

        let mut pos = (0i32, 0i32);
        let mut visible = false;
        let mut last = Instant::now();
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok((tx_, ty_, click)) => {
                    if !visible {
                        pos = (tx_ - 60, ty_ - 40); // glide in from up-left on first appearance
                        put(normal, pos.0, pos.1);
                        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                        visible = true;
                    }
                    let (sx, sy) = pos;
                    let steps = 6;
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let e = 1.0 - (1.0 - t).powi(3); // ease-out
                        put(normal, sx + ((tx_ - sx) as f32 * e) as i32, sy + ((ty_ - sy) as f32 * e) as i32);
                        std::thread::sleep(Duration::from_millis(6));
                    }
                    pos = (tx_, ty_);
                    if click {
                        put(pressed, pos.0, pos.1);
                        std::thread::sleep(Duration::from_millis(90));
                        put(normal, pos.0, pos.1);
                    }
                    last = Instant::now();
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if visible && last.elapsed() > Duration::from_millis(2500) {
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
