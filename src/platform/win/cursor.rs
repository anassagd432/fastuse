//! Windows surface for the agent overlay (see crate::overlay for the look and the motion):
//! click-through, topmost, non-activating layered windows for the cursor, a glow around each
//! monitor edge and a banner at the top. None of them ever touches the real mouse pointer or
//! keyboard focus; the glow and banner are also excluded from screen capture, so the model's
//! screenshots stay clean.

use std::ffi::c_void;

use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject, DrawTextW, EnumDisplayMonitors, GetDC,
    GetMonitorInfoW, SelectObject, SetBkMode, SetTextColor, AC_SRC_ALPHA, AC_SRC_OVER, ANTIALIASED_QUALITY, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DIB_RGB_COLORS, DT_CENTER,
    DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, HBITMAP, HDC, HMONITOR, MONITORINFO, OUT_DEFAULT_PRECIS, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::overlay::{self, Surface, CANVAS, RIPPLE_FRAMES, TIP};

pub fn init() {
    overlay::init(|| unsafe { WinSurface::new() });
}

pub fn go(x: i32, y: i32, click: bool) {
    overlay::go(x, y, click);
}

pub fn set_active(active: bool) {
    overlay::set_active(active);
}

pub fn enabled() -> bool {
    overlay::enabled()
}

/// A 32-bit premultiplied bitmap selected into a memory DC.
struct Sprite {
    dc: HDC,
    bmp: Option<HBITMAP>,
}

impl Sprite {
    unsafe fn new(screen: HDC, w: i32, h: i32, px: &[u32]) -> Sprite {
        let dc = CreateCompatibleDC(Some(screen));
        let bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut c_void = std::ptr::null_mut();
        let bmp = CreateDIBSection(Some(screen), &bi, DIB_RGB_COLORS, &mut bits, None, 0).ok();
        if let Some(b) = bmp {
            std::ptr::copy_nonoverlapping(px.as_ptr(), bits as *mut u32, px.len().min((w * h) as usize));
            SelectObject(dc, b.into());
        }
        Sprite { dc, bmp }
    }
}

impl Drop for Sprite {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteDC(self.dc);
            if let Some(b) = self.bmp {
                let _ = DeleteObject(b.into());
            }
        }
    }
}

/// Render `text` in white on black into a temporary bitmap and return its coverage mask.
unsafe fn text_coverage(screen: HDC, w: i32, h: i32, s: f32, text: &str) -> Vec<u8> {
    let dc = CreateCompatibleDC(Some(screen));
    let bi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits: *mut c_void = std::ptr::null_mut();
    let mut out = vec![0u8; (w * h) as usize];
    if let Ok(bmp) = CreateDIBSection(Some(screen), &bi, DIB_RGB_COLORS, &mut bits, None, 0) {
        SelectObject(dc, bmp.into());
        let font = CreateFontW(
            -((13.0 * s).round() as i32),
            0,
            0,
            0,
            600,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            ANTIALIASED_QUALITY,
            0,
            w!("Segoe UI"),
        );
        SelectObject(dc, font.into());
        SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, COLORREF(0x00FF_FFFF));
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        let mut rc = RECT { left: 0, top: 0, right: w, bottom: h };
        DrawTextW(dc, &mut wide, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
        let px = std::slice::from_raw_parts(bits as *const u32, (w * h) as usize);
        for (o, p) in out.iter_mut().zip(px) {
            *o = (*p & 0xFF) as u8;
        }
        let _ = DeleteObject(font.into());
        let _ = DeleteObject(bmp.into());
    }
    let _ = DeleteDC(dc);
    out
}

unsafe extern "system" fn wndproc(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    DefWindowProcW(h, m, w, l)
}

