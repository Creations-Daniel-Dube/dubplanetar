//! Luminosité d'un télescope de 305 mm et ciel derrière l'objet.
//!
//! L'ouverture collecte environ 1 800 fois plus de lumière qu'une pupille de 7 mm.
//! Mercure et Vénus saturent : disque blanc, halo large, limbe fondu. Jupiter,
//! Saturne et la Lune restent très clairs, avec assez de contraste pour les bandes
//! et les mers. Uranus reste un disque vert, Neptune un disque bleu plus sombre.
//! Le Soleil garde la couleur de sa photo sous une auréole très claire.

use chrono::{DateTime, Local};
use image::{RgbaImage, imageops::rotate180};

use crate::bodies::Body;
use crate::ephemeris::{self, Appearance};
use crate::state::Viewpoint;

/// Brillance de surface de référence, en mag/″², voisine du disque de Neptune.
const SB_REF: f64 = 9.5;
/// Gain d'écran à cette référence. 0,22 laissait Uranus et Neptune trop petits et noirs.
const GAIN_AT_REF: f64 = 1.50;
/// Au-dessus de 1, les objets brillants montent plus vite que le flux.
/// Le contraste d'albédo est rendu plus bas, dans la courbe, pas ici.
const GAIN_GAMMA: f64 = 1.40;
/// Site du ciel terrestre. La longitude vient du fuseau de l'heure locale.
const SITE_LATITUDE_DEG: f64 = 45.0;
/// Distance moyenne de la Lune, en UA.
const MEAN_MOON_AU: f64 = 384_400.0 / 149_597_870.7;
/// Au-delà, un point de satellite disparaît, comme à l'oculaire.
const POINT_MAG_LIMIT: f64 = 16.0;

/// Fond immédiatement derrière un disque.
#[derive(Clone, Copy, Debug)]
pub struct Sky {
    pub color: egui::Color32,
    /// Assez clair pour poser les textes sur une plaque sombre.
    pub bright: bool,
}

/// Magnitude visuelle. `r_au` est la distance au Soleil, `delta_au` celle à l'observateur.
///
/// Mars à Neptune : Meeus. Mercure, Vénus et la Terre : Mallama. Lune : Allen,
/// plus le terme de distance. `r_au` n'entre pas dans la formule de la Lune.
pub fn visual_magnitude(body: Body, r_au: f64, delta_au: f64, phase_deg: f64, lat_deg: f64) -> f64 {
    let base = 5.0 * (r_au * delta_au).max(1e-12).log10();
    match body {
        Body::Mercury => base + phase_poly(phase_deg, &MERCURY_PHASE),
        Body::Venus => {
            if phase_deg < 163.6 {
                base + phase_poly(phase_deg, &VENUS_PHASE)
            } else {
                base + phase_poly(phase_deg, &VENUS_CRESCENT)
            }
        }
        Body::Earth => base + phase_poly(phase_deg, &EARTH_PHASE),
        Body::Moon => {
            let alpha = phase_deg.abs();
            -12.73
                + 0.026 * alpha
                + 4.0e-9 * alpha.powi(4)
                + 5.0 * (delta_au / MEAN_MOON_AU).max(1e-12).log10()
        }
        Body::Mars => -1.52 + base + 0.016 * phase_deg,
        Body::Jupiter => -9.40 + base + 0.005 * phase_deg,
        Body::Saturn => {
            let b = lat_deg.to_radians().sin().abs();
            -8.88 + base + 0.044 * phase_deg - 2.60 * b + 1.25 * b * b
        }
        Body::Uranus => -7.19 + base,
        Body::Neptune => -6.87 + base,
    }
}

/// Mallama : termes du polynôme de phase, de la constante jusqu'à la plus haute puissance.
const MERCURY_PHASE: [f64; 7] = [
    -0.613,
    6.3280e-2,
    -1.6336e-3,
    3.3644e-5,
    -3.4265e-7,
    1.6893e-9,
    -3.0334e-12,
];
const VENUS_PHASE: [f64; 5] = [-4.384, 1.044e-3, 3.687e-4, -2.814e-6, 8.938e-9];
const VENUS_CRESCENT: [f64; 3] = [236.05828, -2.81914, 8.39034e-3];
const EARTH_PHASE: [f64; 3] = [-3.99, -1.060e-3, 2.054e-4];

fn phase_poly(alpha: f64, terms: &[f64]) -> f64 {
    let mut acc = 0.0;
    let mut power = 1.0;
    for term in terms {
        acc += term * power;
        power *= alpha;
    }
    acc
}

