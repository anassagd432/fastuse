//! Portable image handling: crop, downscale, JPEG encode and a per-window frame cache.
//! Platforms only supply raw BGRA frames.

use std::collections::HashMap;
use std::time::Instant;

pub struct Frame {
    pub w: usize,
    pub h: usize,
    pub bgra: Vec<u8>,
    pub at: Instant,
}

pub struct Opts {
    pub region: Option<[usize; 4]>, // x, y, w, h in frame pixels
    pub max_dim: usize,             // 0 = no downscale
    pub quality: u8,
    pub max_age_ms: u128,           // reuse a cached frame younger than this
    pub screen: bool,               // ask the platform for a screen-region grab
}

pub struct Out {
    pub jpeg: Vec<u8>,
    pub w: usize,
    pub h: usize,
    pub from_cache: bool,
    pub capture_us: u128,
    pub encode_us: u128,
}

#[derive(Default)]
pub struct Cache {
    frames: HashMap<i64, Frame>,
}

impl Cache {
    /// `id` keys the cache (0 = desktop). `grab` is only called when no fresh-enough frame exists.
    pub fn capture(&mut self, id: i64, o: &Opts, grab: &mut dyn FnMut() -> Result<Frame, String>) -> Result<Out, String> {
        let t0 = Instant::now();
        let fresh = match self.frames.get(&id) {
            Some(f) if o.max_age_ms > 0 && f.at.elapsed().as_millis() < o.max_age_ms => false,
            _ => true,
        };
        if fresh {
            let f = grab()?;
            self.frames.insert(id, f);
        }
        let capture_us = t0.elapsed().as_micros();
        let f = &self.frames[&id];
        let t1 = Instant::now();

        let (rx, ry, rw, rh) = match o.region {
            Some([x, y, w, h]) => {
                let x = x.min(f.w.saturating_sub(1));
                let y = y.min(f.h.saturating_sub(1));
                (x, y, w.min(f.w - x).max(1), h.min(f.h - y).max(1))
            }
            None => (0, 0, f.w, f.h),
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: usize, h: usize) -> Frame {
        Frame { w, h, bgra: vec![200; w * h * 4], at: Instant::now() }
    }

    fn opts() -> Opts {
        Opts { region: None, max_dim: 0, quality: 70, max_age_ms: 0, screen: false }
    }

    #[test]
    fn crop_and_downscale() {
        let mut c = Cache::default();
        let mut o = opts();
        o.region = Some([10, 10, 100, 50]);
        let out = c.capture(1, &o, &mut || Ok(solid(200, 100))).unwrap();
        assert_eq!((out.w, out.h), (100, 50));
        o.region = None;
        o.max_dim = 100;
        let out = c.capture(1, &o, &mut || Ok(solid(200, 100))).unwrap();
        assert_eq!((out.w, out.h), (100, 50));
        assert_eq!(&out.jpeg[..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn cache_reuses_young_frames() {
        let mut c = Cache::default();
        let mut o = opts();
        o.max_age_ms = 60_000;
        let mut calls = 0;
        for _ in 0..3 {
            c.capture(7, &o, &mut || {
                calls += 1;
                Ok(solid(20, 20))
            })
            .unwrap();
        }
        assert_eq!(calls, 1);
    }
}
