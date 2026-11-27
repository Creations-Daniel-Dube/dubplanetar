//! Luminosité d'un télescope de 305 mm et ciel derrière l'objet.
//!
//! L'ouverture collecte environ 1 800 fois plus de lumière qu'une pupille de 7 mm.
//! Ce gain fixe le zéro d'exposition : Neptune reste un disque lisible. L'écran
//! ne peut pas suivre le flux en linéaire, donc la courbe compresse les magnitudes
//! tout en laissant Vénus et la Lune saturer.

use chrono::{DateTime, Local};
use image::{RgbaImage, imageops::rotate180};

use crate::bodies::Body;
use crate::ephemeris::{self, Appearance};
use crate::state::Viewpoint;

/// Brillance de surface de référence, en mag/″², voisine du disque de Neptune.
const SB_REF: f64 = 9.5;
/// Gain d'écran à cette référence : sombre, coloré, encore lisible.
const GAIN_AT_REF: f64 = 0.22;
/// Au-dessus de 1, les objets brillants partent vers le blanc plus vite que le flux.
/// 0,55 laissait Jupiter et la Lune presque aussi posés qu'en rendu détaillé.
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

/// `None` : pixels d'origine. `Some` : épaule `(x g) / (1 + x g)`.
pub fn present_disk(north_up: &RgbaImage, north_down: bool, gain: Option<f64>) -> egui::ColorImage {
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
        for channel in &mut px[..3] {
            *channel = tone_channel(*channel, gain);
        }
    }
    egui::ColorImage::from_rgba_unmultiplied(size, &pixels)
}

pub fn tone_channel(channel: u8, gain: f64) -> u8 {
    let x = f64::from(channel) / 255.0;
    let y = (x * gain) / (1.0 + x * gain);
    (y * 255.0).round().clamp(0.0, 255.0) as u8
}

pub fn tone_rgb(color: [u8; 3], gain: f64) -> egui::Color32 {
    egui::Color32::from_rgb(
        tone_channel(color[0], gain),
        tone_channel(color[1], gain),
        tone_channel(color[2], gain),
    )
}

/// Halo lisse. `illuminated` coupe le halo d'une nouvelle lune noire.
pub fn paint_halo(
    painter: &egui::Painter,
    ctx: &egui::Context,
    center: egui::Pos2,
    radius: f32,
    gain: f64,
    illuminated: f64,
    warm: bool,
) {
    if gain <= 1.15 || radius < 1.0 || illuminated < 0.02 {
        return;
    }
    let strength = (1.0 - (-(gain - 1.0) * 0.07).exp()).clamp(0.0, 1.0)
        * illuminated.sqrt().clamp(0.0, 1.0);
    let spread = (2.05 + 0.42 * gain.max(1.0).ln()).clamp(2.05, 3.5) as f32;
    let alpha = (230.0 * strength).round().clamp(0.0, 255.0) as u8;
    let tint = if warm {
        egui::Color32::from_rgba_unmultiplied(255, 236, 200, alpha)
    } else {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha)
    };
    paint_soft_disc(painter, ctx, center, radius * spread, tint);
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
    fn jupiter_clips_white_while_neptune_stays_dim() {
        let jupiter = disk_gain(5.4);
        let neptune = disk_gain(9.4);
        let jupiter_px = tone_channel(200, jupiter);
        let neptune_px = tone_channel(200, neptune);
        assert!(jupiter_px > 230, "Jupiter {jupiter_px} gain {jupiter}");
        assert!(neptune_px < 90, "Neptune {neptune_px} gain {neptune}");
        assert!(jupiter > neptune * 25.0, "{jupiter} vs {neptune}");
    }

    #[test]
    fn tone_curve_clips_a_bright_disk_and_keeps_a_dim_one() {
        let bright = tone_channel(220, 12.0);
        let dim = tone_channel(220, 0.22);
        assert!(bright > 220);
        assert!(dim < 80);
    }
}
