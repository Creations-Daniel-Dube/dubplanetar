//! Disque vu de la Terre : projection orthographique, phase de Lambert, anneaux.

use image::{Rgba, RgbaImage};

use crate::bodies::{self, AXIS_REACH, CANVAS_MARGIN, HeightMap, RingZone};
use crate::ephemeris::{Orientation, Vec3};

const AXIS_RGBA: [u8; 4] = [70, 150, 255, 255];
/// Hauteur (fraction du rayon, 1 = toute l'échelle de la carte) convertie en pente.
const BUMP_STRENGTH: f64 = 0.02;
/// Au-delà, la pente est rabotée pour ne pas retourner la normale.
const BUMP_MAX_GRAD: f64 = 48.0;
const RING_RGB: [u8; 3] = [230, 206, 150];

pub struct DiskInput<'a> {
    pub orientation: &'a Orientation,
    pub flattening: f64,
    pub albedo: [u8; 3],
    pub texture: Option<&'a RgbaImage>,
    /// Tours ajoutés à la longitude. 0,5 place le méridien 0° au centre de la carte.
    pub lon_offset_turns: f64,
    pub bump: Option<&'a HeightMap>,
    pub rings: &'a [RingZone],
    pub draw_axis: bool,
}

pub fn render_disk(input: &DiskInput<'_>, mut eq_radius_px: f64, max_side: i32) -> RgbaImage {
    eq_radius_px = eq_radius_px.max(0.75);
    let outer = bodies::outer_radius(input.rings);
    let extent = outer * CANVAS_MARGIN;
    let mut half = (eq_radius_px * extent).ceil() as i32 + 1;
    let mut size = half * 2 + 1;
    let cap = max_side.max(8);
    if size > cap {
        eq_radius_px *= f64::from(cap) / f64::from(size);
        half = (eq_radius_px * extent).ceil() as i32 + 1;
        size = half * 2 + 1;
    }
    let size_u = size as u32;
    let mut img = RgbaImage::new(size_u, size_u);
    let center = f64::from(half);

    let polar = (1.0 - input.flattening).clamp(0.05, 1.0);
    let cam = camera(input.orientation.to_observer);
    let los_b = to_body(input.orientation, cam.los);
    let sun_b = to_body(input.orientation, input.orientation.to_sun);
    let observer_b = to_body(input.orientation, input.orientation.to_observer);

    for y in 0..size_u {
        for x in 0..size_u {
            let dx = (f64::from(x) - center) / eq_radius_px;
            let dy = (center - f64::from(y)) / eq_radius_px;
            let sky = cam.right * dx + cam.up * dy;
            let sky_b = to_body(input.orientation, sky);
            let rgb = pixel_color(input, sky_b, los_b, sun_b, observer_b, polar, eq_radius_px);
            if let Some(rgb) = rgb {
                img.put_pixel(x, y, Rgba([rgb[0], rgb[1], rgb[2], 255]));
            }
        }
    }

    if input.draw_axis {
        draw_axis(
            &mut img,
            input.orientation,
            &cam,
            center,
            outer,
            eq_radius_px,
        );
    }
    img
}

fn pixel_color(
    input: &DiskInput<'_>,
    sky_b: Vec3,
    los_b: Vec3,
    sun_b: Vec3,
    observer_b: Vec3,
    polar: f64,
    eq_radius_px: f64,
) -> Option<[u8; 3]> {
    let globe = globe_hit(input, sky_b, los_b, sun_b, polar);
    let ring = ring_hit(
        input.rings,
        sky_b,
        los_b,
        sun_b,
        observer_b,
        polar,
        eq_radius_px,
    );
    match (globe, ring) {
        (Some(globe), Some(ring)) if ring.lambda < globe.lambda => Some(ring.rgb),
        (Some(globe), _) => Some(globe.rgb),
        (None, Some(ring)) => Some(ring.rgb),
        (None, None) => None,
    }
}

struct SurfaceHit {
    lambda: f64,
    rgb: [u8; 3],
}

fn globe_hit(
    input: &DiskInput<'_>,
    sky_b: Vec3,
    los_b: Vec3,
    sun_b: Vec3,
    polar: f64,
) -> Option<SurfaceHit> {
    let (lambda, point) = spheroid_front(sky_b, los_b, polar)?;
    let geometric = Vec3::new(point.x, point.y, point.z / (polar * polar)).normalized();
    let normal_b = bumped_normal(point, geometric, input.bump, input.lon_offset_turns);
    let lambert = normal_b.dot(sun_b).max(0.0);
    let albedo = match input.texture {
        Some(tex) => sample_texture(tex, point.x, point.y, point.z, input.lon_offset_turns),
        None => input.albedo,
    };
    Some(SurfaceHit {
        lambda,
        rgb: shade(albedo, lambert),
    })
}

