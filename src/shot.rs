//! Window capture: PrintWindow into a DIB section, JPEG encode, per-window frame cache.

use std::collections::HashMap;
use std::ffi::c_void;
use std::time::Instant;

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
    SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};
use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

pub struct Frame {
    pub w: usize,
    pub h: usize,
    pub bgra: Vec<u8>,
    pub at: Instant,
}

#[derive(Default)]
pub struct Cache {
    frames: HashMap<isize, Frame>,
}

pub struct Opts {
    pub region: Option<[usize; 4]>, // x, y, w, h in window pixels
    pub max_dim: usize,             // 0 = no downscale
    pub quality: u8,
    pub max_age_ms: u128,           // reuse cached frame younger than this
    pub screen: bool,               // BitBlt from screen instead of PrintWindow
}

pub struct Out {
    pub jpeg: Vec<u8>,
    pub w: usize,
    pub h: usize,
    pub from_cache: bool,
    pub capture_us: u128,
    pub encode_us: u128,
}

fn grab(hwnd: HWND, screen: bool) -> Result<Frame, String> {
    unsafe {
        let mut r = Default::default();
        GetWindowRect(hwnd, &mut r).map_err(|e| e.to_string())?;
        let (w, h) = ((r.right - r.left).max(1), (r.bottom - r.top).max(1));
        let sdc = GetDC(None);
        let mdc = CreateCompatibleDC(Some(sdc));
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
        let bmp = CreateDIBSection(Some(sdc), &bi, DIB_RGB_COLORS, &mut bits, None, 0)
            .map_err(|e| e.to_string())?;
        let old = SelectObject(mdc, bmp.into());
        let ok = if screen {
            BitBlt(mdc, 0, 0, w, h, Some(sdc), r.left, r.top, SRCCOPY).is_ok()
        } else {
            PrintWindow(hwnd, mdc, PRINT_WINDOW_FLAGS(2)).as_bool() // PW_RENDERFULLCONTENT
        };
        let n = (w * h * 4) as usize;
        let bgra = std::slice::from_raw_parts(bits as *const u8, n).to_vec();
        SelectObject(mdc, old);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mdc);
        ReleaseDC(None, sdc);
        if !ok {
            return Err("capture failed".into());
        }
        Ok(Frame { w: w as usize, h: h as usize, bgra, at: Instant::now() })
    }
}

/// Whole virtual desktop (all monitors), including layered windows such as the agent cursor.
fn grab_screen() -> Result<Frame, String> {
    use windows::Win32::Graphics::Gdi::{CAPTUREBLT, ROP_CODE};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
    };
    unsafe {
        let (x, y) = (GetSystemMetrics(SM_XVIRTUALSCREEN), GetSystemMetrics(SM_YVIRTUALSCREEN));
        let (w, h) = (GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1), GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1));
        let sdc = GetDC(None);
        let mdc = CreateCompatibleDC(Some(sdc));
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
        let bmp = CreateDIBSection(Some(sdc), &bi, DIB_RGB_COLORS, &mut bits, None, 0).map_err(|e| e.to_string())?;
        let old = SelectObject(mdc, bmp.into());
        let ok = BitBlt(mdc, 0, 0, w, h, Some(sdc), x, y, ROP_CODE(SRCCOPY.0 | CAPTUREBLT.0)).is_ok();
        let bgra = std::slice::from_raw_parts(bits as *const u8, (w * h * 4) as usize).to_vec();
        SelectObject(mdc, old);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mdc);
        ReleaseDC(None, sdc);
        if !ok {
            return Err("desktop capture failed".into());
        }
        Ok(Frame { w: w as usize, h: h as usize, bgra, at: Instant::now() })
    }
}

impl Cache {
    pub fn capture(&mut self, hwnd: isize, o: &Opts) -> Result<Out, String> {
        let t0 = Instant::now();
        let fresh = match self.frames.get(&hwnd) {
            Some(f) if o.max_age_ms > 0 && f.at.elapsed().as_millis() < o.max_age_ms => false,
            _ => true,
        };
        if fresh {
            // hwnd 0 means the whole desktop
            let f = if hwnd == 0 { grab_screen()? } else { grab(HWND(hwnd as *mut c_void), o.screen)? };
            self.frames.insert(hwnd, f);
        }
        let capture_us = t0.elapsed().as_micros();
        let f = &self.frames[&hwnd];
        let t1 = Instant::now();

        // crop
        let (rx, ry, rw, rh) = match o.region {
            Some([x, y, w, h]) => {
                let x = x.min(f.w.saturating_sub(1));
                let y = y.min(f.h.saturating_sub(1));
                (x, y, w.min(f.w - x).max(1), h.min(f.h - y).max(1))
            }
            None => (0, 0, f.w, f.h),
        };
        // downscale by integer-ish nearest step
        let big = rw.max(rh);
        let (ow, oh) = if o.max_dim > 0 && big > o.max_dim {
            let s = o.max_dim as f32 / big as f32;
            (((rw as f32 * s) as usize).max(1), ((rh as f32 * s) as usize).max(1))
        } else {
            (rw, rh)
        };
        let mut buf = vec![0u8; ow * oh * 4];
        for oy in 0..oh {
            let sy = ry + oy * rh / oh;
            let srow = sy * f.w;
            let drow = oy * ow;
            if ow == rw {
                let s = (srow + rx) * 4;
                buf[drow * 4..(drow + ow) * 4].copy_from_slice(&f.bgra[s..s + ow * 4]);
            } else {
                for ox in 0..ow {
                    let sx = rx + ox * rw / ow;
                    let s = (srow + sx) * 4;
                    let d = (drow + ox) * 4;
                    buf[d..d + 4].copy_from_slice(&f.bgra[s..s + 4]);
                }
            }
        }
        let mut jpeg = Vec::with_capacity(ow * oh / 6);
        jpeg_encoder::Encoder::new(&mut jpeg, o.quality)
            .encode(&buf, ow as u16, oh as u16, jpeg_encoder::ColorType::Bgra)
            .map_err(|e| e.to_string())?;
        Ok(Out { jpeg, w: ow, h: oh, from_cache: !fresh, capture_us, encode_us: t1.elapsed().as_micros() })
    }
}
