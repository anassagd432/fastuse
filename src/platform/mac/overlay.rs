//! macOS surface for the agent overlay (see crate::overlay for the look and the motion).
//!
//! AppKit only works on the main thread, so `run_app` (mod.rs) gives the main thread to
//! `NSApplication.run()` and runs the MCP server on a worker. The overlay thread computes
//! positions and pixels, and sends every window change to the main queue. The windows are
//! borderless, transparent, click-through, above everything and on every Space; the app is an
//! "accessory" (no Dock icon), so nothing here can take keyboard focus from the user.
//!
//! Units: the agent works in points (what the Accessibility API reports). Sprites are rendered
//! at the display's backing scale so they stay sharp on Retina screens.

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::mpsc;
use std::time::Duration;

use objc2::rc::Retained;
use objc2::{AnyThread, MainThreadMarker};
use objc2_app_kit::{
    NSBackingStoreType, NSBitmapImageRep, NSBox, NSBoxType, NSColor, NSDeviceRGBColorSpace, NSFont, NSImage, NSImageView,
    NSScreen, NSTextAlignment, NSTextField, NSTitlePosition, NSWindow, NSWindowCollectionBehavior, NSWindowSharingType,
    NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use crate::overlay::{self, Surface, CANVAS, RIPPLE_FRAMES, TIP};

// ---- running things on the main thread -------------------------------------------------------

extern "C" {
    static _dispatch_main_q: c_void;
    fn dispatch_async_f(queue: *const c_void, context: *mut c_void, work: extern "C" fn(*mut c_void));
}

extern "C" fn trampoline<F: FnOnce()>(ctx: *mut c_void) {
    // SAFETY: `ctx` is the Box<F> leaked by `on_main`, and libdispatch calls this exactly once.
    let f = unsafe { Box::from_raw(ctx as *mut F) };
    f();
}

/// Queue `f` on the main thread and return immediately. Order is preserved.
fn on_main<F: FnOnce() + Send + 'static>(f: F) {
    let ctx = Box::into_raw(Box::new(f)) as *mut c_void;
    // SAFETY: the main queue outlives the process; `trampoline::<F>` frees `ctx`.
    unsafe { dispatch_async_f(&raw const _dispatch_main_q, ctx, trampoline::<F>) };
}

/// Run `f` on the main thread and wait for its result (None if the main loop is not running).
fn on_main_sync<R: Send + 'static, F: FnOnce() -> R + Send + 'static>(f: F) -> Option<R> {
    let (tx, rx) = mpsc::channel();
    on_main(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(Duration::from_secs(3)).ok()
}

// ---- main-thread UI state --------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
struct ScreenInfo {
    /// Cocoa frame in points (origin bottom-left of the primary screen)
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    /// top edge of the usable area (below the menu bar and notch)
    top: f64,
    backing: f64,
}

struct Ui {
    mtm: MainThreadMarker,
    primary_h: f64,
    recordable: bool,
    cursor: Retained<NSWindow>,
    cursor_view: Retained<NSImageView>,
    sprites: Vec<Retained<NSImage>>,
    glow: Vec<Retained<NSWindow>>,
    banner: Option<Retained<NSWindow>>,
}

thread_local! {
    static UI: RefCell<Option<Ui>> = const { RefCell::new(None) };
}

fn with_ui<R>(f: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    UI.with(|u| {
        let mut u = u.borrow_mut();
        if u.is_none() {
            *u = Ui::new();
        }
        u.as_mut().map(f)
    })
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> NSRect {
    NSRect::new(NSPoint::new(x, y), NSSize::new(w, h))
}

fn ns_color(c: (f32, f32, f32), k: f32, a: f64) -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha((c.0 * k / 255.0) as f64, (c.1 * k / 255.0) as f64, (c.2 * k / 255.0) as f64, a)
}

/// A bitmap of premultiplied 0xAARRGGBB pixels shown at `lw`x`lh` points.
fn make_image(wpx: i32, hpx: i32, lw: f64, lh: f64, px: &[u32]) -> Option<Retained<NSImage>> {
    let rgba = overlay::to_rgba_bytes(px);
    // SAFETY: plain bitmap allocation; the buffer is copied into the rep's own storage below.
    unsafe {
        let rep = NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
            NSBitmapImageRep::alloc(),
            std::ptr::null_mut(),
            wpx as isize,
            hpx as isize,
            8,
            4,
            true,
            false,
            NSDeviceRGBColorSpace,
            (wpx * 4) as isize,
            32,
        )?;
        let dst = rep.bitmapData();
        if dst.is_null() {
            return None;
        }
        std::ptr::copy_nonoverlapping(rgba.as_ptr(), dst, rgba.len().min((wpx * hpx * 4) as usize));
        rep.setSize(NSSize::new(lw, lh));
        let img = NSImage::initWithSize(NSImage::alloc(), NSSize::new(lw, lh));
        img.addRepresentation(&rep);
        Some(img)
    }
}