fn ring_hit(
    zones: &[RingZone],
    sky_b: Vec3,
    los_b: Vec3,
    sun_b: Vec3,
    observer_b: Vec3,
    polar: f64,
    eq_radius_px: f64,
) -> Option<SurfaceHit> {
    if zones.is_empty() {
        return None;
    }
    let outer = bodies::outer_radius(zones);
    let minor_px = outer * los_b.z.abs() * eq_radius_px;
    let (lambda, point) = if minor_px < 1.5 {
        let perp_px = sky_b.z.abs() * eq_radius_px;
        if perp_px > 0.75 {
            return None;
        }
        let rho = sky_b.x.hypot(sky_b.y);
        (0.0, Vec3::new(sky_b.x, sky_b.y, 0.0).normalized() * rho)
    } else {
        let lambda = -sky_b.z / los_b.z;
        let point = sky_b + los_b * lambda;
        (lambda, point)
    };
    let rho = point.x.hypot(point.y);
    let zone = zones
        .iter()
        .find(|z| z.brightness > 0.0 && rho >= z.inner && rho < z.outer)?;
    let face = observer_b.z.signum();
    let illum = (face * sun_b.z).max(0.0);
    let light = if in_shadow(point, sun_b, polar) {
        0.08
    } else {
        illum.max(0.05)
    };
    Some(SurfaceHit {
        lambda,
        rgb: shade(RING_RGB, zone.brightness * light),
    })
}

/// Première intersection (côté observateur) d'un rayon orthographique avec le sphéroïde.
fn spheroid_front(sky_b: Vec3, los_b: Vec3, polar: f64) -> Option<(f64, Vec3)> {
    let c2 = polar * polar;
    let a = los_b.x * los_b.x + los_b.y * los_b.y + los_b.z * los_b.z / c2;
    let b = 2.0 * (sky_b.x * los_b.x + sky_b.y * los_b.y + sky_b.z * los_b.z / c2);
    let c = sky_b.x * sky_b.x + sky_b.y * sky_b.y + sky_b.z * sky_b.z / c2 - 1.0;
    if a.abs() < 1e-14 {
        return None;
    }
    let mut disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        if disc > -1e-8 {
            disc = 0.0;
        } else {
            return None;
        }
    }
    let lambda = (-b - disc.sqrt()) / (2.0 * a);
    Some((lambda, sky_b + los_b * lambda))
}

fn in_shadow(point: Vec3, sun_b: Vec3, polar: f64) -> bool {
    let c2 = polar * polar;
    let a = sun_b.x * sun_b.x + sun_b.y * sun_b.y + sun_b.z * sun_b.z / c2;
    let b = 2.0 * (point.x * sun_b.x + point.y * sun_b.y + point.z * sun_b.z / c2);
    let c = point.x * point.x + point.y * point.y + point.z * point.z / c2 - 1.0;
    if a.abs() < 1e-14 {
        return false;
    }
    let disc = b * b - 4.0 * a * c;
    if disc <= 0.0 {
        return false;
    }
    let t_far = (-b + disc.sqrt()) / (2.0 * a);
    t_far > 1e-3
}

fn shade(albedo: [u8; 3], factor: f64) -> [u8; 3] {
    let f = factor.clamp(0.0, 1.0);
    [
        (f64::from(albedo[0]) * f).round() as u8,
        (f64::from(albedo[1]) * f).round() as u8,
        (f64::from(albedo[2]) * f).round() as u8,
    ]
}

/// Normale locale. La pente est mesurée sur la sphère, dans le même cadrage que la photo.
fn bumped_normal(
    point: Vec3,
    geometric: Vec3,
    bump: Option<&HeightMap>,
    lon_offset_turns: f64,
) -> Vec3 {
    let Some(bump) = bump else {
        return geometric;
    };
    if bump.width() < 2 || bump.height() < 2 {
        return geometric;
    }
    let lon = point.y.atan2(point.x);
    let lat = point.z.atan2(point.x.hypot(point.y));
    let u = (lon / std::f64::consts::TAU + lon_offset_turns).rem_euclid(1.0);
    let v = (std::f64::consts::FRAC_PI_2 - lat) / std::f64::consts::PI;
    let du = 1.0 / f64::from(bump.width());
    let dv = 1.0 / f64::from(bump.height());
    let dh_east = (bump.sample(u + du, v) - bump.sample(u - du, v))
        / ((2.0 * du) * std::f64::consts::TAU * lat.cos().abs().max(0.15));
    let dh_north =
        (bump.sample(u, v - dv) - bump.sample(u, v + dv)) / ((2.0 * dv) * std::f64::consts::PI);
    let east = Vec3::new(-point.y, point.x, 0.0);
    let elen = east.length();
    if elen < 1e-6 {
        return geometric;
    }
    let east = east * (1.0 / elen);
    let north = point.cross(east).normalized();
    let mut slope = east * dh_east + north * dh_north;
    let grad = slope.length();
    if grad > BUMP_MAX_GRAD {
        slope = slope * (BUMP_MAX_GRAD / grad);
    }
    (geometric - slope * BUMP_STRENGTH).normalized()
}

