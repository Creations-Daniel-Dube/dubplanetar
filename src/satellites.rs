//! Satellites vus de la Terre.
//!
//! Jupiter suit la série courte de Meeus (chapitre 44, théorie E2 allégée).
//! Saturne suit Dourneau 1993, dans la forme du chapitre 46 de Meeus.
//! Uranus, Mars et Triton suivent des éléments moyens publiés (Jacobson et
//! suiv.) : à la distance de la Terre, les termes courts manquants restent
//! sous la minute d'arc. VSOP87 ne donne que la planète.

use crate::bodies::Body;
use crate::ephemeris::{self, Appearance, Vec3};

const AU_KM: f64 = 149_597_870.7;
const ARCSEC_PER_RAD: f64 = 206_264.806_247_096_36;
const SOLAR_RADIUS_KM: f64 = 695_700.0;
const LIGHT_DAYS_PER_AU: f64 = 499.004_783_836 / 86_400.0;
const J2000_JD: f64 = 2_451_545.0;
const B1950_JD: f64 = 2_433_282.423_459_05;
const DEG: f64 = std::f64::consts::PI / 180.0;
const TAU: f64 = std::f64::consts::TAU;
/// Au-delà, le point disparaît, comme un satellite trop faible à l'oculaire.
pub const MAG_LIMIT: f64 = 16.0;