impl Ui {
    fn new() -> Option<Ui> {
        let mtm = MainThreadMarker::new()?;
        let screens = NSScreen::screens(mtm);
        let primary = screens.firstObject()?;
        let backing = primary.backingScaleFactor();
        let primary_h = primary.frame().size.height;
        let recordable = std::env::var("FASTUSE_RECORDABLE").map(|v| v == "1").unwrap_or(false);

        let cursor = overlay_window(mtm, rect(0.0, 0.0, CANVAS as f64, CANVAS as f64), 1002, recordable, true);
        let cursor_view = NSImageView::initWithFrame(mtm.alloc(), rect(0.0, 0.0, CANVAS as f64, CANVAS as f64));
        cursor.setContentView(Some(&cursor_view));
        let size = (CANVAS as f64 * backing).ceil() as i32;
        let s = backing as f32;
        let mut sprites = Vec::new();
        let mut add = |ring: Option<(f32, f32)>| {
            if let Some(i) = make_image(size, size, CANVAS as f64, CANVAS as f64, &overlay::render_cursor(size, s, ring)) {
                sprites.push(i);
            }
        };
        add(None);
        for i in 0..RIPPLE_FRAMES {
            let f = i as f32 / (RIPPLE_FRAMES - 1) as f32;
            add(Some((4.0 + 15.0 * f, 0.75 * (1.0 - f))));
        }
        if sprites.len() != RIPPLE_FRAMES + 1 {
            return None;
        }
        Some(Ui { mtm, primary_h, recordable, cursor, cursor_view, sprites, glow: Vec::new(), banner: None })
    }

    fn screens(&self) -> Vec<ScreenInfo> {
        let screens = NSScreen::screens(self.mtm);
        (0..screens.count())
            .map(|i| {
                let s = screens.objectAtIndex(i);
                let (f, v) = (s.frame(), s.visibleFrame());
                ScreenInfo {
                    x: f.origin.x,
                    y: f.origin.y,
                    w: f.size.width,
                    h: f.size.height,
                    top: v.origin.y + v.size.height,
                    backing: s.backingScaleFactor(),
                }
            })
            .collect()
    }

    /// (Re)build the glow windows and the banner for `screens`; `glow` holds one pixel buffer per screen.
    fn build_frame(&mut self, screens: &[ScreenInfo], glow: Vec<Vec<u32>>) {
        for w in self.glow.drain(..) {
            w.orderOut(None);
        }
        if let Some(b) = self.banner.take() {
            b.orderOut(None);
        }
        self.primary_h = screens.first().map(|s| s.h).unwrap_or(self.primary_h);
        for (s, px) in screens.iter().zip(glow) {
            let (wpx, hpx) = ((s.w * s.backing) as i32, (s.h * s.backing) as i32);
            let Some(img) = make_image(wpx, hpx, s.w, s.h, &px) else { continue };
            let win = overlay_window(self.mtm, rect(s.x, s.y, s.w, s.h), 1000, self.recordable, false);
            let view = NSImageView::imageViewWithImage(&img, self.mtm);
            view.setFrame(rect(0.0, 0.0, s.w, s.h));
            win.setContentView(Some(&view));
            self.glow.push(win);
        }
        if let (Some(text), Some(p)) = (overlay::banner_text(), screens.first()) {
            let (bw, bh) = overlay::banner_size(&text, 1.0);
            let (bw, bh) = (bw as f64, bh as f64);
            let win = overlay_window(self.mtm, rect(p.x + (p.w - bw) / 2.0, p.top - bh - 10.0, bw, bh), 1001, self.recordable, false);
            let pill = NSBox::initWithFrame(self.mtm.alloc(), rect(0.0, 0.0, bw, bh));
            pill.setBoxType(NSBoxType::Custom);
            pill.setTitlePosition(NSTitlePosition::NoTitle);
            pill.setBorderWidth(1.0);
            pill.setCornerRadius(bh / 2.0);
            pill.setContentViewMargins(NSSize::new(0.0, 0.0));
            let col = overlay::color();
            pill.setFillColor(&ns_color(col, 0.62, 0.94));
            pill.setBorderColor(&ns_color((255.0, 255.0, 255.0), 0.85, 0.35));
            let label = NSTextField::labelWithString(&NSString::from_str(&text), self.mtm);
            label.setFont(Some(&NSFont::boldSystemFontOfSize(13.0)));
            label.setTextColor(Some(&NSColor::whiteColor()));
            label.setAlignment(NSTextAlignment::Center);
            label.setFrame(rect(0.0, (bh - 17.0) / 2.0 - 1.0, bw - 2.0, 17.0));
            if let Some(content) = pill.contentView() {
                content.addSubview(&label);
            }
            win.setContentView(Some(&pill));
            self.banner = Some(win);
        }
    }
}