/// Magnitude du disque à l'instant `jd`. La distance au Soleil est l'héliocentrique du moment.
pub fn magnitude_of(body: Body, app: &Appearance, jd: f64) -> f64 {
    let r_au = if body == Body::Moon {
        1.0
    } else {
        ephemeris::heliocentric(body, jd).length().max(1e-9)
    };
    visual_magnitude(
        body,
        r_au,
        app.distance_au,
        app.phase_angle_rad.to_degrees(),
        app.sub_observer_lat_rad.to_degrees(),
    )
}

/// Brillance moyenne de la partie éclairée, en mag/″².
pub fn surface_brightness(magnitude: f64, diameter_arcsec: f64, illuminated: f64) -> f64 {
    let radius = (diameter_arcsec * 0.5).max(1e-4);
    let area = illuminated.clamp(1e-4, 1.0) * std::f64::consts::PI * radius * radius;
    magnitude + 2.5 * area.max(1e-12).log10()
}

/// Multiplicateur appliqué au pixel linéaire (0–1) avant l'épaule.
pub fn disk_gain(surface_brightness: f64) -> f64 {
    let ratio = 10_f64.powf(-0.4 * (surface_brightness - SB_REF));
    GAIN_AT_REF * ratio.powf(GAIN_GAMMA)
}

/// Gain du disque pour le mode réel.
pub fn gain_for(body: Body, app: &Appearance, jd: f64) -> f64 {
    let mag = magnitude_of(body, app, jd);
    disk_gain(surface_brightness(
        mag,
        app.eq_diameter_arcsec,
        app.illuminated,
    ))
}

/// Éclat d'un point (satellite) sur la même exposition que les disques.
/// 0,35 à la magnitude 8. Au-delà de 16, rien.
pub fn real_point_gain(magnitude: f64) -> f64 {
    if magnitude > POINT_MAG_LIMIT {
        return 0.0;
    }
    let ratio = 10_f64.powf(-0.4 * (magnitude - 8.0));
    0.35 * ratio.powf(GAIN_GAMMA)
}

pub fn quantize_gain(gain: f64) -> i32 {
    (gain * 100.0).round().clamp(0.0, 100_000.0) as i32
}

/// Teinte ajoutée en vision réelle. Le neutre garde la photo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cast {
    Neutral,
    /// Mars : un peu de rose, sans effacer les zones sombres.
    Pink,
    /// Uranus.
    Green,
    /// Neptune.
    Blue,
}

pub fn cast_of(body: Body) -> Cast {
    match body {
        Body::Mars => Cast::Pink,
        Body::Uranus => Cast::Green,
        Body::Neptune => Cast::Blue,
        _ => Cast::Neutral,
    }
}

/// Halo autour du disque. `Wash` est le reflet aveuglant de Mercure et Vénus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glare {
    None,
    /// Halo modéré. Le disque garde ses bandes ou ses mers.
    Soft,
    /// Grand halo et limbe fondu dans le ciel.
    Wash,
}

pub fn glare_of(body: Body, gain: f64, illuminated: f64) -> Glare {
    if gain <= 1.2 || illuminated < 0.02 {
        return Glare::None;
    }
    match body {
        Body::Mercury | Body::Venus => Glare::Wash,
        Body::Uranus | Body::Neptune => Glare::None,
        _ => Glare::Soft,
    }
}

/// `None` : pixels d'origine. Sinon courbe réelle, teinte, puis frange de limbe.
pub fn present_disk(
    north_up: &RgbaImage,
    north_down: bool,
    gain: Option<f64>,
    cast: Cast,
    glare: Glare,
) -> egui::ColorImage {
    let owned;
    let image = if north_down {
        owned = rotate180(north_up);
        &owned
    } else {
        north_up
    };
    let size = [image.width() as usize, image.height() as usize];
    let Some(gain) = gain else {
        return egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    };
    let mut pixels = image.as_raw().clone();
    for px in pixels.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        let graded = grade_color([px[0], px[1], px[2]], gain, cast);
        px[0] = graded[0];
        px[1] = graded[1];
        px[2] = graded[2];
    }
    limb_bloom(&mut pixels, size[0], size[1], glare);
    egui::ColorImage::from_rgba_unmultiplied(size, &pixels)
}

