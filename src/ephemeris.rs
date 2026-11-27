//! Positions, tailles apparentes et orientation.
//!
//! VSOP87A donne des coordonnées écliptiques J2000. Une itération de temps de lumière
//! place la planète au moment où la lumière que l'on reçoit en est partie.
//! L'axe et le méridien suivent le rapport IAU WGCCRE 2015 (termes principaux).
//! L'origine est la Terre, sauf si un autre `Viewpoint` est demandé. La Lune reste
//! toujours vue de la Terre.

use chrono::{DateTime, Utc};

use crate::bodies::Body;
use crate::state::Viewpoint;

const AU_KM: f64 = 149_597_870.7;
const ARCSEC_PER_RAD: f64 = 206_264.806_247_096_36;
/// Rayon nominal du Soleil, UAI 2015.
const SOLAR_RADIUS_KM: f64 = 695_700.0;
/// Jours-lumière pour une unité astronomique.
const LIGHT_DAYS_PER_AU: f64 = 499.004_783_836 / 86_400.0;
/// Obliquité moyenne J2000, pour passer de l'écliptique VSOP87 à l'équatorial.
const OBLIQUITY_J2000_DEG: f64 = 23.439_291_1;
const J2000_JD: f64 = 2_451_545.0;

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalized(self) -> Self {
        let len = self.length();
        if len < 1e-15 {
            self
        } else {
            self * (1.0 / len)
        }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl std::ops::Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

/// Repère du globe : X vers le méridien origine, Z vers le pôle nord IAU.
#[derive(Clone, Copy, Debug)]
pub struct Orientation {
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub z_axis: Vec3,
    /// Depuis le centre de la planète vers l'observateur.
    pub to_observer: Vec3,
    /// Depuis le centre de la planète vers le Soleil.
    pub to_sun: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct Appearance {
    pub distance_au: f64,
    pub eq_diameter_arcsec: f64,
    pub illuminated: f64,
    pub phase_angle_rad: f64,
    pub sub_observer_lat_rad: f64,
    pub pole_pa_rad: f64,
    pub bright_limb_pa_rad: f64,
    pub flattening: f64,
    pub orientation: Orientation,
}

pub fn julian_day_utc(dt: DateTime<Utc>) -> f64 {
    let secs = dt.timestamp() as f64 + f64::from(dt.timestamp_subsec_nanos()) * 1e-9;
    secs / 86_400.0 + 2_440_587.5
}

pub fn appearance(body: Body, jd: f64) -> Appearance {
    appearance_from(body, Viewpoint::Earth, jd)
}

/// Disque de `target` vu depuis `viewpoint` à l'instant `jd`.
///
/// La Lune ignore `viewpoint` et reste géocentrique. L'observateur est pris à `jd`,
/// la cible une itération de temps de lumière plus tôt.
pub fn appearance_from(target: Body, viewpoint: Viewpoint, jd: f64) -> Appearance {
    if target == Body::Moon {
        return moon_appearance(jd);
    }
    debug_assert_ne!(
        viewpoint.as_body(),
        target,
        "la cible ne peut pas être l'observateur"
    );
    let observer = heliocentric(viewpoint.as_body(), jd);
    let mut planet = heliocentric(target, jd);
    let light_days = (planet - observer).length() * LIGHT_DAYS_PER_AU;
    planet = heliocentric(target, jd - light_days);

    let geo = planet - observer;
    let distance = geo.length().max(1e-9);
    let to_observer = -geo.normalized();
    let to_sun = -planet.normalized();

    let cos_i = to_observer.dot(to_sun).clamp(-1.0, 1.0);
    let (eq_km, _) = target.radii_km();
    let eq_diameter = 2.0 * eq_km / (distance * AU_KM) * ARCSEC_PER_RAD;

    let (alpha0, delta0, w_deg) = rotational_elements(target, jd - light_days);
    let orientation = orientation_axes(alpha0, delta0, w_deg, to_observer, to_sun);

    let planet_ra = geo.y.atan2(geo.x);
    let planet_dec = geo.z.atan2(geo.x.hypot(geo.y));
    let sun = -observer;
    let sun_ra = sun.y.atan2(sun.x);
    let sun_dec = sun.z.atan2(sun.x.hypot(sun.y));

    Appearance {
        distance_au: distance,
        eq_diameter_arcsec: eq_diameter,
        illuminated: (1.0 + cos_i) * 0.5,
        phase_angle_rad: cos_i.acos(),
        sub_observer_lat_rad: orientation.z_axis.dot(to_observer).clamp(-1.0, 1.0).asin(),
        pole_pa_rad: position_angle(
            planet_ra,
            planet_dec,
            alpha0.to_radians(),
            delta0.to_radians(),
        ),
        bright_limb_pa_rad: position_angle(planet_ra, planet_dec, sun_ra, sun_dec),
        flattening: target.flattening(),
        orientation,
    }
}

/// Angle de position de `to` vu depuis `from`, depuis le nord vers l'est, en radians.
pub(crate) fn position_angle(from_ra: f64, from_dec: f64, to_ra: f64, to_dec: f64) -> f64 {
    let dra = to_ra - from_ra;
    let y = to_dec.cos() * dra.sin();
    let x = from_dec.cos() * to_dec.sin() - from_dec.sin() * to_dec.cos() * dra.cos();
    y.atan2(x)
}

fn orientation_axes(
    alpha0_deg: f64,
    delta0_deg: f64,
    w_deg: f64,
    to_observer: Vec3,
    to_sun: Vec3,
) -> Orientation {
    let z_axis = ra_dec_to_vec(alpha0_deg.to_radians(), delta0_deg.to_radians());
    let mut node = Vec3::new(-z_axis.y, z_axis.x, 0.0);
    let nlen = node.length();
    if nlen < 1e-10 {
        node = Vec3::new(1.0, 0.0, 0.0);
    } else {
        node = node * (1.0 / nlen);
    }
    let w = w_deg.to_radians();
    let x_axis = (node * w.cos() + z_axis.cross(node) * w.sin()).normalized();
    let y_axis = z_axis.cross(x_axis).normalized();
    Orientation {
        x_axis,
        y_axis,
        z_axis,
        to_observer,
        to_sun,
    }
}

fn ra_dec_to_vec(ra: f64, dec: f64) -> Vec3 {
    Vec3::new(dec.cos() * ra.cos(), dec.cos() * ra.sin(), dec.sin())
}

/// `(α0, δ0, W)` en degrés. `d` et `T` comptés depuis J2000.
fn rotational_elements(body: Body, jd: f64) -> (f64, f64, f64) {
    let d = jd - J2000_JD;
    let t = d / 36_525.0;
    match body {
        Body::Mercury => (
            281.0103 - 0.0328 * t,
            61.4155 - 0.0049 * t,
            329.5988 + 6.138_510_8 * d,
        ),
        Body::Venus => (272.76, 67.16, 160.20 - 1.481_368_8 * d),
        Body::Mars => {
            let a = 79.398_797 + 0.504_261_5 * t;
            let b = 166.325_722 + 0.504_261_5 * t;
            let c = 95.391_654 + 0.504_261_5 * t;
            (
                317.269_202 - 0.109_275_47 * t + 0.419_057 * deg_sin(a),
                54.432_516 - 0.058_271_05 * t + 1.591_274 * deg_cos(b),
                176.049_863 + 350.891_982_443_297 * d + 0.584_542 * deg_sin(c),
            )
        }
        Body::Jupiter => (
            268.056_595 - 0.006_499 * t,
            64.495_303 + 0.002_413 * t,
            284.95 + 870.536 * d,
        ),
        Body::Saturn => (
            40.589 - 0.036 * t,
            83.537 - 0.004 * t,
            38.90 + 810.793_902_4 * d,
        ),
        Body::Uranus => (257.311, -15.175, 203.81 - 501.160_092_8 * d),
        Body::Neptune => {
            let n = 357.85 + 52.316 * t;
            (
                299.36 + 0.70 * deg_sin(n),
                43.46 - 0.51 * deg_cos(n),
                249.978 + 541.139_775_7 * d - 0.48 * deg_sin(n),
            )
        }
        Body::Earth => (-0.641 * t, 90.0 - 0.557 * t, 190.147 + 360.985_623_5 * d),
        Body::Moon => moon_rotational_elements(d, t),
    }
}

/// Pôle et méridien de la Lune, rapport IAU WGCCRE 2015. Les sinus portent la libration.
fn moon_rotational_elements(d: f64, t: f64) -> (f64, f64, f64) {
    let e1 = 125.045 - 0.052_992_1 * d;
    let e2 = 250.089 - 0.105_984_2 * d;
    let e3 = 260.008 + 13.012_000_9 * d;
    let e4 = 176.625 + 13.340_715_4 * d;
    let e5 = 357.529 + 0.985_600_3 * d;
    let e6 = 311.589 + 26.405_708_4 * d;
    let e7 = 134.963 + 13.064_993_0 * d;
    let e8 = 276.617 + 0.328_714_6 * d;
    let e9 = 34.226 + 1.748_487_7 * d;
    let e10 = 15.134 - 0.158_976_3 * d;
    let e11 = 119.743 + 0.003_609_6 * d;
    let e12 = 239.961 + 0.164_357_3 * d;
    let e13 = 25.053 + 12.959_008_8 * d;
    let alpha0 = 269.9949 + 0.0031 * t - 3.8787 * deg_sin(e1) - 0.1204 * deg_sin(e2)
        + 0.0700 * deg_sin(e3)
        - 0.0172 * deg_sin(e4)
        + 0.0072 * deg_sin(e6)
        - 0.0052 * deg_sin(e10)
        + 0.0043 * deg_sin(e13);
    let delta0 = 66.5392 + 0.0130 * t + 1.5419 * deg_cos(e1) + 0.0239 * deg_cos(e2)
        - 0.0278 * deg_cos(e3)
        + 0.0068 * deg_cos(e4)
        - 0.0029 * deg_cos(e6)
        + 0.0009 * deg_cos(e7)
        + 0.0008 * deg_cos(e10)
        - 0.0009 * deg_cos(e13);
    let w =
        38.3213 + 13.176_358_15 * d - 1.4e-12 * d * d + 3.5610 * deg_sin(e1) + 0.1208 * deg_sin(e2)
            - 0.0642 * deg_sin(e3)
            + 0.0158 * deg_sin(e4)
            + 0.0252 * deg_sin(e5)
            - 0.0066 * deg_sin(e6)
            - 0.0047 * deg_sin(e7)
            - 0.0046 * deg_sin(e8)
            + 0.0028 * deg_sin(e9)
            + 0.0052 * deg_sin(e10)
            + 0.0040 * deg_sin(e11)
            + 0.0019 * deg_sin(e12)
            - 0.0044 * deg_sin(e13);
    (alpha0, delta0, w)
}

fn deg_sin(deg: f64) -> f64 {
    deg.to_radians().sin()
}

fn deg_cos(deg: f64) -> f64 {
    deg.to_radians().cos()
}

fn equatorial_planet(body: Body, jd: f64) -> Vec3 {
    let c = match body {
        Body::Mercury => vsop87::vsop87a::mercury(jd),
        Body::Venus => vsop87::vsop87a::venus(jd),
        Body::Mars => vsop87::vsop87a::mars(jd),
        Body::Jupiter => vsop87::vsop87a::jupiter(jd),
        Body::Saturn => vsop87::vsop87a::saturn(jd),
        Body::Uranus => vsop87::vsop87a::uranus(jd),
        Body::Neptune => vsop87::vsop87a::neptune(jd),
        Body::Earth | Body::Moon => {
            unreachable!("la Terre et la Lune ne passent pas par cette table")
        }
    };
    ecliptic_to_equatorial(Vec3::new(c.x, c.y, c.z))
}

fn heliocentric(body: Body, jd: f64) -> Vec3 {
    if body == Body::Earth {
        equatorial_earth(jd)
    } else {
        equatorial_planet(body, jd)
    }
}

fn equatorial_earth(jd: f64) -> Vec3 {
    let c = vsop87::vsop87a::earth(jd);
    ecliptic_to_equatorial(Vec3::new(c.x, c.y, c.z))
}

/// Distance observateur–Soleil et diamètre apparent à l'instant `jd`.
///
/// Le Soleil est à l'origine de VSOP87 : la distance est la longueur du vecteur
/// héliocentrique de l'observateur, en unités astronomiques. Pas de phase : le
/// disque est toujours plein.
#[derive(Clone, Copy, Debug)]
pub struct SunApparent {
    pub distance_au: f64,
    pub eq_diameter_arcsec: f64,
}

pub fn sun_apparent(jd: f64) -> SunApparent {
    sun_apparent_from(jd, Viewpoint::Earth)
}

pub fn sun_apparent_from(jd: f64, viewpoint: Viewpoint) -> SunApparent {
    let distance_au = heliocentric(viewpoint.as_body(), jd).length().max(1e-12);
    let eq_diameter_arcsec = 2.0 * SOLAR_RADIUS_KM / (distance_au * AU_KM) * ARCSEC_PER_RAD;
    SunApparent {
        distance_au,
        eq_diameter_arcsec,
    }
}

fn ecliptic_to_equatorial(v: Vec3) -> Vec3 {
    ecliptic_to_equatorial_obliquity(v, OBLIQUITY_J2000_DEG.to_radians())
}

fn ecliptic_to_equatorial_obliquity(v: Vec3, eps: f64) -> Vec3 {
    let (ce, se) = (eps.cos(), eps.sin());
    Vec3::new(v.x, v.y * ce - v.z * se, v.y * se + v.z * ce)
}

/// Longitude, latitude écliptiques de la date (degrés) et distance (km).
struct MoonEcliptic {
    lon_deg: f64,
    lat_deg: f64,
    dist_km: f64,
}

/// Séries courtes de Meeus, chapitre 47. Environ 10″ en longitude et 4″ en latitude.
fn moon_ecliptic(jd: f64) -> MoonEcliptic {
    let t = (jd - J2000_JD) / 36_525.0;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let lp = 218.316_447_7 + 481_267.881_234_21 * t - 0.001_578_6 * t2 + t3 / 538_841.0
        - t4 / 65_194_000.0;
    let d = 297.850_192_1 + 445_267.111_403_4 * t - 0.001_881_9 * t2 + t3 / 545_868.0
        - t4 / 113_065_000.0;
    let m = 357.529_109_2 + 35_999.050_290_9 * t - 0.000_153_6 * t2 + t3 / 24_490_000.0;
    let mp = 134.963_396_4 + 477_198.867_505_5 * t + 0.008_741_4 * t2 + t3 / 69_699.0
        - t4 / 14_712_000.0;
    let f = 93.272_095_0 + 483_202.017_523_3 * t - 0.003_653_9 * t2 - t3 / 3_526_000.0
        + t4 / 863_310_000.0;
    let e = 1.0 - 0.002_516 * t - 0.000_007_4 * t2;
    let ee = e * e;

    let mut sum_l = 0.0;
    let mut sum_r = 0.0;
    for (cd, cm, cmp, cf, sl, sr) in MOON_LR {
        let arg = (d * f64::from(cd) + m * f64::from(cm) + mp * f64::from(cmp) + f * f64::from(cf))
            .to_radians();
        let factor = match cm.abs() {
            1 => e,
            2 => ee,
            _ => 1.0,
        };
        sum_l += factor * f64::from(sl) * arg.sin();
        sum_r += factor * f64::from(sr) * arg.cos();
    }
    let a1 = 119.75 + 131.849 * t;
    let a2 = 53.09 + 479_264.290 * t;
    let a3 = 313.45 + 481_266.484 * t;
    sum_l += 3_958.0 * deg_sin(a1) + 1_962.0 * deg_sin(lp - f) + 318.0 * deg_sin(a2);

    let mut sum_b = 0.0;
    for (cd, cm, cmp, cf, sb) in MOON_B {
        let arg = (d * f64::from(cd) + m * f64::from(cm) + mp * f64::from(cmp) + f * f64::from(cf))
            .to_radians();
        let factor = match cm.abs() {
            1 => e,
            2 => ee,
            _ => 1.0,
        };
        sum_b += factor * f64::from(sb) * arg.sin();
    }
    sum_b += -2_235.0 * deg_sin(lp)
        + 382.0 * deg_sin(a3)
        + 175.0 * deg_sin(a1 - f)
        + 175.0 * deg_sin(a1 + f)
        + 127.0 * deg_sin(lp - mp)
        - 115.0 * deg_sin(lp + mp);

    MoonEcliptic {
        lon_deg: lp + sum_l * 1e-6,
        lat_deg: sum_b * 1e-6,
        dist_km: 385_000.56 + sum_r * 0.001,
    }
}

/// `(D, M, M', F, Σl en 10⁻⁶ °, Σr en 0,001 km)`.
const MOON_LR: [(i8, i8, i8, i8, i32, i32); 60] = [
    (0, 0, 1, 0, 6_288_774, -20_905_355),
    (2, 0, -1, 0, 1_274_027, -3_699_111),
    (2, 0, 0, 0, 658_314, -2_955_968),
    (0, 0, 2, 0, 213_618, -569_925),
    (0, 1, 0, 0, -185_116, 48_888),
    (0, 0, 0, 2, -114_332, -3_149),
    (2, 0, -2, 0, 58_793, 246_158),
    (2, -1, -1, 0, 57_066, -152_138),
    (2, 0, 1, 0, 53_322, -170_733),
    (2, -1, 0, 0, 45_758, -204_586),
    (0, 1, -1, 0, -40_923, -129_620),
    (1, 0, 0, 0, -34_720, 108_743),
    (0, 1, 1, 0, -30_383, 104_755),
    (2, 0, 0, -2, 15_327, 10_321),
    (0, 0, 1, 2, -12_528, 0),
    (0, 0, 1, -2, 10_980, 79_661),
    (4, 0, -1, 0, 10_675, -34_782),
    (0, 0, 3, 0, 10_034, -23_210),
    (4, 0, -2, 0, 8_548, -21_636),
    (2, 1, -1, 0, -7_888, 24_208),
    (2, 1, 0, 0, -6_766, 30_824),
    (1, 0, -1, 0, -5_163, -8_379),
    (1, 1, 0, 0, 4_987, -16_675),
    (2, -1, 1, 0, 4_036, -12_831),
    (2, 0, 2, 0, 3_994, -10_445),
    (4, 0, 0, 0, 3_861, -11_650),
    (2, 0, -3, 0, 3_665, 14_403),
    (0, 1, -2, 0, -2_689, -7_003),
    (2, 0, -1, 2, -2_602, 0),
    (2, -1, -2, 0, 2_390, 10_056),
    (1, 0, 1, 0, -2_348, 6_322),
    (2, -2, 0, 0, 2_236, -9_884),
    (0, 1, 2, 0, -2_120, 5_751),
    (0, 2, 0, 0, -2_069, 0),
    (2, -2, -1, 0, 2_048, -4_950),
    (2, 0, 1, -2, -1_773, 4_130),
    (2, 0, 0, 2, -1_595, 0),
    (4, -1, -1, 0, 1_215, -3_958),
    (0, 0, 2, 2, -1_110, 0),
    (3, 0, -1, 0, -892, 3_258),
    (2, 1, 1, 0, -810, 2_616),
    (4, -1, -2, 0, 759, -1_897),
    (0, 2, -1, 0, -713, -2_117),
    (2, 2, -1, 0, -700, 2_354),
    (2, 1, -2, 0, 691, 0),
    (2, -1, 0, -2, 596, 0),
    (4, 0, 1, 0, 549, -1_423),
    (0, 0, 4, 0, 537, -1_117),
    (4, -1, 0, 0, 520, -1_571),
    (1, 0, -2, 0, -487, -1_739),
    (2, 1, 0, -2, -399, 0),
    (0, 0, 2, -2, -381, -4_421),
    (1, 1, 1, 0, 351, 0),
    (3, 0, -2, 0, -340, 0),
    (4, 0, -3, 0, 330, 0),
    (2, -1, 2, 0, 327, 0),
    (0, 2, 1, 0, -323, 1_165),
    (1, 1, -1, 0, 299, 0),
    (2, 0, 3, 0, 294, 0),
    (2, 0, -1, -2, 0, 8_752),
];

/// `(D, M, M', F, Σb en 10⁻⁶ °)`.
const MOON_B: [(i8, i8, i8, i8, i32); 60] = [
    (0, 0, 0, 1, 5_128_122),
    (0, 0, 1, 1, 280_602),
    (0, 0, 1, -1, 277_693),
    (2, 0, 0, -1, 173_237),
    (2, 0, -1, 1, 55_413),
    (2, 0, -1, -1, 46_271),
    (2, 0, 0, 1, 32_573),
    (0, 0, 2, 1, 17_198),
    (2, 0, 1, -1, 9_266),
    (0, 0, 2, -1, 8_822),
    (2, -1, 0, -1, 8_216),
    (2, 0, -2, -1, 4_324),
    (2, 0, 1, 1, 4_200),
    (2, 1, 0, -1, -3_359),
    (2, -1, -1, 1, 2_463),
    (2, -1, 0, 1, 2_211),
    (2, -1, -1, -1, 2_065),
    (0, 1, -1, -1, -1_870),
    (4, 0, -1, -1, 1_828),
    (0, 1, 0, 1, -1_794),
    (0, 0, 0, 3, -1_749),
    (0, 1, -1, 1, -1_565),
    (1, 0, 0, 1, -1_491),
    (0, 1, 1, 1, -1_475),
    (0, 1, 1, -1, -1_410),
    (0, 1, 0, -1, -1_344),
    (1, 0, 0, -1, -1_335),
    (0, 0, 3, 1, 1_107),
    (4, 0, 0, -1, 1_021),
    (4, 0, -1, 1, 833),
    (0, 0, 1, -3, 777),
    (4, 0, -2, 1, 671),
    (2, 0, 0, -3, 607),
    (2, 0, 2, -1, 596),
    (2, -1, 1, -1, 491),
    (2, 0, -2, 1, -451),
    (0, 0, 3, -1, 439),
    (2, 0, 2, 1, 422),
    (2, 0, -3, -1, 421),
    (2, 1, -1, 1, -366),
    (2, 1, 0, 1, -351),
    (4, 0, 0, 1, 331),
    (2, -1, 1, 1, 315),
    (2, -2, 0, -1, 302),
    (0, 0, 1, 3, -283),
    (2, 1, 1, -1, -229),
    (1, 1, 0, -1, 223),
    (1, 1, 0, 1, 223),
    (0, 1, -2, -1, -220),
    (2, 1, -1, -1, -220),
    (1, 0, 1, 1, -185),
    (2, -1, -2, -1, 181),
    (0, 1, 2, 1, -177),
    (4, 0, -2, -1, 176),
    (4, -1, -1, -1, 166),
    (1, 0, 1, -1, -164),
    (4, 0, 1, -1, 132),
    (1, 0, -1, -1, -119),
    (4, -1, 0, -1, 115),
    (2, -2, 0, 1, 107),
];

fn moon_appearance(jd: f64) -> Appearance {
    let earth = equatorial_earth(jd);
    let mut geo = moon_geocentric_j2000(jd);
    let light_days = geo.length() * LIGHT_DAYS_PER_AU;
    geo = moon_geocentric_j2000(jd - light_days);

    let distance = geo.length().max(1e-12);
    let moon = earth + geo;
    let to_observer = -geo.normalized();
    let to_sun = -moon.normalized();
    let cos_i = to_observer.dot(to_sun).clamp(-1.0, 1.0);
    let (eq_km, _) = Body::Moon.radii_km();
    let eq_diameter = 2.0 * eq_km / (distance * AU_KM) * ARCSEC_PER_RAD;

    let (alpha0, delta0, w_deg) = rotational_elements(Body::Moon, jd - light_days);
    let orientation = orientation_axes(alpha0, delta0, w_deg, to_observer, to_sun);

    let moon_ra = geo.y.atan2(geo.x);
    let moon_dec = geo.z.atan2(geo.x.hypot(geo.y));
    let sun = -earth;
    let sun_ra = sun.y.atan2(sun.x);
    let sun_dec = sun.z.atan2(sun.x.hypot(sun.y));

    Appearance {
        distance_au: distance,
        eq_diameter_arcsec: eq_diameter,
        illuminated: (1.0 + cos_i) * 0.5,
        phase_angle_rad: cos_i.acos(),
        sub_observer_lat_rad: orientation.z_axis.dot(to_observer).clamp(-1.0, 1.0).asin(),
        pole_pa_rad: position_angle(moon_ra, moon_dec, alpha0.to_radians(), delta0.to_radians()),
        bright_limb_pa_rad: position_angle(moon_ra, moon_dec, sun_ra, sun_dec),
        flattening: Body::Moon.flattening(),
        orientation,
    }
}

/// Géocentrique, équatorial J2000, en UA.
fn moon_geocentric_j2000(jd: f64) -> Vec3 {
    let place = moon_ecliptic(jd);
    let dist_au = place.dist_km / AU_KM;
    let lon = place.lon_deg.to_radians();
    let lat = place.lat_deg.to_radians();
    let ecliptic = Vec3::new(
        dist_au * lat.cos() * lon.cos(),
        dist_au * lat.cos() * lon.sin(),
        dist_au * lat.sin(),
    );
    let t = (jd - J2000_JD) / 36_525.0;
    let equatorial = ecliptic_to_equatorial_obliquity(ecliptic, obliquity_of_date(t));
    precess_to_j2000(equatorial, t)
}

fn obliquity_of_date(t: f64) -> f64 {
    let arcsec = 84_381.448 - 46.8150 * t - 0.000_59 * t * t + 0.001_813 * t * t * t;
    (arcsec / 3600.0).to_radians()
}

/// Équatorial moyen de la date vers l'équatorial J2000 (précession IAU 1976).
fn precess_to_j2000(v: Vec3, t: f64) -> Vec3 {
    let arcsec = |seconds: f64| (seconds / 3600.0).to_radians();
    let t2 = t * t;
    let t3 = t2 * t;
    let zeta = arcsec(2306.2181 * t + 0.30188 * t2 + 0.017998 * t3);
    let z = arcsec(2306.2181 * t + 1.09468 * t2 + 0.018203 * t3);
    let theta = arcsec(2004.3109 * t - 0.42665 * t2 - 0.041833 * t3);
    let (sz, cz) = (z.sin(), z.cos());
    let (st, ct) = (theta.sin(), theta.cos());
    let (szeta, czeta) = (zeta.sin(), zeta.cos());
    // P envoie J2000 vers la date. On applique sa transposée.
    let p00 = czeta * ct * cz - szeta * sz;
    let p01 = -szeta * ct * cz - czeta * sz;
    let p02 = -st * cz;
    let p10 = czeta * ct * sz + szeta * cz;
    let p11 = -szeta * ct * sz + czeta * cz;
    let p12 = -st * sz;
    let p20 = czeta * st;
    let p21 = -szeta * st;
    let p22 = ct;
    Vec3::new(
        p00 * v.x + p10 * v.y + p20 * v.z,
        p01 * v.x + p11 * v.y + p21 * v.z,
        p02 * v.x + p12 * v.y + p22 * v.z,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn jd(year: i32, month: u32, day: u32) -> f64 {
        let dt = Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
        julian_day_utc(dt)
    }

    #[test]
    fn position_angle_north_and_east() {
        let north = position_angle(0.0, 0.0, 0.0, 10_f64.to_radians());
        assert!(north.abs() < 1e-12, "nord {north}");
        let east = position_angle(0.0, 0.0, 10_f64.to_radians(), 0.0);
        assert!(
            (east - std::f64::consts::FRAC_PI_2).abs() < 1e-12,
            "est {east}"
        );
    }

    #[test]
    fn venus_grows_and_wanes_through_april_2012() {
        let early = appearance(Body::Venus, jd(2012, 4, 1));
        let late = appearance(Body::Venus, jd(2012, 4, 30));
        assert!(
            early.eq_diameter_arcsec < late.eq_diameter_arcsec,
            "diamètre 1er avril {:.2}\" devrait être plus petit que le 30 ({:.2}\")",
            early.eq_diameter_arcsec,
            late.eq_diameter_arcsec
        );
        assert!(
            early.illuminated > late.illuminated,
            "fraction éclairée le 1er avril {:.3} devrait dépasser celle du 30 ({:.3})",
            early.illuminated,
            late.illuminated
        );
    }

    #[test]
    fn venus_is_nearly_new_at_the_2012_transit() {
        let venus = appearance(Body::Venus, jd(2012, 6, 6));
        assert!(
            venus.illuminated < 0.05,
            "fraction éclairée {:.3}, attendue presque nulle",
            venus.illuminated
        );
        assert!(
            (50.0..80.0).contains(&venus.eq_diameter_arcsec),
            "diamètre {:.2}\", attendu entre 50\" et 80\"",
            venus.eq_diameter_arcsec
        );
    }

    #[test]
    fn every_body_has_a_finite_disk_and_a_right_handed_pole() {
        let when = jd(2012, 4, 15);
        for body in Body::ALL {
            let app = appearance(body, when);
            assert!(app.eq_diameter_arcsec.is_finite() && app.eq_diameter_arcsec > 0.0);
            assert!((0.0..=1.0).contains(&app.illuminated));
            assert!(app.sub_observer_lat_rad.abs() <= std::f64::consts::FRAC_PI_2 + 1e-9);
            assert!(app.pole_pa_rad.is_finite() && app.bright_limb_pa_rad.is_finite());
            let o = app.orientation;
            assert!((o.z_axis.length() - 1.0).abs() < 1e-9);
            assert!(o.x_axis.dot(o.y_axis).abs() < 1e-9);
            assert!((o.x_axis.cross(o.y_axis).dot(o.z_axis) - 1.0).abs() < 1e-6);
            assert!((o.to_observer.length() - 1.0).abs() < 1e-9);
            assert!((o.to_sun.length() - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn moon_matches_the_meeus_1992_example() {
        // 12 avril 1992, 0 h TT. Les séries courtes restent à quelques secondes d'arc
        // de l'exemple ELP (133,162655°, −3,229126°, 368 409,7 km).
        let place = moon_ecliptic(2_448_724.5);
        let lon = place.lon_deg.rem_euclid(360.0);
        assert!((lon - 133.162_655).abs() < 0.01, "longitude {lon:.5}°");
        assert!(
            (place.lat_deg + 3.229_126).abs() < 0.005,
            "latitude {:.5}°",
            place.lat_deg
        );
        assert!(
            (place.dist_km - 368_409.7).abs() < 30.0,
            "distance {:.1} km",
            place.dist_km
        );
    }

    #[test]
    fn moon_is_full_and_large_on_6_may_2012() {
        let dt = Utc.with_ymd_and_hms(2012, 5, 6, 3, 35, 0).unwrap();
        let moon = appearance(Body::Moon, julian_day_utc(dt));
        assert!(
            moon.illuminated > 0.97,
            "fraction éclairée {:.4}",
            moon.illuminated
        );
        assert!(
            moon.eq_diameter_arcsec > 1_900.0,
            "diamètre {:.1}\"",
            moon.eq_diameter_arcsec
        );
    }

    #[test]
    fn sun_is_larger_in_january_than_in_july() {
        let january = sun_apparent(jd(2026, 1, 4));
        let july = sun_apparent(jd(2026, 7, 6));
        assert!(
            january.eq_diameter_arcsec > july.eq_diameter_arcsec,
            "janvier {:.1}\" devrait dépasser juillet {:.1}\"",
            january.eq_diameter_arcsec,
            july.eq_diameter_arcsec
        );
        let ratio = january.eq_diameter_arcsec / july.eq_diameter_arcsec;
        assert!(
            (1.025..1.05).contains(&ratio),
            "rapport janvier/juillet {ratio:.4}, attendu autour de 1,034"
        );
        assert!(
            (0.97..0.99).contains(&january.distance_au),
            "périhélie {:.4} UA",
            january.distance_au
        );
        assert!(
            (1.01..1.03).contains(&july.distance_au),
            "aphélie {:.4} UA",
            july.distance_au
        );
        assert!(january.eq_diameter_arcsec > 1_900.0 && july.eq_diameter_arcsec > 1_860.0);
    }

    #[test]
    fn moon_libration_stays_within_the_limb() {
        for (year, month, day) in [(1992, 4, 12), (2012, 5, 6), (2026, 11, 26)] {
            let moon = appearance(Body::Moon, jd(year, month, day));
            let lat = moon.sub_observer_lat_rad.to_degrees();
            assert!(
                lat.abs() < 8.0,
                "{year}-{month}-{day} : latitude sub-terrestre {lat:.2}°"
            );
            assert!(moon.eq_diameter_arcsec > 1_700.0 && moon.eq_diameter_arcsec < 2_100.0);
            let o = moon.orientation;
            assert!((o.x_axis.cross(o.y_axis).dot(o.z_axis) - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn geocentric_appearance_matches_the_earth_viewpoint() {
        let when = jd(2012, 4, 15);
        let from_helper = appearance(Body::Mars, when);
        let from_explicit = appearance_from(Body::Mars, Viewpoint::Earth, when);
        assert!((from_helper.distance_au - from_explicit.distance_au).abs() < 1e-12);
        assert!((from_helper.illuminated - from_explicit.illuminated).abs() < 1e-12);
        assert!((from_helper.eq_diameter_arcsec - from_explicit.eq_diameter_arcsec).abs() < 1e-9);
    }

    #[test]
    fn earth_seen_from_mars_is_a_finite_disk() {
        let app = appearance_from(Body::Earth, Viewpoint::Mars, jd(2012, 4, 15));
        assert!(
            (0.3..3.0).contains(&app.distance_au),
            "distance {:.3} UA",
            app.distance_au
        );
        assert!(app.eq_diameter_arcsec.is_finite() && app.eq_diameter_arcsec > 0.0);
        assert!((0.0..=1.0).contains(&app.illuminated));
        assert!((app.orientation.to_observer.length() - 1.0).abs() < 1e-9);
        assert!((app.orientation.to_sun.length() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn jupiter_sun_distance_is_the_heliocentric_radius() {
        let when = jd(2012, 4, 15);
        let c = vsop87::vsop87a::jupiter(when);
        let dist = (c.x * c.x + c.y * c.y + c.z * c.z).sqrt();
        let sun = sun_apparent_from(when, Viewpoint::Jupiter);
        assert!(
            (sun.distance_au - dist).abs() < 1e-9,
            "soleil {:.6} UA, vecteur {:.6} UA",
            sun.distance_au,
            dist
        );
        let earth = sun_apparent(when);
        assert!(sun.distance_au > earth.distance_au);
        assert!(sun.eq_diameter_arcsec < earth.eq_diameter_arcsec);
    }
}