#[derive(Clone, Copy, Debug)]
pub struct Shadow {
    pub east_arcsec: f64,
    pub north_arcsec: f64,
    pub diameter_arcsec: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Satellite {
    pub name: &'static str,
    pub color: [u8; 3],
    pub east_arcsec: f64,
    pub north_arcsec: f64,
    pub diameter_arcsec: f64,
    pub magnitude: f64,
    /// Derrière le disque ou dans l'ombre de la planète.
    pub hidden: bool,
    pub shadow: Option<Shadow>,
}

#[derive(Clone, Debug)]
pub struct MoonSystem {
    pub planet_magnitude: f64,
    /// Demi-largeur du champ qui contient l'orbite la plus large, en secondes d'arc.
    pub outer_arcsec: f64,
    pub moons: Vec<Satellite>,
}

pub fn has_moons(body: Body) -> bool {
    matches!(
        body,
        Body::Mars | Body::Jupiter | Body::Saturn | Body::Uranus | Body::Neptune
    )
}

pub fn system(body: Body, jd: f64) -> Option<MoonSystem> {
    if !has_moons(body) {
        return None;
    }
    let app = ephemeris::appearance(body, jd);
    let (planet, _earth) = light_time_pair(body, jd);
    let geo = planet - ephemeris::heliocentric(Body::Earth, jd);
    let distance_km = geo.length() * AU_KM;
    let sun_dist_km = planet.length() * AU_KM;
    let (eq_km, _) = body.radii_km();
    let pole = app.orientation.z_axis;
    // Jupiter et Saturne incluent déjà le temps de lumière. Les autres non.
    let delayed = jd - light_delay(body, jd);
    let raw = match body {
        Body::Jupiter => jupiter_moons(jd),
        Body::Saturn => saturn_moons(jd),
        Body::Mars => mars_moons(delayed, pole),
        Body::Uranus => uranus_moons(delayed, pole),
        Body::Neptune => triton(delayed, pole),
        _ => Vec::new(),
    };
    let phase_deg = app.phase_angle_rad.to_degrees();
    let planet_magnitude = crate::vision::visual_magnitude(
        body,
        planet.length(),
        app.distance_au,
        phase_deg,
        app.sub_observer_lat_rad.to_degrees(),
    );
    let mut outer = 0.0_f64;
    let mut moons = Vec::with_capacity(raw.len());
    for moon in raw {
        outer = outer.max(moon.a_km);
        let placed = place_moon(
            &moon,
            &app,
            distance_km,
            sun_dist_km,
            eq_km,
            phase_deg,
            body,
        );
        moons.push(placed);
    }
    let outer_arcsec = outer / distance_km.max(1.0) * ARCSEC_PER_RAD;
    Some(MoonSystem {
        planet_magnitude,
        outer_arcsec,
        moons,
    })
}

struct RawMoon {
    name: &'static str,
    color: [u8; 3],
    /// Position planétocentrique, équatorial J2000, en km.
    pos_km: Vec3,
    radius_km: f64,
    a_km: f64,
    v10: f64,
    beta: f64,
    /// Ajout de magnitude (Japet, selon le côté éclairé).
    mag_extra: f64,
    casts_shadow: bool,
}

fn place_moon(
    moon: &RawMoon,
    app: &Appearance,
    distance_km: f64,
    sun_dist_km: f64,
    planet_radius_km: f64,
    phase_deg: f64,
    planet: Body,
) -> Satellite {
    let (east, north) = sky_arcsec(app.orientation.to_observer, moon.pos_km, distance_km);
    let diameter = 2.0 * moon.radius_km / distance_km * ARCSEC_PER_RAD;
    let magnitude = moon.v10
        + 5.0 * (app.distance_au * sun_dist_km / AU_KM).max(1e-12).log10()
        + moon.beta * phase_deg
        + moon.mag_extra;
    let hidden = occulted(moon.pos_km, app.orientation.to_observer, planet_radius_km)
        || in_umbra(
            moon.pos_km,
            app.orientation.to_sun,
            planet_radius_km,
            sun_dist_km,
        );
    let shadow = if moon.casts_shadow && matches!(planet, Body::Jupiter | Body::Saturn) {
        shadow_on_planet(
            moon.pos_km,
            app.orientation.to_sun,
            app.orientation.to_observer,
            planet_radius_km,
        )
        .map(|point| {
            let (east, north) = sky_arcsec(app.orientation.to_observer, point, distance_km);
            Shadow {
                east_arcsec: east,
                north_arcsec: north,
                diameter_arcsec: diameter,
            }
        })
    } else {
        None
    };
    Satellite {
        name: moon.name,
        color: moon.color,
        east_arcsec: east,
        north_arcsec: north,
        diameter_arcsec: diameter,
        magnitude,
        hidden,
        shadow,
    }
}

fn light_delay(body: Body, jd: f64) -> f64 {
    let earth = ephemeris::heliocentric(Body::Earth, jd);
    let planet = ephemeris::heliocentric(body, jd);
    (planet - earth).length() * LIGHT_DAYS_PER_AU
}

fn light_time_pair(body: Body, jd: f64) -> (Vec3, Vec3) {
    let earth = ephemeris::heliocentric(Body::Earth, jd);
    let mut planet = ephemeris::heliocentric(body, jd);
    let light = (planet - earth).length() * LIGHT_DAYS_PER_AU;
    planet = ephemeris::heliocentric(body, jd - light);
    (planet, earth)
}

/// Écart est / nord célestes, en secondes d'arc. L'est est positif vers l'est.
pub(crate) fn sky_arcsec(to_observer: Vec3, delta_km: Vec3, distance_km: f64) -> (f64, f64) {
    let los = -to_observer;
    let (east, north) = celestial_axes(los);
    let scale = ARCSEC_PER_RAD / distance_km.max(1.0);
    (delta_km.dot(east) * scale, delta_km.dot(north) * scale)
}

fn celestial_axes(los: Vec3) -> (Vec3, Vec3) {
    let mut north = Vec3::new(0.0, 0.0, 1.0);
    north = north - los * north.dot(los);
    if north.length() < 1e-8 {
        north = Vec3::new(0.0, 1.0, 0.0);
        north = north - los * north.dot(los);
    }
    let north = north.normalized();
    let east = north.cross(los).normalized();
    (east, north)
}

/// `X` vers l'ouest dans le plan du ciel, `Y` vers le pôle projeté, `Z` positif
/// loin de la Terre. Le résultat est équatorial J2000, en km.
fn meeus_xyz(
    to_observer: Vec3,
    pole: Vec3,
    x_west_km: f64,
    y_pole_km: f64,
    z_away_km: f64,
) -> Vec3 {
    let los = -to_observer;
    let mut pole_sky = pole - los * pole.dot(los);
    if pole_sky.length() < 1e-8 {
        pole_sky = Vec3::new(0.0, 0.0, 1.0);
    }
    let pole_sky = pole_sky.normalized();
    let west = -(pole_sky.cross(los).normalized());
    west * x_west_km + pole_sky * y_pole_km + los * z_away_km
}

fn occulted(pos: Vec3, to_observer: Vec3, radius_km: f64) -> bool {
    let depth = pos.dot(to_observer);
    if depth >= 0.0 {
        return false;
    }
    let sky = pos - to_observer * depth;
    sky.length() < radius_km
}

fn in_umbra(pos: Vec3, to_sun: Vec3, planet_r: f64, sun_dist_km: f64) -> bool {
    let s = to_sun.normalized();
    let along = pos.dot(s);
    if along >= 0.0 {
        return false;
    }
    let d = -along;
    let perp = (pos - s * along).length();
    let length = planet_r * sun_dist_km / (SOLAR_RADIUS_KM - planet_r).max(1.0);
    if d >= length {
        return false;
    }
    let radius = planet_r * (length - d) / length;
    perp < radius
}

fn shadow_on_planet(moon: Vec3, to_sun: Vec3, to_observer: Vec3, radius_km: f64) -> Option<Vec3> {
    let s = to_sun.normalized();
    if moon.dot(s) <= radius_km {
        return None;
    }
    let b = moon.dot(s);
    let c = moon.dot(moon) - radius_km * radius_km;
    let disc = b * b - c;
    if disc <= 0.0 {
        return None;
    }
    let root = disc.sqrt();
    let t = [b - root, b + root]
        .into_iter()
        .filter(|t| *t > 0.0)
        .reduce(f64::min)?;
    let point = moon - s * t;
    if point.dot(to_observer) <= 0.0 {
        return None;
    }
    Some(point)
}

fn equator_node(pole: Vec3) -> (Vec3, Vec3) {
    let mut node = Vec3::new(-pole.y, pole.x, 0.0);
    if node.length() < 1e-10 {
        node = Vec3::new(1.0, 0.0, 0.0);
    }
    let node = node.normalized();
    let y = pole.cross(node);
    (node, y)
}

fn equator_to_j2000(pole: Vec3, eq: Vec3) -> Vec3 {
    let (node, y) = equator_node(pole);
    node * eq.x + y * eq.y + pole * eq.z
}

struct Kepler {
    a_km: f64,
    i_deg: f64,
    /// Taux de longitude moyenne, degrés par jour.
    n_deg_day: f64,
    lambda0_deg: f64,
    node0_deg: f64,
    dnode_deg_year: f64,
    epoch_jd: f64,
}

fn kepler_pos(k: &Kepler, jd: f64) -> Vec3 {
    let days = jd - k.epoch_jd;
    let years = days / 365.25;
    let lambda = (k.lambda0_deg + k.n_deg_day * days).to_radians();
    let node = (k.node0_deg + k.dnode_deg_year * years).to_radians();
    let u = lambda - node;
    let (su, cu) = u.sin_cos();
    let (sn, cn) = node.sin_cos();
    let (si, ci) = k.i_deg.to_radians().sin_cos();
    Vec3::new(
        k.a_km * (cu * cn - su * sn * ci),
        k.a_km * (cu * sn + su * cn * ci),
        k.a_km * (su * si),
    )
}

/// Triton est rétrograde : l'anomalie et le nœud sont ceux du tableau équatorial.
fn triton_pos(jd: f64) -> Vec3 {
    let epoch = J2000_JD;
    let days = jd - epoch;
    let years = days / 365.25;
    let m = (63.0 + (360.0 / 5.876_854) * days).to_radians();
    let node = (177.7 + 0.524 * years).to_radians();
    let i = 157.3_f64.to_radians();
    let (su, cu) = m.sin_cos();
    let (sn, cn) = node.sin_cos();
    let (si, ci) = i.sin_cos();
    let a = 354_800.0;
    Vec3::new(
        a * (cu * cn - su * sn * ci),
        a * (cu * sn + su * cn * ci),
        a * (su * si),
    )
}

fn mars_moons(jd: f64, pole: Vec3) -> Vec<RawMoon> {
    // Éléments MAR033, époque 1950 janvier 1,0. Le pôle de Laplace est
    // confondu avec l'équateur de Mars à l'échelle du dessin.
    let epoch = 2_433_282.5;
    let phobos = Kepler {
        a_km: 9_380.0,
        i_deg: 1.075,
        n_deg_day: 1_128.844_415_5,
        lambda0_deg: 92.474 + 150.247 + 164.931,
        node0_deg: 164.931,
        dnode_deg_year: -360.0 / 2.262,
        epoch_jd: epoch,
    };
    let deimos = Kepler {
        a_km: 23_460.0,
        i_deg: 1.793,
        n_deg_day: 285.161_891_9,
        lambda0_deg: 296.230 + 290.496 + 339.600,
        node0_deg: 339.600,
        dnode_deg_year: -360.0 / 54.536,
        epoch_jd: epoch,
    };
    vec![
        raw_kepler(
            "Phobos",
            [140, 120, 100],
            &phobos,
            jd,
            pole,
            11.1,
            11.89,
            0.04,
            false,
        ),
        raw_kepler(
            "Déimos",
            [150, 140, 130],
            &deimos,
            jd,
            pole,
            6.2,
            12.94,
            0.04,
            false,
        ),
    ]
}

fn uranus_moons(jd: f64, pole: Vec3) -> Vec<RawMoon> {
    // Éléments équatoriaux URA182, époque J2000. Les termes courts de GUST86
    // déplacent ces lunes de moins d'une seconde d'arc vues de la Terre.
    let specs = [
        (
            "Miranda",
            [170, 170, 175],
            129_846.0,
            4.4,
            1.413_479,
            73.0 + 154.8 + 100.9,
            100.9,
            -360.0 / 17.787,
            235.8,
            3.79,
            0.02,
        ),
        (
            "Ariel",
            [190, 190, 195],
            190_929.0,
            0.0,
            2.520_379,
            193.5 + 9.6,
            0.0,
            0.0,
            578.9,
            1.69,
            0.02,
        ),
        (
            "Umbriel",
            [150, 155, 160],
            265_986.0,
            0.1,
            4.144_177,
            253.0 + 183.4 + 174.8,
            174.8,
            -360.0 / 129.945,
            584.7,
            2.29,
            0.02,
        ),
        (
            "Titania",
            [200, 205, 210],
            436_298.0,
            0.1,
            8.705_869,
            68.1 + 184.0 + 29.5,
            29.5,
            -360.0 / 1_644.649,
            788.9,
            1.19,
            0.02,
        ),
        (
            "Obéron",
            [180, 175, 170],
            583_511.0,
            0.1,
            13.463_237,
            143.6 + 132.2 + 76.8,
            76.8,
            -360.0 / 192.798,
            761.4,
            1.39,
            0.02,
        ),
    ];
    specs
        .into_iter()
        .map(
            |(name, color, a, i, period, lambda, node, dnode, radius, v10, beta)| {
                let k = Kepler {
                    a_km: a,
                    i_deg: i,
                    n_deg_day: 360.0 / period,
                    lambda0_deg: lambda,
                    node0_deg: node,
                    dnode_deg_year: dnode,
                    epoch_jd: J2000_JD,
                };
                raw_kepler(name, color, &k, jd, pole, radius, v10, beta, false)
            },
        )
        .collect()
}

fn triton(jd: f64, pole: Vec3) -> Vec<RawMoon> {
    vec![RawMoon {
        name: "Triton",
        color: [210, 185, 180],
        pos_km: equator_to_j2000(pole, triton_pos(jd)),
        radius_km: 1_353.4,
        a_km: 354_800.0,
        v10: -1.20,
        beta: 0.02,
        mag_extra: 0.0,
        casts_shadow: false,
    }]
}

fn raw_kepler(
    name: &'static str,
    color: [u8; 3],
    k: &Kepler,
    jd: f64,
    pole: Vec3,
    radius_km: f64,
    v10: f64,
    beta: f64,
    casts_shadow: bool,
) -> RawMoon {
    RawMoon {
        name,
        color,
        pos_km: equator_to_j2000(pole, kepler_pos(k, jd)),
        radius_km,
        a_km: k.a_km,
        v10,
        beta,
        mag_extra: 0.0,
        casts_shadow,
    }
}

/// `(X ouest, Y nord, Z loin de la Terre, rayon)` en rayons équatoriaux.
/// Exemple de Meeus, 16 décembre 1992, 0 h TD : voir les tests.
fn galilean_xy(jd: f64) -> [(f64, f64, f64, f64); 4] {
    let d = jd - J2000_JD;
    let v = (172.74 + 0.001_115_88 * d) * DEG;
    let m = (357.529 + 0.985_600_3 * d) * DEG;
    let (sv, _cv) = v.sin_cos();
    let n = (20.02 + 0.083_085_3 * d) * DEG + 0.329 * DEG * sv;
    let j = (66.115 + 0.902_517_9 * d) * DEG - 0.329 * DEG * sv;
    let (sm, cm) = m.sin_cos();
    let (sn, cn) = n.sin_cos();
    let (s2m, c2m) = (2.0 * m).sin_cos();
    let (s2n, c2n) = (2.0 * n).sin_cos();
    let a = 1.915 * DEG * sm + 0.02 * DEG * s2m;
    let b = 5.555 * DEG * sn + 0.168 * DEG * s2n;
    let k = j + a - b;
    let radius_earth = 1.000_14 - 0.016_71 * cm - 0.000_14 * c2m;
    let radius_jup = 5.208_72 - 0.252_08 * cn - 0.006_11 * c2n;
    let (sk, ck) = k.sin_cos();
    let delta = (radius_jup * radius_jup + radius_earth * radius_earth
        - 2.0 * radius_jup * radius_earth * ck)
        .sqrt();
    let psi = (radius_earth / delta * sk).asin();
    let lambda = (34.35 + 0.083_091 * d) * DEG + 0.329 * DEG * sv + b;
    let ds = 3.12 * DEG * (lambda + 42.8 * DEG).sin();
    let de = ds
        - 2.22 * DEG * psi.sin() * (lambda + 22.0 * DEG).cos()
        - 1.3 * DEG * (radius_jup - delta) / delta * (lambda - 100.5 * DEG).sin();
    let dd = d - delta / 173.0;
    let u1 = 163.806_9 * DEG + 203.405_864_6 * DEG * dd + psi - b;
    let u2 = 358.414 * DEG + 101.291_633_5 * DEG * dd + psi - b;
    let u3 = 5.717_6 * DEG + 50.234_518 * DEG * dd + psi - b;
    let u4 = 224.809_2 * DEG + 21.487_98 * DEG * dd + psi - b;
    let g = 331.18 * DEG + 50.310_482 * DEG * dd;
    let h = 87.45 * DEG + 21.569_231 * DEG * dd;
    let (s212, c212) = (2.0 * (u1 - u2)).sin_cos();
    let (s223, c223) = (2.0 * (u2 - u3)).sin_cos();
    let (sg, cg) = g.sin_cos();
    let (sh, ch) = h.sin_cos();
    let c1 = 0.473 * DEG * s212;
    let c2 = 1.065 * DEG * s223;
    let c3 = 0.165 * DEG * sg;
    let c4 = 0.843 * DEG * sh;
    let r1 = 5.905_7 - 0.024_4 * c212;
    let r2 = 9.396_6 - 0.088_2 * c223;
    let r3 = 14.988_3 - 0.021_6 * cg;
    let r4 = 26.362_7 - 0.193_9 * ch;
    let sde = de.sin();
    let cde = de.cos();
    let one = |u: f64, r: f64| {
        let (su, cu) = u.sin_cos();
        (r * su, -r * cu * sde, -r * cu * cde, r)
    };
    [
        one(u1 + c1, r1),
        one(u2 + c2, r2),
        one(u3 + c3, r3),
        one(u4 + c4, r4),
    ]
}

fn jupiter_moons(jd: f64) -> Vec<RawMoon> {
    let app = ephemeris::appearance(Body::Jupiter, jd);
    let radius = 71_492.0;
    let specs = [
        ("Io", [230, 190, 110], 1_821.6, -1.68, 421_700.0),
        ("Europe", [220, 220, 210], 1_560.8, -1.41, 671_100.0),
        ("Ganymède", [176, 156, 130], 2_634.1, -2.09, 1_070_400.0),
        ("Callisto", [120, 110, 96], 2_410.3, -1.05, 1_882_700.0),
    ];
    galilean_xy(jd)
        .into_iter()
        .zip(specs)
        .map(|((x, y, z, _r), (name, color, rad, v10, a))| RawMoon {
            name,
            color,
            pos_km: meeus_xyz(
                app.orientation.to_observer,
                app.orientation.z_axis,
                x * radius,
                y * radius,
                z * radius,
            ),
            radius_km: rad,
            a_km: a,
            v10,
            beta: 0.022,
            mag_extra: 0.0,
            casts_shadow: true,
        })
        .collect()
}

fn saturn_moons(jd: f64) -> Vec<RawMoon> {
    let app = ephemeris::appearance(Body::Saturn, jd);
    let radius = 60_268.0;
    let (lambda, beta, delta, jde) = saturn_apparent(jd);
    let moons = dourneau(jde, lambda, beta, delta);
    let specs = [
        ("Téthys", [210, 210, 200], 531.1, 0.66, 294_660.0),
        ("Dioné", [205, 205, 198], 561.4, 0.86, 377_400.0),
        ("Rhéa", [200, 200, 195], 763.8, 0.16, 527_040.0),
        ("Titan", [214, 148, 72], 2_574.7, -1.14, 1_221_870.0),
        ("Japet", [168, 160, 150], 734.5, 1.46, 3_560_820.0),
    ];
    moons
        .into_iter()
        .zip(specs)
        .map(|((x, y, z, r), (name, color, rad, v10, a))| {
            let mag_extra = if name == "Japet" {
                // L'hémisphère arrière, vu à l'ouest, est le plus clair.
                -0.85 * (x / r.max(1.0))
            } else {
                0.0
            };
            RawMoon {
                name,
                color,
                pos_km: meeus_xyz(
                    app.orientation.to_observer,
                    app.orientation.z_axis,
                    x * radius,
                    y * radius,
                    z * radius,
                ),
                radius_km: rad,
                a_km: a,
                v10,
                beta: 0.02,
                mag_extra,
                casts_shadow: true,
            }
        })
        .collect()
}

/// Longitude et latitude écliptiques B1950 de Saturne, distance (UA) et instant retardé.
fn saturn_apparent(jd: f64) -> (f64, f64, f64, f64) {
    let earth = ephemeris::heliocentric(Body::Earth, jd);
    let mut saturn = ephemeris::heliocentric(Body::Saturn, jd);
    let light = (saturn - earth).length() * LIGHT_DAYS_PER_AU;
    let jde = jd - light;
    saturn = ephemeris::heliocentric(Body::Saturn, jde);
    let geo = saturn - earth;
    let delta = geo.length().max(1e-9);
    let t = (B1950_JD - J2000_JD) / 36_525.0;
    let at_b1950 = precess_from_j2000(geo, t);
    let eps = obliquity(t);
    let ecl = equatorial_to_ecliptic(at_b1950, eps);
    let lambda = ecl.y.atan2(ecl.x);
    let beta = ecl.z.atan2(ecl.x.hypot(ecl.y));
    (lambda, beta, delta, jde)
}

fn obliquity(t: f64) -> f64 {
    let arcsec = 84_381.448 - 46.8150 * t - 0.000_59 * t * t + 0.001_813 * t * t * t;
    (arcsec / 3600.0).to_radians()
}

fn equatorial_to_ecliptic(v: Vec3, eps: f64) -> Vec3 {
    let (ce, se) = eps.sin_cos();
    Vec3::new(v.x, v.y * ce + v.z * se, -v.y * se + v.z * ce)
}

fn precess_from_j2000(v: Vec3, t: f64) -> Vec3 {
    let arcsec = |seconds: f64| (seconds / 3600.0).to_radians();
    let t2 = t * t;
    let t3 = t2 * t;
    let zeta = arcsec(2306.2181 * t + 0.30188 * t2 + 0.017998 * t3);
    let z = arcsec(2306.2181 * t + 1.09468 * t2 + 0.018203 * t3);
    let theta = arcsec(2004.3109 * t - 0.42665 * t2 - 0.041833 * t3);
    let (sz, cz) = z.sin_cos();
    let (st, ct) = theta.sin_cos();
    let (szeta, czeta) = zeta.sin_cos();
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
        p00 * v.x + p01 * v.y + p02 * v.z,
        p10 * v.x + p11 * v.y + p12 * v.z,
        p20 * v.x + p21 * v.y + p22 * v.z,
    )
}