pub fn tone_channel(channel: u8, gain: f64) -> u8 {
    let y = tone_luma(f64::from(channel) / 255.0, gain, 0.0);
    (y * 255.0).round().clamp(0.0, 255.0) as u8
}

pub fn tone_rgb(color: [u8; 3], gain: f64) -> egui::Color32 {
    let graded = grade_color(color, gain, Cast::Neutral);
    egui::Color32::from_rgb(graded[0], graded[1], graded[2])
}

/// Canal gris, ou couleur graduée. Sert aux tests et aux points de satellites.
pub fn grade_color(color: [u8; 3], gain: f64, cast: Cast) -> [u8; 3] {
    let lin = [
        f64::from(color[0]) / 255.0,
        f64::from(color[1]) / 255.0,
        f64::from(color[2]) / 255.0,
    ];
    let luma = rec709(lin);
    if luma < 1e-8 || gain <= 0.0 {
        return [0, 0, 0];
    }
    let spec = cast_spec(cast);
    let max_channel = lin.iter().copied().fold(0.0_f64, f64::max);
    let signal = luma * (1.0 - spec.max_blend) + max_channel * spec.max_blend;
    let ratios = if let Some(toward) = spec.toward {
        let target = rec709(toward);
        let mut mixed = [0.0; 3];
        for i in 0..3 {
            let current = lin[i] / luma;
            let goal = toward[i] / target;
            mixed[i] = current * (1.0 - spec.mix) + goal * spec.mix;
        }
        mixed
    } else {
        [lin[0] / luma, lin[1] / luma, lin[2] / luma]
    };
    let used_gain = gain * spec.gain_mul;
    let mut sat = spec.sat;
    if cast == Cast::Neutral {
        let preview = tone_luma(signal, used_gain, 0.0);
        let fade = ((preview - 0.80) / 0.20).clamp(0.0, 1.0);
        sat = (0.40 - 0.22 * fade).max(0.18);
    }
    let new_luma = tone_luma(signal, used_gain, spec.extra_room);
    let mut raw = [0.0; 3];
    for i in 0..3 {
        raw[i] = new_luma * (1.0 + (ratios[i] - 1.0) * sat);
    }
    let peak = raw.into_iter().fold(0.0_f64, f64::max);
    if peak > 1.0 && new_luma < 0.999 {
        let fit = (1.0 - new_luma) / (peak - new_luma).max(1e-6);
        let fit = fit.min(1.0);
        for channel in &mut raw {
            *channel = new_luma + (*channel - new_luma) * fit;
        }
    }
    raw.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)
}

struct CastSpec {
    toward: Option<[f64; 3]>,
    mix: f64,
    sat: f64,
    gain_mul: f64,
    max_blend: f64,
    extra_room: f64,
}

fn cast_spec(cast: Cast) -> CastSpec {
    match cast {
        Cast::Neutral => CastSpec {
            toward: None,
            mix: 0.0,
            sat: 0.38,
            gain_mul: 1.0,
            max_blend: 0.0,
            extra_room: 0.0,
        },
        Cast::Pink => CastSpec {
            toward: Some([1.0, 0.68, 0.60]),
            mix: 0.30,
            sat: 0.58,
            gain_mul: 1.0,
            max_blend: 0.62,
            extra_room: 0.14,
        },
        Cast::Green => CastSpec {
            toward: Some([0.40, 1.0, 0.55]),
            mix: 0.60,
            sat: 0.90,
            gain_mul: 1.55,
            max_blend: 0.0,
            extra_room: 0.0,
        },
        Cast::Blue => CastSpec {
            toward: Some([0.26, 0.48, 1.0]),
            mix: 0.66,
            sat: 1.0,
            gain_mul: 3.6,
            max_blend: 0.22,
            extra_room: 0.0,
        },
    }
}

/// Niveau d'un albédo. Le pic suit le gain ; au blanc, une fenêtre reste ouverte
/// pour les bandes, les mers et les taches.
fn tone_luma(x: f64, gain: f64, extra_room: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.0 || gain <= 0.0 {
        return 0.0;
    }
    let peak = peak_of(gain);
    let hot = ((peak - 0.73) / 0.26).clamp(0.0, 1.0);
    let room = (peak * (1.0 - hot) + (0.44 + extra_room) * hot).min(peak);
    let floor = (peak - room).max(0.0);
    let gamma = 1.0 + 0.72 * hot;
    let gate = smoothstep(0.012, 0.10, x);
    ((floor + room * x.powf(gamma)) * gate).clamp(0.0, 1.0)
}