/// Push a premultiplied sprite into a layered window. `pos` also moves the window.
unsafe fn push(hwnd: HWND, dc: HDC, w: i32, h: i32, pos: Option<(i32, i32)>, alpha: u8) {
    let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: alpha, AlphaFormat: AC_SRC_ALPHA as u8 };
    let sz = SIZE { cx: w, cy: h };
    let src = POINT { x: 0, y: 0 };
    let dst = pos.map(|(x, y)| POINT { x, y });
    let dstp = dst.as_ref().map(|p| p as *const POINT);
    let _ = UpdateLayeredWindow(hwnd, None, dstp, Some(&sz), Some(dc), Some(&src), COLORREF(0), Some(&blend), ULW_ALPHA);
}

/// Keep the glow and banner out of screenshots (so the model sees the real screen), unless
/// FASTUSE_RECORDABLE=1 asks for them to show up, e.g. when you screen-record a demo.
unsafe fn hide_from_capture(hwnd: HWND) {
    if std::env::var("FASTUSE_RECORDABLE").map(|v| v != "1").unwrap_or(true) {
        let _ = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
    }
}

unsafe fn overlay_window(x: i32, y: i32, w: i32, h: i32) -> Option<HWND> {
    let hinst = GetModuleHandleW(None).ok()?;
    CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
        w!("FastuseOverlay"),
        w!(""),
        WS_POPUP,
        x,
        y,
        w,
        h,
        None,
        None,
        Some(hinst.into()),
        None,
    )
    .ok()
}

struct Mon {
    rect: [i32; 4],
    work: [i32; 4],
    primary: bool,
}

unsafe extern "system" fn mon_cb(m: HMONITOR, _: HDC, _: *mut RECT, l: LPARAM) -> windows::core::BOOL {
    let v = &mut *(l.0 as *mut Vec<Mon>);
    let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
    if GetMonitorInfoW(m, &mut mi).as_bool() {
        let (r, k) = (mi.rcMonitor, mi.rcWork);
        v.push(Mon { rect: [r.left, r.top, r.right, r.bottom], work: [k.left, k.top, k.right, k.bottom], primary: mi.dwFlags & 1 != 0 });
    }
    true.into()
}

fn monitors() -> Vec<Mon> {
    let mut v: Vec<Mon> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(mon_cb), LPARAM(&mut v as *mut Vec<Mon> as isize));
    }
    v
}

struct Layer {
    hwnd: HWND,
    sprite: Sprite,
    rect: [i32; 4], // x, y, w, h
}

impl Layer {
    unsafe fn push(&self, alpha: u8, move_to: bool) {
        let [x, y, w, h] = self.rect;
        push(self.hwnd, self.sprite.dc, w, h, if move_to { Some((x, y)) } else { None }, alpha);
    }
}

impl Drop for Layer {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

pub struct WinSurface {
    s: f32,
    size: i32,
    tip: i32,
    screen: HDC,
    cursor: HWND,
    normal: Sprite,
    ripple: Vec<Sprite>,
    monitors: Vec<[i32; 4]>,
    glow: Vec<Layer>,
    banner: Option<Layer>,
}

impl WinSurface {
    unsafe fn new() -> Option<WinSurface> {
        let hinst = GetModuleHandleW(None).ok()?;
        let wc = WNDCLASSW { lpfnWndProc: Some(wndproc), hInstance: hinst.into(), lpszClassName: w!("FastuseOverlay"), ..Default::default() };
        RegisterClassW(&wc);
        let s = (GetDpiForSystem() as f32 / 96.0).max(1.0);
        let size = (CANVAS * s).ceil() as i32;
        let cursor = overlay_window(0, 0, size, size)?;
        let screen = GetDC(None);
        let normal = Sprite::new(screen, size, size, &overlay::render_cursor(size, s, None));
        let ripple = (0..RIPPLE_FRAMES)
            .map(|i| {
                let f = i as f32 / (RIPPLE_FRAMES - 1) as f32;
                Sprite::new(screen, size, size, &overlay::render_cursor(size, s, Some((4.0 + 15.0 * f, 0.75 * (1.0 - f)))))
            })
            .collect();
        Some(WinSurface { s, size, tip: (TIP * s) as i32, screen, cursor, normal, ripple, monitors: Vec::new(), glow: Vec::new(), banner: None })
    }