struct Qs {
    t1: f64,
    t2: f64,
    t4: f64,
    t7: f64,
    t10: f64,
    t11: f64,
    w4: f64,
    w5: f64,
    w6: f64,
    w8: f64,
    s1: f64,
    c1: f64,
    s2: f64,
    c2: f64,
    e1: f64,
    sw0: f64,
    s3w0: f64,
    s5w0: f64,
    sw1: f64,
    sw2: f64,
    sw3: f64,
    cw3: f64,
    sw4: f64,
    cw4: f64,
    sw7: f64,
    cw7: f64,
}

fn new_qs(jde: f64) -> Qs {
    let t1 = jde - 2_411_093.0;
    let t2 = t1 / 365.25;
    let t3 = (jde - 2_433_282.423) / 365.25 + 1950.0;
    let t4 = jde - 2_411_368.0;
    let t5 = t4 / 365.25;
    let t6 = jde - 2_415_020.0;
    let t7 = t6 / 36_525.0;
    let t10 = jde - 2_409_786.0;
    let t11 = t10 / 36_525.0;
    let w0 = 5.095 * DEG * (t3 - 1866.39);
    let w1 = 74.4 * DEG + 32.39 * DEG * t2;
    let w2 = 134.3 * DEG + 92.62 * DEG * t2;
    let w3 = 42.0 * DEG - 0.5118 * DEG * t5;
    let w4 = 276.59 * DEG + 0.5118 * DEG * t5;
    let w5 = 267.2635 * DEG + 1222.1136 * DEG * t7;
    let w6 = 175.4762 * DEG + 1221.5515 * DEG * t7;
    let w7 = 2.4891 * DEG + 0.002_435 * DEG * t7;
    let w8 = 113.35 * DEG - 0.2597 * DEG * t7;
    let (s1, c1) = (28.0817 * DEG).sin_cos();
    let (s2, c2) = (168.8112 * DEG).sin_cos();
    let (sw3, cw3) = w3.sin_cos();
    let (sw4, cw4) = w4.sin_cos();
    let (sw7, cw7) = w7.sin_cos();
    Qs {
        t1,
        t2,
        t4,
        t7,
        t10,
        t11,
        w4,
        w5,
        w6,
        w8,
        s1,
        c1,
        s2,
        c2,
        e1: 0.05589 - 0.000_346 * t7,
        sw0: w0.sin(),
        s3w0: (3.0 * w0).sin(),
        s5w0: (5.0 * w0).sin(),
        sw1: w1.sin(),
        sw2: w2.sin(),
        sw3,
        cw3,
        sw4,
        cw4,
        sw7,
        cw7,
    }
}