/// A transparent, click-through window above everything, on every Space, that never takes focus.
fn overlay_window(mtm: MainThreadMarker, frame: NSRect, level: isize, recordable: bool, keep_in_capture: bool) -> Retained<NSWindow> {
    // SAFETY: standard NSWindow construction on the main thread; the window is kept alive by `Retained`.
    unsafe {
        let w = NSWindow::initWithContentRect_styleMask_backing_defer(
            mtm.alloc(),
            frame,
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        );
        w.setReleasedWhenClosed(false);
        w.setOpaque(false);
        w.setBackgroundColor(Some(&NSColor::clearColor()));
        w.setHasShadow(false);
        w.setIgnoresMouseEvents(true);
        w.setLevel(level);
        w.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::Stationary
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::IgnoresCycle,
        );
        // the glow and banner stay out of screenshots (the model sees the real screen) unless recording a demo
        if !recordable && !keep_in_capture {
            w.setSharingType(NSWindowSharingType::None);
        }
        w
    }
}

// ---- the Surface -----------------------------------------------------------------------------

pub struct MacSurface {
    shown: Vec<ScreenInfo>,
}

impl MacSurface {
    pub fn new() -> Option<MacSurface> {
        // make sure AppKit is up and the sprites render before the first action needs them
        on_main_sync(|| with_ui(|_| ()))??;
        Some(MacSurface { shown: Vec::new() })
    }
}

impl Surface for MacSurface {
    /// Positions are points, so one logical unit is one point.
    fn scale(&self) -> f32 {
        1.0
    }

    fn cursor_frame(&mut self, frame: usize, alpha: u8) {
        on_main(move || {
            with_ui(|ui| {
                if let Some(img) = ui.sprites.get(frame) {
                    ui.cursor_view.setImage(Some(img));
                }
                ui.cursor.setAlphaValue(alpha as f64 / 255.0);
            });
        });
    }

    fn cursor_place(&mut self, x: f32, y: f32) {
        // Accessibility/CoreGraphics points have their origin top-left; Cocoa's is bottom-left.
        on_main(move || {
            with_ui(|ui| {
                ui.cursor.setFrameOrigin(NSPoint::new(x as f64 - TIP as f64, ui.primary_h - y as f64 - (CANVAS - TIP) as f64));
            });
        });
    }

    fn cursor_visible(&mut self, on: bool) {
        on_main(move || {
            with_ui(|ui| {
                if on {
                    ui.cursor.orderFrontRegardless();
                } else {
                    ui.cursor.orderOut(None);
                }
            });
        });
    }

    fn frame_visible(&mut self, on: bool) {
        if !on {
            on_main(|| {
                with_ui(|ui| {
                    for w in ui.glow.iter().chain(ui.banner.iter()) {
                        w.orderOut(None);
                    }
                });
            });
            return;
        }
        let Some(screens) = on_main_sync(|| with_ui(|ui| ui.screens())).flatten() else { return };
        if screens == self.shown {
            on_main(|| {
                with_ui(|ui| {
                    for w in ui.glow.iter().chain(ui.banner.iter()) {
                        w.orderFrontRegardless();
                    }
                    ui.cursor.orderFrontRegardless();
                });
            });
            return;
        }
        // new or changed screens: render the glow here (off the main thread), then build the windows
        let px: Vec<Vec<u32>> = screens
            .iter()
            .map(|s| overlay::render_glow((s.w * s.backing) as i32, (s.h * s.backing) as i32, s.backing as f32))
            .collect();
        let layout = screens.clone();
        on_main(move || {
            with_ui(|ui| {
                ui.build_frame(&layout, px);
                for w in ui.glow.iter().chain(ui.banner.iter()) {
                    w.setAlphaValue(0.0);
                    w.orderFrontRegardless();
                }
                ui.cursor.orderFrontRegardless();
            });
        });
        self.shown = screens;
    }

    fn frame_alpha(&mut self, glow: f32, banner: f32) {
        on_main(move || {
            with_ui(|ui| {
                for w in &ui.glow {
                    w.setAlphaValue(glow.clamp(0.0, 1.0) as f64);
                }
                if let Some(b) = &ui.banner {
                    b.setAlphaValue(banner.clamp(0.0, 1.0) as f64);
                }
            });
        });
    }
}