fn sample_texture(tex: &RgbaImage, bx: f64, by: f64, bz: f64, lon_offset_turns: f64) -> [u8; 3] {
    let lon = by.atan2(bx);
    let lat = bz.atan2(bx.hypot(by));
    let u = (lon / std::f64::consts::TAU + lon_offset_turns).rem_euclid(1.0);
    let v = (std::f64::consts::FRAC_PI_2 - lat) / std::f64::consts::PI;
    bilinear(tex, u, v)
}

fn bilinear(tex: &RgbaImage, u: f64, v: f64) -> [u8; 3] {
    let w = tex.width() as f64;
    let h = tex.height() as f64;
    if w < 1.0 || h < 1.0 {
        return [0, 0, 0];
    }
    let x = u.rem_euclid(1.0) * w - 0.5;
    let y = v.clamp(0.0, 1.0) * h - 0.5;
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = (x - f64::from(x0)).clamp(0.0, 1.0);
    let ty = (y - f64::from(y0)).clamp(0.0, 1.0);
    let c00 = texel(tex, x0, y0);
    let c10 = texel(tex, x0 + 1, y0);
    let c01 = texel(tex, x0, y0 + 1);
    let c11 = texel(tex, x0 + 1, y0 + 1);
    mix_rgb(mix_rgb(c00, c10, tx), mix_rgb(c01, c11, tx), ty)
}

fn texel(tex: &RgbaImage, x: i32, y: i32) -> [u8; 3] {
    let w = tex.width() as i32;
    let h = tex.height() as i32;
    let xx = x.rem_euclid(w.max(1)) as u32;
    let yy = y.clamp(0, h - 1) as u32;
    let p = tex.get_pixel(xx, yy);
    [p[0], p[1], p[2]]
}

fn mix_rgb(a: [u8; 3], b: [u8; 3], t: f64) -> [u8; 3] {
    [
        mix_u8(a[0], b[0], t),
        mix_u8(a[1], b[1], t),
        mix_u8(a[2], b[2], t),
    ]
}

fn mix_u8(a: u8, b: u8, t: f64) -> u8 {
    (f64::from(a) * (1.0 - t) + f64::from(b) * t).round() as u8
}

fn draw_axis(
    img: &mut RgbaImage,
    orientation: &Orientation,
    cam: &Camera,
    center: f64,
    outer: f64,
    eq_radius_px: f64,
) {
    let pole_sky = orientation.z_axis - cam.los * orientation.z_axis.dot(cam.los);
    let rx = pole_sky.dot(cam.right);
    let uy = pole_sky.dot(cam.up);
    let len = rx.hypot(uy);
    if len < 1e-6 {
        return;
    }
    let reach = outer * AXIS_REACH * eq_radius_px;
    let ux = rx / len;
    let uy = uy / len;
    let rad = if eq_radius_px >= 80.0 { 1 } else { 0 };
    draw_line(
        img,
        (center + ux * reach).round() as i32,
        (center - uy * reach).round() as i32,
        (center - ux * reach).round() as i32,
        (center + uy * reach).round() as i32,
        AXIS_RGBA,
        rad,
    );
}

struct Camera {
    los: Vec3,
    right: Vec3,
    up: Vec3,
}

/// Nord céleste en haut, est à gauche : le +x de l'image est l'ouest.
fn camera(to_observer: Vec3) -> Camera {
    let los = -to_observer;
    let icrf_north = Vec3::new(0.0, 0.0, 1.0);
    let mut up = icrf_north - los * icrf_north.dot(los);
    if up.length() < 1e-8 {
        let fallback = Vec3::new(1.0, 0.0, 0.0);
        up = fallback - los * fallback.dot(los);
    }
    up = up.normalized();
    let east = up.cross(los).normalized();
    Camera {
        los,
        right: -east,
        up,
    }
}

fn to_body(o: &Orientation, v: Vec3) -> Vec3 {
    Vec3::new(v.dot(o.x_axis), v.dot(o.y_axis), v.dot(o.z_axis))
}