struct Orb {
    lambda: f64,
    r: f64,
    gamma: f64,
    omega: f64,
}

fn subr(q: &Qs, lambda_p: f64, p: f64, e: f64, a: f64, omega: f64, i: f64) -> Orb {
    let m = (lambda_p - p).rem_euclid(TAU);
    let e2 = e * e;
    let e3 = e2 * e;
    let e4 = e2 * e2;
    let e5 = e3 * e2;
    let c = (2.0 * e - 0.25 * e3 + 0.052_083_333_3 * e5) * m.sin()
        + (1.25 * e2 - 0.458_333_333 * e4) * (2.0 * m).sin()
        + (1.083_333_333 * e3 - 0.671_875 * e5) * (3.0 * m).sin()
        + 1.072_917 * e4 * (4.0 * m).sin()
        + 1.142_708 * e5 * (5.0 * m).sin();
    let r = a * (1.0 - e2) / (1.0 + e * (m + c).cos());
    let g = omega - 168.8112 * DEG;
    let (si, ci) = i.sin_cos();
    let (sg, cg) = g.sin_cos();
    let a1 = si * sg;
    let a2 = q.c1 * si * cg - q.s1 * ci;
    let gamma = a1.hypot(a2).asin();
    let u = a1.atan2(a2);
    let omega_out = 168.8112 * DEG + u;
    let h = q.c1 * si - q.s1 * ci * cg;
    let psi = (q.s1 * sg).atan2(h);
    Orb {
        lambda: lambda_p + c + u - g - psi,
        r,
        gamma,
        omega: omega_out,
    }
}

