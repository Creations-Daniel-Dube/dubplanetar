//! Image SDO/HMI continuum : téléchargement, disque seul, cache local.
//!
//! Le JPEG quick-look place le Soleil au centre et une légende dans la marge
//! noire. On suit le limbe depuis le centre ; le texte, au-delà du fond noir,
//! ne fait pas partie du disque.

use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use image::{Rgba, RgbaImage};

const URLS: [&str; 2] = [
    "https://sdo.gsfc.nasa.gov/assets/img/latest/latest_2048_HMIIC.jpg",
    "https://sdo.gsfc.nasa.gov/assets/img/latest/latest_1024_HMIIC.jpg",
];
const LIMB_THRESHOLD: u8 = 24;
const MAX_BYTES: u64 = 16 * 1024 * 1024;

pub enum Acquired {
    Live(RgbaImage),
    Cached(RgbaImage),
    Missing(String),
}

/// Télécharge le continuum du jour. Sans réseau, relit le dernier disque enregistré.
pub fn acquire() -> Acquired {
    match fetch_latest() {
        Ok(image) => {
            let _ = save_cache(&image);
            Acquired::Live(image)
        }
        Err(err) => load_cache()
            .map(Acquired::Cached)
            .unwrap_or(Acquired::Missing(err)),
    }
}

fn fetch_latest() -> Result<RgbaImage, String> {
    let mut last_err = String::from("téléchargement impossible");
    for url in URLS {
        match download(url) {
            Ok(bytes) => match decode_and_isolate(&bytes) {
                Some(image) => return Ok(image),
                None => last_err = format!("disque introuvable ({url})"),
            },
            Err(err) => last_err = err,
        }
    }
    Err(last_err)
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let response = ureq::get(url)
        .timeout(Duration::from_secs(20))
        .set("User-Agent", "dubPlanetar (affichage du Soleil)")
        .call()
        .map_err(|err| format!("{url} : {err}"))?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(MAX_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|err| format!("{url} : {err}"))?;
    if bytes.is_empty() {
        return Err(format!("{url} : réponse vide"));
    }
    Ok(bytes)
}

fn decode_and_isolate(bytes: &[u8]) -> Option<RgbaImage> {
    let image = image::load_from_memory(bytes).ok()?;
    isolate_disk(&image.to_rgba8())
}

pub fn cache_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .unwrap_or_else(std::env::temp_dir);
    base.join("dubPlanetar").join("sun_hmi.png")
}

fn save_cache(image: &RgbaImage) -> Result<(), String> {
    let path = cache_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    }
    image.save(&path).map_err(|err| err.to_string())
}

fn load_cache() -> Option<RgbaImage> {
    image::open(cache_path()).ok().map(|img| img.to_rgba8())
}

/// Carré du disque solaire. L'alpha vaut 0 en dehors du cercle.
pub fn isolate_disk(source: &RgbaImage) -> Option<RgbaImage> {
    let (w, h) = source.dimensions();
    if w < 32 || h < 32 {
        return None;
    }
    let cx = w / 2;
    let cy = h / 2;
    if luma(source.get_pixel(cx, cy)) < LIMB_THRESHOLD {
        return None;
    }
    let left = edge_from_center(source, cx, cy, -1, 0)?;
    let right = edge_from_center(source, cx, cy, 1, 0)?;
    let top = edge_from_center(source, cx, cy, 0, -1)?;
    let bottom = edge_from_center(source, cx, cy, 0, 1)?;
    if right <= left || bottom <= top {
        return None;
    }
    let center_x = (f64::from(left) + f64::from(right)) / 2.0;
    let center_y = (f64::from(top) + f64::from(bottom)) / 2.0;
    let radius = (f64::from(right - left) + f64::from(bottom - top)) / 4.0;
    if radius < 16.0 {
        return None;
    }
    let side = (radius * 2.0).round().max(1.0) as u32;
    let mut out = RgbaImage::new(side, side);
    let r2 = radius * radius;
    for y in 0..side {
        for x in 0..side {
            let src_x = center_x + (f64::from(x) + 0.5 - radius);
            let src_y = center_y + (f64::from(y) + 0.5 - radius);
            let dx = src_x - center_x;
            let dy = src_y - center_y;
            if dx * dx + dy * dy > r2 {
                continue;
            }
            let ix = src_x.floor() as i32;
            let iy = src_y.floor() as i32;
            if ix < 0 || iy < 0 || ix >= w as i32 || iy >= h as i32 {
                continue;
            }
            let mut px = *source.get_pixel(ix as u32, iy as u32);
            px.0[3] = 255;
            out.put_pixel(x, y, px);
        }
    }
    Some(out)
}

fn luma(pixel: &Rgba<u8>) -> u8 {
    let [r, g, b, _] = pixel.0;
    ((u16::from(r) * 30 + u16::from(g) * 59 + u16::from(b) * 11) / 100) as u8
}

/// Dernier pixel clair en partant du centre, avant le fond noir.
fn edge_from_center(
    image: &RgbaImage,
    origin_x: u32,
    origin_y: u32,
    step_x: i32,
    step_y: i32,
) -> Option<u32> {
    let (w, h) = image.dimensions();
    let mut x = origin_x as i32;
    let mut y = origin_y as i32;
    let mut last_bright = None;
    loop {
        if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
            return last_bright;
        }
        let bright = luma(image.get_pixel(x as u32, y as u32)) >= LIMB_THRESHOLD;
        if bright {
            last_bright = Some(if step_x != 0 { x as u32 } else { y as u32 });
        } else if last_bright.is_some() {
            return last_bright;
        }
        x += step_x;
        y += step_y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caption_below_the_disk_is_removed() {
        let mut source = RgbaImage::new(240, 240);
        let center = 120.0;
        let radius = 80.0;
        for y in 0..240 {
            for x in 0..240 {
                let dx = f64::from(x) - center;
                let dy = f64::from(y) - center;
                if dx * dx + dy * dy <= radius * radius {
                    source.put_pixel(x, y, Rgba([180, 180, 180, 255]));
                }
            }
        }
        for x in 8..100 {
            source.put_pixel(x, 220, Rgba([255, 255, 255, 255]));
        }
        source.put_pixel(120, 220, Rgba([255, 255, 255, 255]));

        let disk = isolate_disk(&source).expect("disque");
        assert_eq!(disk.width(), disk.height());
        assert!((158..=162).contains(&disk.width()), "côté {}", disk.width());
        assert_eq!(disk.get_pixel(0, 0).0[3], 0, "coin transparent");
        let mid = disk.width() / 2;
        let center_px = disk.get_pixel(mid, mid).0;
        assert_eq!(center_px[3], 255);
        assert_eq!(center_px[0], 180);
        assert!(
            disk.pixels().all(|px| px.0[0] != 255),
            "la légende blanche est encore dans le disque"
        );
    }
}