fn peak_of(gain: f64) -> f64 {
    if gain <= 0.0 {
        return 0.0;
    }
    let slow = 0.54 + 0.098 * gain.max(0.05).ln();
    let t = ((gain - 7.0) / 22.0).clamp(0.0, 1.0);
    let w = t * t * (3.0 - 2.0 * t);
    (slow * (1.0 - w) + 0.992 * w).clamp(0.0, 0.992)
}

fn rec709(rgb: [f64; 3]) -> f64 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

/// Halo derrière le disque. Le limbe lui-même est adouci dans `present_disk`.
pub fn paint_glare(
    painter: &egui::Painter,
    ctx: &egui::Context,
    center: egui::Pos2,
    radius: f32,
    gain: f64,
    illuminated: f64,
    glare: Glare,
) {
    if glare == Glare::None || radius < 1.0 || illuminated < 0.02 {
        return;
    }
    let lit = illuminated.sqrt().clamp(0.0, 1.0);
    match glare {
        Glare::Wash => {
            let outer = (radius * 5.6).max(56.0);
            let wide = (205.0 * lit).round().clamp(0.0, 255.0) as u8;
            paint_soft_disc(
                painter,
                ctx,
                center,
                outer,
                egui::Color32::from_rgba_unmultiplied(255, 236, 200, wide),
            );
            let core = (radius * 2.35).max(30.0);
            let hot = (255.0 * lit).round().clamp(0.0, 255.0) as u8;
            paint_soft_disc(
                painter,
                ctx,
                center,
                core,
                egui::Color32::from_rgba_unmultiplied(255, 252, 238, hot),
            );
        }
        Glare::Soft => {
            let spread = (2.35 + 0.36 * (gain.max(1.0) as f32).ln()).clamp(2.35, 4.2);
            let alpha = (200.0 * lit).round().clamp(0.0, 255.0) as u8;
            paint_soft_disc(
                painter,
                ctx,
                center,
                radius * spread,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
            );
        }
        Glare::None => {}
    }
}

/// Auréole derrière la photo du Soleil. Le disque reste par-dessus.
pub fn paint_sun_aureole(
    painter: &egui::Painter,
    ctx: &egui::Context,
    center: egui::Pos2,
    radius: f32,
) {
    if radius < 1.0 {
        return;
    }
    paint_soft_disc(
        painter,
        ctx,
        center,
        radius * 7.2,
        egui::Color32::from_rgba_unmultiplied(255, 214, 150, 210),
    );
    paint_soft_disc(
        painter,
        ctx,
        center,
        radius * 3.6,
        egui::Color32::from_rgba_unmultiplied(255, 236, 190, 235),
    );
    paint_soft_disc(
        painter,
        ctx,
        center,
        radius * 1.85,
        egui::Color32::from_rgba_unmultiplied(255, 250, 230, 255),
    );
}

/// Voile blanc chaud sur la photo. Le centre brûle, l'orange reste vers le limbe.
pub fn paint_sun_veil(
    painter: &egui::Painter,
    ctx: &egui::Context,
    center: egui::Pos2,
    radius: f32,
) {
    if radius < 1.0 {
        return;
    }
    paint_soft_disc(
        painter,
        ctx,
        center,
        radius * 1.22,
        egui::Color32::from_rgba_unmultiplied(255, 252, 240, 188),
    );
}

/// Disque dont l'alpha suit une gaussienne, du centre vers le bord.
pub fn paint_soft_disc(
    painter: &egui::Painter,
    ctx: &egui::Context,
    center: egui::Pos2,
    radius: f32,
    tint: egui::Color32,
) {
    if radius < 1.0 || tint.a() == 0 {
        return;
    }
    let tex = glow_texture(ctx);
    let side = radius * 2.0;
    painter.image(
        tex.id(),
        egui::Rect::from_center_size(center, egui::vec2(side, side)),
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        tint,
    );
}