fn tethys(q: &Qs) -> Orb {
    Orb {
        lambda: 285.306 * DEG
            + 190.697_912_26 * DEG * q.t1
            + 2.063 * DEG * q.sw0
            + 0.034_09 * DEG * q.s3w0
            + 0.001_015 * DEG * q.s5w0,
        r: 4.880_998,
        gamma: 1.097_6 * DEG,
        omega: 111.33 * DEG - 72.244_1 * DEG * q.t2,
    }
}

fn dione(q: &Qs) -> Orb {
    let l = 254.712 * DEG + 131.534_931_93 * DEG * q.t1
        - 0.021_5 * DEG * q.sw1
        - 0.017_33 * DEG * q.sw2;
    let p = 174.8 * DEG + 30.82 * DEG * q.t2;
    let m = (l - p).rem_euclid(TAU);
    let c = 0.247_17 * DEG * m.sin() + 0.000_33 * DEG * (2.0 * m).sin();
    Orb {
        lambda: l + c,
        r: 6.248_71 / (1.0 + 0.002_157 * (m + c).cos()),
        gamma: 0.013_9 * DEG,
        omega: 232.0 * DEG - 30.27 * DEG * q.t2,
    }
}

fn rhea(q: &Qs) -> Orb {
    let p_prime = 342.7 * DEG + 10.057 * DEG * q.t2;
    let (sp, cp) = p_prime.sin_cos();
    let a1 = 0.000_265 * sp + 0.001 * q.sw4;
    let a2 = 0.000_265 * cp + 0.001 * q.cw4;
    let e = a1.hypot(a2);
    let p = a1.atan2(a2);
    let n = 345.0 * DEG - 10.057 * DEG * q.t2;
    let (sn, cn) = n.sin_cos();
    let lambda_p = 359.244 * DEG + 79.690_047_2 * DEG * q.t1 + 0.086_754 * DEG * sn;
    let i = 28.036_2 * DEG + 0.346_898 * DEG * cn + 0.019_3 * DEG * q.cw3;
    let omega = 168.803_4 * DEG + 0.736_936 * DEG * sn + 0.041 * DEG * q.sw3;
    subr(q, lambda_p, p, e, 8.725_924, omega, i)
}