fn draw_line(
    img: &mut RgbaImage,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: [u8; 4],
    rad: i32,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        stamp(img, x0, y0, color, rad);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn stamp(img: &mut RgbaImage, x: i32, y: i32, color: [u8; 4], rad: i32) {
    let w = img.width() as i32;
    let h = img.height() as i32;
    for dy in -rad..=rad {
        for dx in -rad..=rad {
            if dx * dx + dy * dy > rad * rad {
                continue;
            }
            let px = x + dx;
            let py = y + dy;
            if px >= 0 && py >= 0 && px < w && py < h {
                img.put_pixel(px as u32, py as u32, Rgba(color));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bodies::{Body, HeightMap};
    use crate::ephemeris::{self, Orientation, Vec3};
    use chrono::TimeZone;
    use image::Rgba;

    fn equator_view(to_sun: Vec3) -> Orientation {
        Orientation {
            x_axis: Vec3::new(1.0, 0.0, 0.0),
            y_axis: Vec3::new(0.0, 1.0, 0.0),
            z_axis: Vec3::new(0.0, 0.0, 1.0),
            to_observer: Vec3::new(1.0, 0.0, 0.0),
            to_sun,
        }
    }

    fn render_plain(to_sun: Vec3, albedo: [u8; 3], texture: Option<&RgbaImage>) -> RgbaImage {
        render_disk(
            &DiskInput {
                orientation: &equator_view(to_sun),
                flattening: 0.0,
                albedo,
                texture,
                lon_offset_turns: 0.0,
                bump: None,
                rings: &[],
                draw_axis: false,
            },
            100.0,
            1000,
        )
    }

    fn luma(p: &Rgba<u8>) -> i32 {
        (i32::from(p[0]) + i32::from(p[1]) + i32::from(p[2])) / 3
    }

    #[test]
    fn full_phase_lights_the_center_and_both_sides() {
        let img = render_plain(Vec3::new(1.0, 0.0, 0.0), [255, 255, 255], None);
        let cx = img.width() / 2;
        let cy = img.height() / 2;
        let center = img.get_pixel(cx, cy);
        let left = img.get_pixel(cx - 45, cy);
        let right = img.get_pixel(cx + 45, cy);
        assert!(luma(center) > 240, "centre {center:?}");
        assert!(luma(left) > 150, "gauche {left:?}");
        assert!(luma(right) > 150, "droite {right:?}");
        assert_eq!(img.get_pixel(0, 0)[3], 0);
    }

    #[test]
    fn quarter_phase_splits_the_disk() {
        // Soleil vers +Y : à droite de l'image quand la Terre est en +X.
        let img = render_plain(Vec3::new(0.0, 1.0, 0.0), [255, 255, 255], None);
        let cx = img.width() / 2;
        let cy = img.height() / 2;
        let lit = img.get_pixel(cx + 45, cy);
        let dark = img.get_pixel(cx - 45, cy);
        assert!(luma(lit) > 70, "côté éclairé {lit:?}");
        assert!(luma(dark) < 25, "côté nuit {dark:?}");
        assert_eq!(dark[3], 255);
        assert_eq!(lit[3], 255);
    }

    #[test]
    fn missing_texture_keeps_the_solid_color() {
        let img = render_plain(Vec3::new(1.0, 0.0, 0.0), [20, 180, 30], None);
        let p = img.get_pixel(img.width() / 2, img.height() / 2);
        assert!(p[1] > 150 && p[0] < 40, "couleur unie {p:?}");
    }

    #[test]
    fn texture_replaces_the_lit_albedo() {
        let tex = RgbaImage::from_pixel(8, 4, Rgba([220, 10, 10, 255]));
        let img = render_plain(Vec3::new(1.0, 0.0, 0.0), [20, 180, 30], Some(&tex));
        let p = img.get_pixel(img.width() / 2, img.height() / 2);
        assert!(p[0] > 180 && p[1] < 40, "photo {p:?}");
    }

    #[test]
    fn north_pole_axis_extends_past_the_limb() {
        let orientation = equator_view(Vec3::new(1.0, 0.0, 0.0));
        let img = render_disk(
            &DiskInput {
                orientation: &orientation,
                flattening: 0.0,
                albedo: [255, 255, 255],
                texture: None,
                lon_offset_turns: 0.0,
                bump: None,
                rings: &[],
                draw_axis: true,
            },
            100.0,
            1000,
        );
        let cx = img.width() / 2;
        let cy = img.height() / 2;
        let y = cy - 105;
        let p = img.get_pixel(cx, y);
        assert_eq!(p.0, AXIS_RGBA, "axe {p:?} à ({cx}, {y})");
    }

    #[test]
    fn saturn_rings_extend_beyond_the_globe() {
        let dt = chrono::Utc.with_ymd_and_hms(2012, 4, 15, 0, 0, 0).unwrap();
        let app = ephemeris::appearance(Body::Saturn, ephemeris::julian_day_utc(dt));
        let radius = 48.0;
        let img = render_disk(
            &DiskInput {
                orientation: &app.orientation,
                flattening: app.flattening,
                albedo: Body::Saturn.albedo(),
                texture: None,
                lon_offset_turns: 0.0,
                bump: None,
                rings: Body::Saturn.ring_zones(),
                draw_axis: false,
            },
            radius,
            1000,
        );
        let cx = img.width() as f64 / 2.0;
        let cy = img.height() as f64 / 2.0;
        let found = (0..img.width()).any(|x| {
            (0..img.height()).any(|y| {
                let p = img.get_pixel(x, y);
                if p[3] == 0 {
                    return false;
                }
                let dx = f64::from(x) - cx;
                let dy = f64::from(y) - cy;
                dx.hypot(dy) > 1.25 * radius
            })
        });
        assert!(found, "les anneaux doivent dépasser le limbe du globe");
    }

    #[test]
    fn moon_longitude_offset_reads_the_center_of_the_map() {
        // Bord gauche bleu, centre rouge. La Terre est dans l'axe +X : longitude 0°.
        let mut tex = RgbaImage::from_pixel(8, 4, Rgba([10, 20, 220, 255]));
        for y in 0..4 {
            for x in 3..5 {
                tex.put_pixel(x, y, Rgba([220, 15, 10, 255]));
            }
        }
        let orientation = equator_view(Vec3::new(1.0, 0.0, 0.0));
        let img = render_disk(
            &DiskInput {
                orientation: &orientation,
                flattening: 0.0,
                albedo: [0, 0, 0],
                texture: Some(&tex),
                lon_offset_turns: 0.5,
                bump: None,
                rings: &[],
                draw_axis: false,
            },
            40.0,
            1000,
        );
        let p = img.get_pixel(img.width() / 2, img.height() / 2);
        assert!(p[0] > 180 && p[2] < 40, "centre de la carte {p:?}");
    }

    fn render_bump(to_sun: Vec3, bump: Option<&HeightMap>) -> RgbaImage {
        render_disk(
            &DiskInput {
                orientation: &equator_view(to_sun),
                flattening: 0.0,
                albedo: [255, 255, 255],
                texture: None,
                lon_offset_turns: 0.5,
                bump,
                rings: &[],
                draw_axis: false,
            },
            40.0,
            1000,
        )
    }

    fn center_luma(img: &RgbaImage) -> i32 {
        luma(img.get_pixel(img.width() / 2, img.height() / 2))
    }

    /// Marche vers l'est : bas à gauche de la carte, haut à droite.
    fn eastward_step() -> HeightMap {
        let width = 64;
        let height = 16;
        let mut samples = Vec::with_capacity((width * height) as usize);
        for _y in 0..height {
            for x in 0..width {
                samples.push(if x < width / 2 { 0 } else { u16::MAX });
            }
        }
        HeightMap::from_samples(width, height, samples)
    }

    #[test]
    fn flat_bump_matches_the_smooth_disk() {
        let flat = HeightMap::from_samples(8, 4, vec![u16::MAX / 2; 32]);
        let sun = Vec3::new(1.0, 0.4, 0.1).normalized();
        let plain = center_luma(&render_bump(sun, None));
        let bumped = center_luma(&render_bump(sun, Some(&flat)));
        assert_eq!(plain, bumped, "plate {bumped}, lisse {plain}");
    }

    #[test]
    fn ramp_faces_the_low_side() {
        // La marche monte vers l'est. La pente regarde le côté bas, à l'ouest.
        // Le Soleil du côté haut éclaire moins le centre que celui du côté bas.
        let bump = eastward_step();
        let uphill = Vec3::new(1.0, 1.0, 0.0).normalized();
        let downhill = Vec3::new(1.0, -1.0, 0.0).normalized();
        let from_high = center_luma(&render_bump(uphill, Some(&bump)));
        let from_low = center_luma(&render_bump(downhill, Some(&bump)));
        let smooth = center_luma(&render_bump(uphill, None));
        assert!(
            from_high < from_low,
            "côté haut {from_high}, côté bas {from_low}"
        );
        assert!(from_high < smooth, "relief {from_high}, lisse {smooth}");
    }
}
