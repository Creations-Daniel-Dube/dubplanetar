//! Constantes physiques et cartes optionnelles.
//!
//! Les photos, si elles existent, sont des cartes équirectangulaires :
//! nord en haut, longitude 0° au bord gauche, 360° au bord droit.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use image::RgbaImage;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Body {
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Moon,
}

impl Body {
    #[cfg(test)]
    pub const ALL: [Body; 7] = [
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
    ];

    pub fn name_fr(self) -> &'static str {
        match self {
            Body::Mercury => "Mercure",
            Body::Venus => "Vénus",
            Body::Mars => "Mars",
            Body::Jupiter => "Jupiter",
            Body::Saturn => "Saturne",
            Body::Uranus => "Uranus",
            Body::Neptune => "Neptune",
            Body::Moon => "Lune",
        }
    }

    pub fn file_stem(self) -> &'static str {
        match self {
            Body::Mercury => "mercury",
            Body::Venus => "venus",
            Body::Mars => "mars",
            Body::Jupiter => "jupiter",
            Body::Saturn => "saturn",
            Body::Uranus => "uranus",
            Body::Neptune => "neptune",
            Body::Moon => "moon",
        }
    }

    /// Rayon équatorial et polaire, en kilomètres.
    /// Vénus utilise le sommet des nuages : c'est le limbe vu dans un télescope.
    pub fn radii_km(self) -> (f64, f64) {
        match self {
            Body::Mercury => (2439.4, 2439.4),
            Body::Venus => (6120.0, 6120.0),
            Body::Mars => (3396.2, 3376.2),
            Body::Jupiter => (71_492.0, 66_854.0),
            Body::Saturn => (60_268.0, 54_364.0),
            Body::Uranus => (25_559.0, 24_973.0),
            Body::Neptune => (24_764.0, 24_341.0),
            Body::Moon => (1_737.4, 1_737.4),
        }
    }

    pub fn flattening(self) -> f64 {
        let (eq, pol) = self.radii_km();
        ((eq - pol) / eq).max(0.0)
    }

    /// Couleur unie, utilisée quand aucune photo n'est fournie.
    pub fn albedo(self) -> [u8; 3] {
        match self {
            Body::Mercury => [168, 164, 158],
            Body::Venus => [236, 220, 170],
            Body::Mars => [188, 72, 38],
            Body::Jupiter => [214, 178, 130],
            Body::Saturn => [224, 196, 140],
            Body::Uranus => [170, 220, 214],
            Body::Neptune => [70, 110, 230],
            Body::Moon => [186, 186, 180],
        }
    }

    /// Décalage de la carte, en tours. La Lune a sa longitude 0° au centre de l'image.
    pub fn texture_lon_offset(self) -> f64 {
        match self {
            Body::Moon => 0.5,
            _ => 0.0,
        }
    }

    /// Anneaux en rayons équatoriaux. La division de Cassini est le trou entre B et A.
    pub fn ring_zones(self) -> &'static [RingZone] {
        match self {
            Body::Saturn => &SATURN_RINGS,
            _ => &[],
        }
    }

    /// Demi-diagonale du dessin, en rayons équatoriaux (globe, anneaux et axe).
    pub fn canvas_extent(self) -> f64 {
        outer_radius(self.ring_zones()) * CANVAS_MARGIN
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RingZone {
    pub inner: f64,
    pub outer: f64,
    pub brightness: f64,
}

/// Rayons des anneaux de Saturne (C, B, A) rapportés au rayon équatorial 60 268 km.
const SATURN_RINGS: [RingZone; 3] = [
    RingZone {
        inner: 74_658.0 / 60_268.0,
        outer: 92_000.0 / 60_268.0,
        brightness: 0.42,
    },
    RingZone {
        inner: 92_000.0 / 60_268.0,
        outer: 117_580.0 / 60_268.0,
        brightness: 1.0,
    },
    RingZone {
        inner: 122_170.0 / 60_268.0,
        outer: 136_775.0 / 60_268.0,
        brightness: 0.78,
    },
];

pub const CANVAS_MARGIN: f64 = 1.22;
pub const AXIS_REACH: f64 = 1.12;

pub fn outer_radius(zones: &[RingZone]) -> f64 {
    zones.iter().map(|z| z.outer).fold(1.0, f64::max)
}

pub fn texture_file(body: Body) -> Option<PathBuf> {
    for dir in texture_dirs() {
        for ext in ["png", "jpg", "jpeg"] {
            let path = dir.join(format!("{}.{ext}", body.file_stem()));
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

/// `(secondes depuis l'époque unix, taille)` pour le cache. `None` s'il n'y a pas de fichier.
pub fn texture_stamp(body: Body) -> Option<(u64, u64)> {
    let path = texture_file(body)?;
    let meta = std::fs::metadata(&path).ok()?;
    let secs = meta
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some((secs, meta.len()))
}

pub fn load_texture(body: Body) -> Option<RgbaImage> {
    let path = texture_file(body)?;
    load_path(&path).or_else(|| {
        eprintln!("Photo illisible : {}", path.display());
        None
    })
}

/// Hauteurs normalisées. 0 est le bas, 65535 le haut.
#[derive(Clone)]
pub struct HeightMap {
    width: u32,
    height: u32,
    samples: Vec<u16>,
}

impl HeightMap {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    #[cfg(test)]
    pub(crate) fn from_samples(width: u32, height: u32, samples: Vec<u16>) -> Self {
        Self {
            width,
            height,
            samples,
        }
    }

    /// Hauteur bilinéaire dans `[0, 1]`. `u` boucle, `v` est borné aux pôles.
    pub fn sample(&self, u: f64, v: f64) -> f64 {
        let w = self.width as f64;
        let h = self.height as f64;
        if w < 1.0 || h < 1.0 {
            return 0.0;
        }
        let x = u.rem_euclid(1.0) * w - 0.5;
        let y = v.clamp(0.0, 1.0) * h - 0.5;
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let tx = (x - f64::from(x0)).clamp(0.0, 1.0);
        let ty = (y - f64::from(y0)).clamp(0.0, 1.0);
        let h00 = self.texel(x0, y0);
        let h10 = self.texel(x0 + 1, y0);
        let h01 = self.texel(x0, y0 + 1);
        let h11 = self.texel(x0 + 1, y0 + 1);
        let top = h00 * (1.0 - tx) + h10 * tx;
        let bottom = h01 * (1.0 - tx) + h11 * tx;
        top * (1.0 - ty) + bottom * ty
    }

    fn texel(&self, x: i32, y: i32) -> f64 {
        let w = self.width as i32;
        let h = self.height as i32;
        let xx = x.rem_euclid(w.max(1)) as u32;
        let yy = y.clamp(0, h - 1) as u32;
        f64::from(self.samples[(yy * self.width + xx) as usize]) / f64::from(u16::MAX)
    }
}

pub fn bump_file(body: Body) -> Option<PathBuf> {
    for dir in texture_dirs() {
        for ext in ["png", "jpg", "jpeg"] {
            let path = dir.join(format!("{}_bump.{ext}", body.file_stem()));
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

pub fn bump_stamp(body: Body) -> Option<(u64, u64)> {
    let path = bump_file(body)?;
    let meta = std::fs::metadata(&path).ok()?;
    let secs = meta
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some((secs, meta.len()))
}

pub fn load_bump_file(path: &Path) -> Option<HeightMap> {
    let img = image::open(path).ok()?;
    let gray = img.to_luma16();
    let (width, height) = gray.dimensions();
    if width == 0 || height == 0 {
        return None;
    }
    Some(HeightMap {
        width,
        height,
        samples: gray.into_raw(),
    })
}

pub(crate) fn load_path(path: &Path) -> Option<RgbaImage> {
    image::open(path).ok().map(|img| img.to_rgba8())
}

fn texture_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/textures"));
    if let Ok(cwd) = std::env::current_dir() {
        let dir = cwd.join("assets/textures");
        if dirs.iter().all(|d| d != &dir) {
            dirs.push(dir);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let dir = parent.join("assets/textures");
            if dirs.iter().all(|d| d != &dir) {
                dirs.push(dir);
            }
        }
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_file_yields_none() {
        let path = Path::new("assets/textures/__absent__.png");
        assert!(load_path(path).is_none());
    }

    #[test]
    fn stamp_tracks_presence() {
        let present = texture_file(Body::Mercury).is_some();
        assert_eq!(texture_stamp(Body::Mercury).is_some(), present);
    }

    #[test]
    fn bump_stamp_tracks_presence() {
        let present = bump_file(Body::Moon).is_some();
        assert_eq!(bump_stamp(Body::Moon).is_some(), present);
    }
}