fn titan(q: &Qs) -> Orb {
    let l = 261.158_2 * DEG + 22.576_978_55 * DEG * q.t4 + 0.074_025 * DEG * q.sw3;
    let i_p = 27.451_41 * DEG + 0.295_999 * DEG * q.cw3;
    let omega_p = 168.669_25 * DEG + 0.628_808 * DEG * q.sw3;
    let (si, ci) = i_p.sin_cos();
    let (so, co) = (omega_p - q.w8).sin_cos();
    let a1 = q.sw7 * so;
    let a2 = q.cw7 * si - q.sw7 * ci * co;
    let g0 = 102.862_3 * DEG;
    let psi = a1.atan2(a2);
    let s = a1.hypot(a2);
    let mut g = q.w4 - omega_p - psi;
    let (s2g0, c2g0) = (2.0 * g0).sin_cos();
    let mut varpi = 0.0;
    for _ in 0..3 {
        varpi = q.w4 + 0.375_15 * DEG * ((2.0 * g).sin() - s2g0);
        g = varpi - omega_p - psi;
    }
    let e_p = 0.029_092 + 0.000_190_48 * ((2.0 * g).cos() - c2g0);
    let qq = 2.0 * (q.w5 - varpi);
    let b1 = si * so;
    let b2 = q.cw7 * si * co - q.sw7 * ci;
    let theta = b1.atan2(b2) + q.w8;
    let (sq, cq) = qq.sin_cos();
    let e = e_p + 0.002_778_797 * e_p * cq;
    let p = varpi + 0.159_215 * DEG * sq;
    let u = 2.0 * q.w5 - 2.0 * theta + psi;
    let (su, cu) = u.sin_cos();
    let h = 0.9375 * e_p * e_p * sq + 0.1875 * s * s * (2.0 * (q.w5 - theta)).sin();
    let lambda_p =
        l - 0.254_744 * DEG * (q.e1 * q.w6.sin() + 0.75 * q.e1 * q.e1 * (2.0 * q.w6).sin() + h);
    let i = i_p + 0.031_843 * DEG * s * cu;
    let omega = omega_p + 0.031_843 * DEG * s * su / si;
    subr(q, lambda_p, p, e, 20.216_193, omega, i)
}