/// Frange claire dans la marge transparente. `Wash` voile aussi la face éclairée.
fn limb_bloom(pixels: &mut [u8], width: usize, height: usize, glare: Glare) {
    let (frac, outside, onto) = match glare {
        Glare::None => return,
        Glare::Soft => (0.030_f32, 0.95_f32, 0.0_f32),
        Glare::Wash => (0.078_f32, 1.10_f32, 0.40_f32),
    };
    if width < 4 || height < 4 {
        return;
    }
    let dw = width.min(128);
    let dh = (height * dw / width).clamp(4, 160);
    let mut small = vec![0.0_f32; dw * dh];
    for y in 0..height {
        let sy = y * dh / height;
        let row = y * width;
        for x in 0..width {
            let i = (row + x) * 4;
            if pixels[i + 3] == 0 {
                continue;
            }
            let lum = (0.2126 * f32::from(pixels[i])
                + 0.7152 * f32::from(pixels[i + 1])
                + 0.0722 * f32::from(pixels[i + 2]))
                / 255.0;
            let sx = x * dw / width;
            let k = sy * dw + sx;
            if lum > small[k] {
                small[k] = lum;
            }
        }
    }
    let radius = (frac * dw as f32).round().clamp(1.0, 26.0) as i32;
    box_blur(&mut small, dw, dh, radius);
    box_blur(&mut small, dw, dh, radius);

    for y in 0..height {
        let fy = (y as f32 + 0.5) * dh as f32 / height as f32 - 0.5;
        for x in 0..width {
            let fx = (x as f32 + 0.5) * dw as f32 / width as f32 - 0.5;
            let b = sample_blur(&small, dw, dh, fx, fy);
            if b < 0.02 {
                continue;
            }
            let i = (y * width + x) * 4;
            if pixels[i + 3] == 0 {
                let dither = f32::from(BAYER[(y & 7) * 8 + (x & 7)]) / 64.0 - 0.5;
                let alpha = (b * outside * 255.0 + dither).round().clamp(0.0, 255.0) as u8;
                if alpha == 0 {
                    continue;
                }
                pixels[i] = 255;
                pixels[i + 1] = 250;
                pixels[i + 2] = 236;
                pixels[i + 3] = alpha;
            } else if onto > 0.0 {
                let lum = 0.2126 * f32::from(pixels[i])
                    + 0.7152 * f32::from(pixels[i + 1])
                    + 0.0722 * f32::from(pixels[i + 2]);
                if lum < 12.0 {
                    continue;
                }
                let add = (b * onto).clamp(0.0, 1.0);
                for c in 0..3 {
                    let s = f32::from(pixels[i + c]) / 255.0;
                    let out = s + (1.0 - s) * add;
                    pixels[i + c] = (out * 255.0).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
}

fn box_blur(buf: &mut [f32], w: usize, h: usize, radius: i32) {
    blur_axis(buf, w, h, radius, true);
    blur_axis(buf, w, h, radius, false);
}

fn blur_axis(buf: &mut [f32], w: usize, h: usize, radius: i32, horizontal: bool) {
    let n = if horizontal { w } else { h };
    let lines = if horizontal { h } else { w };
    let mut prefix = vec![0.0_f32; n + 1];
    let mut tmp = vec![0.0_f32; n];
    for line in 0..lines {
        prefix[0] = 0.0;
        for i in 0..n {
            let (x, y) = if horizontal { (i, line) } else { (line, i) };
            prefix[i + 1] = prefix[i] + buf[y * w + x];
        }
        for i in 0..n {
            let a = (i as i32 - radius).max(0) as usize;
            let b = (i as i32 + radius + 1).min(n as i32) as usize;
            tmp[i] = (prefix[b] - prefix[a]) / (b - a) as f32;
        }
        for i in 0..n {
            let (x, y) = if horizontal { (i, line) } else { (line, i) };
            buf[y * w + x] = tmp[i];
        }
    }
}

fn sample_blur(buf: &[f32], w: usize, h: usize, x: f32, y: f32) -> f32 {
    let x = x.clamp(0.0, (w - 1) as f32);
    let y = y.clamp(0.0, (h - 1) as f32);
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;
    let at = |xx: usize, yy: usize| buf[yy * w + xx];
    let top = at(x0, y0) * (1.0 - tx) + at(x1, y0) * tx;
    let bottom = at(x0, y1) * (1.0 - tx) + at(x1, y1) * tx;
    top * (1.0 - ty) + bottom * ty
}

fn glow_texture(ctx: &egui::Context) -> egui::TextureHandle {
    let id = egui::Id::unique("vision-gaussian-glow");
    if let Some(tex) = ctx.data(|data| data.get_temp::<egui::TextureHandle>(id)) {
        return tex;
    }
    let tex = ctx.load_texture(
        "vision-gaussian-glow",
        gaussian_image(),
        egui::TextureOptions::LINEAR,
    );
    ctx.data_mut(|data| data.insert_temp(id, tex.clone()));
    tex
}

fn gaussian_image() -> egui::ColorImage {
    const N: usize = 512;
    let mut raw = vec![0_u8; N * N * 4];
    let half = (N as f32 - 1.0) * 0.5;
    let sigma = half * 0.46;
    let inv = 1.0 / (2.0 * sigma * sigma);
    for y in 0..N {
        for x in 0..N {
            let dx = x as f32 - half;
            let dy = y as f32 - half;
            let g = (-(dx * dx + dy * dy) * inv).exp();
            let dither = f32::from(BAYER[(y & 7) * 8 + (x & 7)]) / 64.0 - 0.5;
            let a = (g * 255.0 + dither).round().clamp(0.0, 255.0) as u8;
            let i = (y * N + x) * 4;
            raw[i] = 255;
            raw[i + 1] = 255;
            raw[i + 2] = 255;
            raw[i + 3] = a;
        }
    }
    egui::ColorImage::from_rgba_unmultiplied([N, N], &raw)
}

/// Trame 8×8, 0…63, pour casser les paliers quand la texture est étirée.
const BAYER: [u8; 64] = [
    0, 32, 8, 40, 2, 34, 10, 42, 48, 16, 56, 24, 50, 18, 58, 26, 12, 44, 4, 36, 14, 46, 6, 38, 60,
    28, 52, 20, 62, 30, 54, 22, 3, 35, 11, 43, 1, 33, 9, 41, 51, 19, 59, 27, 49, 17, 57, 25, 15, 47,
    7, 39, 13, 45, 5, 37, 63, 31, 55, 23, 61, 29, 53, 21,
];

/// Longitude est, en degrés, déduite du fuseau de l'heure locale (15° par heure).
pub fn longitude_deg(when: DateTime<Local>) -> f64 {
    f64::from(when.offset().local_minus_utc()) / 3600.0 * 15.0
}

/// Ciel terrestre derrière `body`. `None` si l'observateur n'est pas la Terre
/// (la Lune reste géocentrique et garde ce ciel).
pub fn sky_behind(
    body: Body,
    viewpoint: Viewpoint,
    app: &Appearance,
    jd: f64,
    lon_deg: f64,
) -> Option<Sky> {
    if viewpoint != Viewpoint::Earth && body != Body::Moon {
        return None;
    }
    let (sun_ra, sun_dec) = ephemeris::sun_radec(jd);
    let alt = altitude_deg(jd, lon_deg, sun_ra, sun_dec);
    let elong = elongation_deg(app.ra_rad, app.dec_rad, sun_ra, sun_dec);
    Some(terrestrial_sky(alt, elong))
}

pub fn terrestrial_sky(sun_alt_deg: f64, elongation_deg: f64) -> Sky {
    let night = [4.0, 8.0, 20.0];
    let deep = [16.0, 28.0, 64.0];
    let day_blue = [122.0, 170.0, 216.0];
    let day_white = [238.0, 242.0, 246.0];
    let dusk = [255.0, 156.0, 78.0];

    let day_up = smoothstep(0.0, 10.0, sun_alt_deg);
    let twilight_fill = smoothstep(-18.0, -1.0, sun_alt_deg) * (1.0 - day_up);
    let elong = elongation_deg.clamp(0.0, 180.0);
    let near = (-(elong / 24.0).powi(2)).exp();
    let glow = near * (-((sun_alt_deg + 4.0) / 7.5).powi(2)).exp();

    let mut rgb = night;
    rgb = mix(rgb, deep, twilight_fill * 0.9);
    rgb = mix(rgb, day_blue, day_up * (1.0 - 0.72 * near));
    rgb = mix(rgb, day_white, day_up * near);
    rgb = mix(rgb, dusk, glow);

    let luma = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
    Sky {
        color: rgb_u8(rgb),
        bright: luma > 100.0,
    }
}

/// Hauteur du Soleil, degrés. `lon_deg` est est-positive.
pub fn altitude_deg(jd: f64, lon_deg: f64, ra: f64, dec: f64) -> f64 {
    let lst = gmst_rad(jd) + lon_deg.to_radians();
    let ha = lst - ra;
    altitude_from_hour_angle(SITE_LATITUDE_DEG, dec, ha)
}

pub fn altitude_from_hour_angle(lat_deg: f64, dec: f64, hour_angle: f64) -> f64 {
    let lat = lat_deg.to_radians();
    let sin_alt = lat.sin() * dec.sin() + lat.cos() * dec.cos() * hour_angle.cos();
    sin_alt.clamp(-1.0, 1.0).asin().to_degrees()
}

pub fn elongation_deg(ra1: f64, dec1: f64, ra2: f64, dec2: f64) -> f64 {
    let cos_e = dec1.sin() * dec2.sin() + dec1.cos() * dec2.cos() * (ra1 - ra2).cos();
    cos_e.clamp(-1.0, 1.0).acos().to_degrees()
}

fn gmst_rad(jd: f64) -> f64 {
    let d = jd - 2_451_545.0;
    let t = d / 36_525.0;
    let deg =
        280.460_618_37 + 360.985_647_366_29 * d + 0.000_387_933 * t * t - t * t * t / 38_710_000.0;
    deg.rem_euclid(360.0).to_radians()
}

fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn rgb_u8(rgb: [f64; 3]) -> egui::Color32 {
    let ch = |c: f64| c.round().clamp(0.0, 255.0) as u8;
    egui::Color32::from_rgb(ch(rgb[0]), ch(rgb[1]), ch(rgb[2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luma(color: egui::Color32) -> f64 {
        0.2126 * f64::from(color.r()) + 0.7152 * f64::from(color.g()) + 0.0722 * f64::from(color.b())
    }

    fn luma_rgb(color: [u8; 3]) -> f64 {
        luma(egui::Color32::from_rgb(color[0], color[1], color[2]))
    }

    #[test]
    fn venus_is_near_minus_four_and_far_brighter_than_neptune() {
        let venus = visual_magnitude(Body::Venus, 0.723, 1.0, 40.0, 0.0);
        let neptune = visual_magnitude(Body::Neptune, 30.0, 29.0, 0.0, 0.0);
        assert!(
            (-5.2..-3.5).contains(&venus),
            "Vénus {venus}, attendu près de −4"
        );
        assert!(
            (7.5..8.3).contains(&neptune),
            "Neptune {neptune}, attendu près de +8"
        );
        let venus_gain = disk_gain(surface_brightness(venus, 25.0, 0.85));
        let neptune_gain = disk_gain(surface_brightness(neptune, 2.3, 1.0));
        assert!(
            venus_gain > neptune_gain * 8.0,
            "gains {venus_gain} et {neptune_gain}"
        );
    }

    #[test]
    fn uranus_stays_brighter_than_neptune() {
        let uranus = visual_magnitude(Body::Uranus, 19.2, 18.2, 0.0, 0.0);
        let neptune = visual_magnitude(Body::Neptune, 30.0, 29.0, 0.0, 0.0);
        let uranus_gain = disk_gain(surface_brightness(uranus, 3.7, 1.0));
        let neptune_gain = disk_gain(surface_brightness(neptune, 2.3, 1.0));
        assert!(uranus < neptune);
        assert!(uranus_gain > neptune_gain);
    }

    #[test]
    fn full_moon_gain_exceeds_a_dim_planet() {
        let moon = visual_magnitude(Body::Moon, 1.0, MEAN_MOON_AU, 0.0, 0.0);
        assert!(moon < -12.0, "{moon}");
        let moon_gain = disk_gain(surface_brightness(moon, 1_800.0, 1.0));
        let neptune = disk_gain(surface_brightness(7.8, 2.3, 1.0));
        assert!(moon_gain > neptune * 4.0, "{moon_gain} {neptune}");
    }

    #[test]
    fn night_sky_is_black_and_daytime_near_the_sun_is_bright() {
        let night = terrestrial_sky(-40.0, 170.0);
        let day = terrestrial_sky(42.0, 8.0);
        let dusk = terrestrial_sky(-5.0, 15.0);
        assert!(luma(night.color) < 30.0, "nuit {}", luma(night.color));
        assert!(!night.bright);
        assert!(luma(day.color) > 150.0, "jour {}", luma(day.color));
        assert!(day.bright);
        assert!(luma(dusk.color) > luma(night.color) + 40.0);
    }

    #[test]
    fn elongation_and_meridian_altitude() {
        let same = elongation_deg(0.2, -0.3, 0.2, -0.3);
        assert!(same.abs() < 1e-6);
        let opposite = elongation_deg(0.0, 0.0, std::f64::consts::PI, 0.0);
        assert!((opposite - 180.0).abs() < 1e-6);
        let alt = altitude_from_hour_angle(45.0, 0.0, 0.0);
        assert!((alt - 45.0).abs() < 1e-6, "{alt}");
    }

    #[test]
    fn gmst_at_j2000_matches_meeus() {
        let deg = gmst_rad(2_451_545.0).to_degrees();
        assert!((deg - 280.460_618_37).abs() < 1e-6, "{deg}");
    }

    #[test]
    fn jupiter_stays_bright_and_keeps_band_contrast() {
        let jupiter = disk_gain(5.4);
        let zone = tone_channel(220, jupiter);
        let band = tone_channel(130, jupiter);
        assert!(zone > 215, "zone {zone} gain {jupiter}");
        assert!(band > 155, "bande {band}");
        assert!(zone - band > 28, "contraste {zone} contre {band}");
        let neptune = disk_gain(9.4);
        let neptune_px = grade_color([70, 110, 230], neptune, Cast::Blue);
        let neptune_l = luma(egui::Color32::from_rgb(
            neptune_px[0],
            neptune_px[1],
            neptune_px[2],
        ));
        assert!(zone as f64 > neptune_l + 80.0, "{zone} vs {neptune_l}");
    }

    #[test]
    fn moon_keeps_maria_darker_than_the_highlands() {
        let moon = disk_gain(3.3);
        let high = tone_channel(220, moon);
        let mare = tone_channel(110, moon);
        assert!(high > 220, "hautes terres {high}");
        assert!((160..215).contains(&mare), "mer {mare}");
        assert!(high - mare > 30, "{high} contre {mare}");
    }

    #[test]
    fn mars_stays_pink_with_darker_patches() {
        let mars = disk_gain(4.6);
        let bright = grade_color([188, 72, 38], mars, Cast::Pink);
        let dark = grade_color([85, 34, 18], mars, Cast::Pink);
        assert!(bright[0] > bright[1] + 25, "rose {bright:?}");
        assert!(bright[1] > bright[2], "rose {bright:?}");
        assert!(bright[0] > 220, "rose {bright:?}");
        assert!(luma_rgb(bright) > luma_rgb(dark) + 30.0, "{bright:?} {dark:?}");
    }

    #[test]
    fn uranus_is_green_and_neptune_is_a_dimmer_blue() {
        let uranus = grade_color([170, 220, 214], disk_gain(8.1), Cast::Green);
        let neptune = grade_color([70, 110, 230], disk_gain(9.4), Cast::Blue);
        assert!(uranus[1] > uranus[0] + 20, "vert {uranus:?}");
        assert!(uranus[1] > uranus[2], "vert {uranus:?}");
        assert!(neptune[2] > neptune[1] + 40, "bleu {neptune:?}");
        assert!(neptune[2] > 160, "bleu {neptune:?}");
        let uranus_l = luma_rgb(uranus);
        let neptune_l = luma_rgb(neptune);
        assert!(
            (90.0..170.0).contains(&neptune_l),
            "Neptune {neptune_l} {neptune:?}"
        );
        assert!(
            uranus_l > neptune_l + 30.0,
            "Uranus {uranus_l} Neptune {neptune_l}"
        );
    }

    #[test]
    fn wash_feathers_the_lit_limb_and_leaves_the_night_dark() {
        let side = 48u32;
        let mut image = RgbaImage::new(side, side);
        let center = f64::from(side) / 2.0;
        let radius = 14.0;
        for y in 0..side {
            for x in 0..side {
                let dx = f64::from(x) - center;
                let dy = f64::from(y) - center;
                if dx * dx + dy * dy <= radius * radius {
                    let rgb = if dx > 1.0 {
                        [230, 220, 200]
                    } else {
                        [0, 0, 0]
                    };
                    image.put_pixel(x, y, image::Rgba([rgb[0], rgb[1], rgb[2], 255]));
                }
            }
        }
        let out = present_disk(&image, false, Some(80.0), Cast::Neutral, Glare::Wash);
        let at = |x: i32, y: i32| out.pixels[(y as usize) * side as usize + x as usize];
        let outside = at(40, 24);
        assert!(outside.a() > 15, "frange alpha {}", outside.a());
        let night = at(14, 24);
        assert!(luma(night) < 50.0, "nuit {}", luma(night));
    }

    #[test]
    fn tone_curve_lifts_a_bright_disk_and_keeps_a_dim_one() {
        let bright = tone_channel(220, 12.0);
        let dim = tone_channel(220, 0.22);
        assert!(bright > 165, "{bright}");
        assert!(dim < 130, "{dim}");
        assert!(bright > dim + 80, "{bright} {dim}");
        assert_eq!(tone_channel(0, 80.0), 0);
    }
}