    /// (Re)build the per-monitor glow and the banner when the monitor layout changed.
    unsafe fn build_frame(&mut self) {
        let mons = monitors();
        let rects: Vec<[i32; 4]> = mons.iter().map(|m| m.rect).collect();
        if rects == self.monitors && !self.glow.is_empty() {
            return;
        }
        self.glow.clear();
        self.banner = None;
        for m in &mons {
            let (x, y, w, h) = (m.rect[0], m.rect[1], m.rect[2] - m.rect[0], m.rect[3] - m.rect[1]);
            if let Some(hwnd) = overlay_window(x, y, w, h) {
                hide_from_capture(hwnd);
                let sprite = Sprite::new(self.screen, w, h, &overlay::render_glow(w, h, self.s));
                self.glow.push(Layer { hwnd, sprite, rect: [x, y, w, h] });
            }
        }
        if let (Some(text), Some(m)) = (overlay::banner_text(), mons.iter().find(|m| m.primary).or(mons.first())) {
            let (bw, bh) = overlay::banner_size(&text, self.s);
            let x = (m.work[0] + m.work[2] - bw) / 2;
            let y = m.work[1] + (10.0 * self.s) as i32;
            let mut px = overlay::render_pill(bw, bh, self.s);
            overlay::composite_text(&mut px, &text_coverage(self.screen, bw, bh, self.s, &text));
            if let Some(hwnd) = overlay_window(x, y, bw, bh) {
                hide_from_capture(hwnd);
                self.banner = Some(Layer { hwnd, sprite: Sprite::new(self.screen, bw, bh, &px), rect: [x, y, bw, bh] });
            }
        }
        self.monitors = rects;
    }
}

impl Surface for WinSurface {
    fn scale(&self) -> f32 {
        self.s
    }

    fn cursor_frame(&mut self, frame: usize, alpha: u8) {
        let sp = if frame == 0 { &self.normal } else { &self.ripple[(frame - 1).min(self.ripple.len() - 1)] };
        unsafe { push(self.cursor, sp.dc, self.size, self.size, None, alpha) };
    }

    fn cursor_place(&mut self, x: f32, y: f32) {
        unsafe {
            let _ = SetWindowPos(self.cursor, None, x as i32 - self.tip, y as i32 - self.tip, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER);
        }
    }

    fn cursor_visible(&mut self, on: bool) {
        unsafe {
            let _ = ShowWindow(self.cursor, if on { SW_SHOWNOACTIVATE } else { SW_HIDE });
        }
    }

    fn frame_visible(&mut self, on: bool) {
        unsafe {
            if on {
                self.build_frame();
                for l in self.glow.iter().chain(self.banner.iter()) {
                    l.push(0, true);
                    let _ = ShowWindow(l.hwnd, SW_SHOWNOACTIVATE);
                }
                // keep the cursor above the glow and the banner
                let _ = SetWindowPos(self.cursor, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            } else {
                for l in self.glow.iter().chain(self.banner.iter()) {
                    let _ = ShowWindow(l.hwnd, SW_HIDE);
                }
            }
        }
    }

    fn frame_alpha(&mut self, glow: f32, banner: f32) {
        unsafe {
            for l in &self.glow {
                l.push((glow.clamp(0.0, 1.0) * 255.0) as u8, false);
            }
            if let Some(b) = &self.banner {
                b.push((banner.clamp(0.0, 1.0) * 255.0) as u8, false);
            }
        }
    }

    fn pump(&mut self) {
        unsafe {
            let mut m = MSG::default();
            while PeekMessageW(&mut m, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&m);
                DispatchMessageW(&m);
            }
        }
    }
}