fn iapetus(q: &Qs) -> Orb {
    let l = 261.158_2 * DEG + 22.576_978_55 * DEG * q.t4;
    let varpi_p = 91.796 * DEG + 0.562 * DEG * q.t7;
    let psi = 4.367 * DEG - 0.195 * DEG * q.t7;
    let theta = 146.819 * DEG - 3.198 * DEG * q.t7;
    let phi = 60.47 * DEG + 1.521 * DEG * q.t7;
    let phi_big = 205.055 * DEG - 2.091 * DEG * q.t7;
    let e_p = 0.028_298 + 0.001_156 * q.t11;
    let varpi0 = 352.91 * DEG + 11.71 * DEG * q.t11;
    let mu = 76.385_2 * DEG + 4.537_951_25 * DEG * q.t10;
    let i_p = horner(
        q.t11,
        18.460_2 * DEG,
        -0.951_8 * DEG,
        -0.072 * DEG,
        0.005_4 * DEG,
    );
    let omega_p = horner(q.t11, 143.198 * DEG, -3.919 * DEG, 0.116 * DEG, 0.008 * DEG);
    let el = mu - varpi0;
    let g1 = varpi0 - omega_p - phi;
    let ls = q.w5 - varpi_p;
    let lt = l - q.w4;
    let gt = q.w4 - phi_big;
    let u1 = 2.0 * (el + (varpi0 - omega_p - psi) - ls - (varpi_p - theta));
    let u2 = el + g1 - lt - gt;
    let u3 = el + 2.0 * ((varpi0 - omega_p - psi) - ls - (varpi_p - theta));
    let u4 = lt + gt - g1;
    let u5 = 2.0 * (ls + (varpi_p - theta));
    let (sl, cl) = el.sin_cos();
    let (su1, cu1) = u1.sin_cos();
    let (su2, cu2) = u2.sin_cos();
    let (su3, cu3) = u3.sin_cos();
    let (su4, cu4) = u4.sin_cos();
    let (slu2, clu2) = (el + u2).sin_cos();
    let (sg, cg) = (g1 - gt).sin_cos();
    let (su52, cu52) = (u5 - 2.0 * (varpi0 - omega_p - psi)).sin_cos();
    let (su5p, cu5p) = (u5 + psi).sin_cos();
    let (su2p, cu2p) = (u2 + phi).sin_cos();
    let (s5, c5) = (el + g1 + lt + gt + phi).sin_cos();
    let a = 58.935_028 + 0.004_638 * cu1 + 0.058_222 * cu2;
    let e = e_p - 0.001_409_7 * cg
        + 0.000_373_3 * cu52
        + 0.000_118 * cu3
        + 0.000_240_8 * cl
        + 0.000_284_9 * clu2
        + 0.000_619 * cu4;
    let w = 0.080_77 * DEG * sg + 0.021_39 * DEG * su52 - 0.006_76 * DEG * su3
        + 0.013_8 * DEG * sl
        + 0.016_32 * DEG * slu2
        + 0.035_47 * DEG * su4;
    let p = varpi0 + w / e_p;
    let lambda_p = mu
        - 0.042_99 * DEG * su2
        - 0.007_89 * DEG * su1
        - 0.063_12 * DEG * ls.sin()
        - 0.002_95 * DEG * (2.0 * ls).sin()
        - 0.022_31 * DEG * u5.sin()
        + 0.006_5 * DEG * su5p;
    let i = i_p + 0.042_04 * DEG * cu5p + 0.002_35 * DEG * c5 + 0.003_6 * DEG * cu2p;
    let w_prime = 0.042_04 * DEG * su5p + 0.002_35 * DEG * s5 + 0.003_58 * DEG * su2p;
    let omega = omega_p + w_prime / i_p.sin();
    subr(q, lambda_p, p, e, a, omega, i)
}

fn horner(x: f64, a: f64, b: f64, c: f64, d: f64) -> f64 {
    a + x * (b + x * (c + x * d))
}

