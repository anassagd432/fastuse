//! Window and desktop capture into raw BGRA frames. Encoding and caching live in `imaging`.

use std::ffi::c_void;
use std::time::Instant;

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, ROP_CODE, SRCCOPY,
};
use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, GetWindowRect, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

use crate::imaging::Frame;
use crate::platform::CaptureTarget;

unsafe fn dib(sdc: windows::Win32::Graphics::Gdi::HDC, w: i32, h: i32) -> Result<(windows::Win32::Graphics::Gdi::HBITMAP, *mut c_void), String> {
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
    Ok((bmp, bits))
}

fn window(hwnd: HWND, screen: bool) -> Result<Frame, String> {
    unsafe {
        let mut r = Default::default();
        GetWindowRect(hwnd, &mut r).map_err(|e| e.to_string())?;
        let (w, h) = ((r.right - r.left).max(1), (r.bottom - r.top).max(1));
        let sdc = GetDC(None);
        let mdc = CreateCompatibleDC(Some(sdc));
        let (bmp, bits) = dib(sdc, w, h)?;
        let old = SelectObject(mdc, bmp.into());
        let ok = if screen {
            BitBlt(mdc, 0, 0, w, h, Some(sdc), r.left, r.top, SRCCOPY).is_ok()
        } else {
            PrintWindow(hwnd, mdc, PRINT_WINDOW_FLAGS(2)).as_bool() // PW_RENDERFULLCONTENT
        };
        let bgra = std::slice::from_raw_parts(bits as *const u8, (w * h * 4) as usize).to_vec();
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
fn desktop() -> Result<Frame, String> {
    unsafe {
        let (x, y) = (GetSystemMetrics(SM_XVIRTUALSCREEN), GetSystemMetrics(SM_YVIRTUALSCREEN));
        let (w, h) = (GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1), GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1));
        let sdc = GetDC(None);
        let mdc = CreateCompatibleDC(Some(sdc));
        let (bmp, bits) = dib(sdc, w, h)?;
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

pub fn grab(t: CaptureTarget, screen_mode: bool) -> Result<Frame, String> {
    match t {
        CaptureTarget::Desktop => desktop(),
        CaptureTarget::Window(id) => window(HWND(id as isize as *mut c_void), screen_mode),
    }
}