/// `(X ouest, Y nord, Z loin de la Terre, rayon)` en rayons de Saturne.
fn dourneau(jde: f64, lambda0: f64, beta0: f64, delta: f64) -> [(f64, f64, f64, f64); 5] {
    let q = new_qs(jde);
    let orbs = [tethys(&q), dione(&q), rhea(&q), titan(&q), iapetus(&q)];
    let mut xyz = [(0.0, 0.0, 0.0); 6];
    xyz[0] = (0.0, 0.0, 1.0);
    for (slot, orb) in orbs.iter().enumerate() {
        let u = orb.lambda - orb.omega;
        let w = orb.omega - 168.8112 * DEG;
        let (su, cu) = u.sin_cos();
        let (sw, cw) = w.sin_cos();
        let (sg, cg) = orb.gamma.sin_cos();
        let r = orb.r;
        xyz[slot + 1] = (
            r * (cu * cw - su * cg * sw),
            r * (su * cw * cg + cu * sw),
            r * su * sg,
        );
    }
    let (s_l, c_l) = lambda0.sin_cos();
    let (s_b, c_b) = beta0.sin_cos();
    let mut abc = [(0.0, 0.0, 0.0); 6];
    for (slot, (x, y, z)) in xyz.into_iter().enumerate() {
        let a0 = x;
        let b0 = q.c1 * y - q.s1 * z;
        let c0 = q.s1 * y + q.c1 * z;
        let a1 = q.c2 * a0 - q.s2 * b0;
        let b1 = q.s2 * a0 + q.c2 * b0;
        let a2 = a1 * s_l - b1 * c_l;
        let b2 = a1 * c_l + b1 * s_l;
        abc[slot] = (a2, b2 * c_b + c0 * s_b, c0 * c_b - b2 * s_b);
    }
    let d_angle = abc[0].0.atan2(abc[0].2);
    let (sd, cd) = d_angle.sin_cos();
    let mut out = [(0.0, 0.0, 0.0, 0.0); 5];
    for slot in 0..5 {
        let (a, _b, c) = abc[slot + 1];
        let mut x = a * cd - c * sd;
        let y = a * sd + c * cd;
        let z = abc[slot + 1].1;
        let r = orbs[slot].r.max(1e-6);
        let ratio = (x / r).clamp(-1.0, 1.0);
        let k = [
            20_947.0, 23_715.0, 26_382.0, 29_876.0, 35_313.0, 53_800.0, 59_222.0, 91_820.0,
        ];
        // Téthys, Dioné, Rhéa, Titan, Japet : indices 3, 4, 5, 6, 8 de Meeus.
        let kk = [k[2], k[3], k[4], k[5], k[7]][slot];
        x += z.abs() / kk * (1.0 - ratio * ratio).sqrt();
        let w = delta / (delta + z / 2475.0);
        out[slot] = (x * w, y * w, z, r);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_on_the_sky_keeps_the_meeus_separation() {
        let jd = 2_448_972.500_68;
        let app = ephemeris::appearance(Body::Jupiter, jd);
        let moons = system(Body::Jupiter, jd).unwrap();
        let io = moons.moons.iter().find(|moon| moon.name == "Io").unwrap();
        let radii = io.east_arcsec.hypot(io.north_arcsec) / (app.eq_diameter_arcsec * 0.5);
        assert!(
            (radii - 3.45).abs() < 0.08,
            "Io à {radii:.3} rayons, attendu 3,45"
        );
    }

    #[test]
    fn meeus_galilean_example_44a() {
        let [io, europa, ganymede, callisto] = galilean_xy(2_448_972.500_68);
        let expect = [(-3.44, 0.21), (7.44, 0.25), (1.24, 0.65), (7.08, 1.10)];
        for ((x, y, _, _), (ex, ey)) in [io, europa, ganymede, callisto].into_iter().zip(expect) {
            assert!((x - ex).abs() < 0.02, "X {x} attendu {ex}");
            assert!((y - ey).abs() < 0.02, "Y {y} attendu {ey}");
        }
    }

    #[test]
    fn ganymede_is_brighter_than_callisto() {
        let moons = system(Body::Jupiter, J2000_JD).unwrap();
        let mag = |name| {
            moons
                .moons
                .iter()
                .find(|moon| moon.name == name)
                .unwrap()
                .magnitude
        };
        assert!(mag("Ganymède") < mag("Callisto"));
        assert!(mag("Ganymède") < mag("Io"));
    }

    #[test]
    fn phobos_stays_closer_than_deimos() {
        let epoch = 2_433_282.5;
        let phobos = Kepler {
            a_km: 9_380.0,
            i_deg: 1.075,
            n_deg_day: 1_128.844_415_5,
            lambda0_deg: 92.474 + 150.247 + 164.931,
            node0_deg: 164.931,
            dnode_deg_year: -360.0 / 2.262,
            epoch_jd: epoch,
        };
        let deimos = Kepler {
            a_km: 23_460.0,
            i_deg: 1.793,
            n_deg_day: 285.161_891_9,
            lambda0_deg: 296.230 + 290.496 + 339.600,
            node0_deg: 339.600,
            dnode_deg_year: -360.0 / 54.536,
            epoch_jd: epoch,
        };
        let phobos_km = kepler_pos(&phobos, J2000_JD).length();
        let deimos_km = kepler_pos(&deimos, J2000_JD).length();
        assert!(
            phobos_km < 10_000.0 && deimos_km > 23_000.0,
            "{phobos_km} {deimos_km}"
        );
        let moons = system(Body::Mars, J2000_JD).unwrap();
        assert!(moons.outer_arcsec > 0.0);
        assert_eq!(moons.moons.len(), 2);
    }

    #[test]
    fn a_moon_behind_the_disk_is_hidden() {
        let to_observer = Vec3::new(1.0, 0.0, 0.0);
        let behind = to_observer * -500.0;
        assert!(occulted(behind, to_observer, 1_000.0));
        let in_front = to_observer * 500.0;
        assert!(!occulted(in_front, to_observer, 1_000.0));
        let beside = Vec3::new(-100.0, 2_000.0, 0.0);
        assert!(!occulted(beside, to_observer, 1_000.0));
    }

    #[test]
    fn north_and_west_offsets_match_the_sky() {
        let to_observer = Vec3::new(1.0, 0.0, 0.0);
        let distance = AU_KM;
        let (east, north) = sky_arcsec(to_observer, Vec3::new(0.0, 0.0, 1_000.0), distance);
        assert!(north > 0.0, "nord {north}");
        assert!(east.abs() < 1e-6, "est {east}");
        let (east, north) = sky_arcsec(to_observer, Vec3::new(0.0, 1_000.0, 0.0), distance);
        assert!(east < 0.0, "l'est d'un décalage ouest est négatif : {east}");
        assert!(north.abs() < 1e-6, "nord {north}");

        let pole = Vec3::new(0.0, 0.0, 1.0);
        let west = meeus_xyz(to_observer, pole, 1_000.0, 0.0, 0.0);
        let (east, _) = sky_arcsec(to_observer, west, distance);
        assert!(east < 0.0, "X de Meeus part vers l'ouest : {east}");
        let toward_pole = meeus_xyz(to_observer, pole, 0.0, 1_000.0, 0.0);
        let (_, north) = sky_arcsec(to_observer, toward_pole, distance);
        assert!(north > 0.0, "Y de Meeus part vers le pôle : {north}");
    }

    #[test]
    fn each_planet_has_the_expected_moons() {
        assert!(!has_moons(Body::Mercury) && !has_moons(Body::Venus));
        let count = |body| system(body, J2000_JD).unwrap().moons.len();
        assert_eq!(count(Body::Mars), 2);
        assert_eq!(count(Body::Jupiter), 4);
        assert_eq!(count(Body::Saturn), 5);
        assert_eq!(count(Body::Uranus), 5);
        assert_eq!(count(Body::Neptune), 1);
        let saturn = system(Body::Saturn, J2000_JD).unwrap();
        for moon in &saturn.moons {
            let span = moon.east_arcsec.hypot(moon.north_arcsec);
            assert!(span < 1_200.0, "{} trop loin : {span}", moon.name);
        }
        let titan = saturn
            .moons
            .iter()
            .find(|moon| moon.name == "Titan")
            .unwrap();
        let titan_span = titan.east_arcsec.hypot(titan.north_arcsec);
        assert!(
            (80.0..350.0).contains(&titan_span),
            "Titan à {titan_span:.1}\""
        );
        let uranus = system(Body::Uranus, J2000_JD).unwrap();
        let mag = |name: &str| {
            uranus
                .moons
                .iter()
                .find(|moon| moon.name == name)
                .unwrap()
                .magnitude
        };
        assert!(mag("Titania") < mag("Miranda"));
    }
}
