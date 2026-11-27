//! Fenêtre des satellites, ouverte par un double-clic depuis la Terre.

use std::sync::Arc;

use image::{Rgba, RgbaImage};

use crate::bodies::{self, Body};
use crate::disk::{self, DiskInput};
use crate::ephemeris::{self, Appearance};
use crate::raster_pump::{DiskJob, RasterPump};
use crate::satellites::{self, MAG_LIMIT, MoonSystem, Satellite};
use crate::state::{self, Observation, Viewpoint};
use crate::vision;

const LABEL: egui::Color32 = egui::Color32::from_rgb(235, 235, 240);
const DIM: egui::Color32 = egui::Color32::from_rgb(180, 190, 205);

pub struct SatelliteWindow {
    zoom: f64,
    shown: Option<Body>,
    pump: RasterPump,
    token: u64,
    pending: Option<DiskKey>,
    cache: Option<Cached>,
    map_body: Option<Body>,
    map_stamp: Option<(u64, u64)>,
    map: Option<Arc<RgbaImage>>,
    moon_tex: Vec<MoonTex>,
    close_request: bool,
}

struct MoonTex {
    name: String,
    radius_px: i32,
    gain_q: i32,
    real: bool,
    texture: egui::TextureHandle,
}

#[derive(Clone, PartialEq)]
struct DiskKey {
    body: Body,
    minute: i64,
    radius_px: i32,
    stamp: Option<(u64, u64)>,
    axis: bool,
}

struct Cached {
    key: DiskKey,
    north_up: RgbaImage,
    gain_q: i32,
    real: bool,
    north_down: bool,
    texture: egui::TextureHandle,
}

impl SatelliteWindow {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            shown: None,
            pump: RasterPump::new(),
            token: 0,
            pending: None,
            cache: None,
            map_body: None,
            map_stamp: None,
            map: None,
            moon_tex: Vec::new(),
            close_request: false,
        }
    }

    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        observation: &mut Observation,
        body: &mut Option<Body>,
    ) {
        if observation.viewpoint != Viewpoint::Earth {
            *body = None;
        }
        let Some(planet) = *body else {
            self.shown = None;
            return;
        };
        if self.shown != Some(planet) {
            self.shown = Some(planet);
            self.zoom = 1.0;
        }
        let mut open = true;
        let title = format!("{} et ses satellites", planet.name_fr());
        let jd = ephemeris::julian_day_utc(observation.when.with_timezone(&chrono::Utc));
        let app = ephemeris::appearance(planet, jd);
        let system = satellites::system(planet, jd);
        egui::Window::new(title)
            .id(egui::Id::unique("satellite-window"))
            .open(&mut open)
            .default_size([780.0, 700.0])
            .min_size([480.0, 420.0])
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                self.contents(ui, observation, planet, jd, &app, system.as_ref());
            });
        if !open || self.close_request {
            *body = None;
            self.shown = None;
            self.close_request = false;
        }
    }

    fn contents(
        &mut self,
        ui: &mut egui::Ui,
        observation: &mut Observation,
        planet: Body,
        jd: f64,
        app: &Appearance,
        system: Option<&MoonSystem>,
    ) {
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.close_request = true;
        }
        ui.checkbox(&mut observation.real_vision, "Vision réelle des objets");
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "{}   ·   heure locale",
                state::format_local(observation.when)
            ))
            .color(LABEL)
            .size(15.0),
        );
        let north = if observation.north_down {
            "Nord vers le bas"
        } else {
            "Nord vers le haut"
        };
        ui.label(
            egui::RichText::new(format!(
                "{north}. Molette pour zoomer. La croix ou Échap ferment la fenêtre."
            ))
            .color(DIM)
            .size(13.0),
        );
        let sky = ui.available_rect_before_wrap();
        if sky.width() < 40.0 || sky.height() < 40.0 {
            return;
        }
        let response = ui.allocate_rect(sky, egui::Sense::hover());
        let Some(system) = system else {
            return;
        };
        let field = (system.outer_arcsec * 2.0 * 1.18).max(app.eq_diameter_arcsec * 4.0);
        let side = f64::from(sky.width().min(sky.height()));
        let base = side / field;
        let zoom_max = (0.82 * field / app.eq_diameter_arcsec.max(0.2)).max(1.0);
        if response.hovered() {
            let wheel = ui.input(|i| i.zoom_delta());
            if (wheel - 1.0).abs() > 0.001 {
                self.zoom = (self.zoom * f64::from(wheel)).clamp(1.0, zoom_max);
            }
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.0 {
                let factor = (1.0 + f64::from(scroll) * 0.001_2).clamp(0.75, 1.3);
                self.zoom = (self.zoom * factor).clamp(1.0, zoom_max);
            }
        }
        let scale = base * self.zoom;
        let ppp = f64::from(ui.ctx().pixels_per_point());
        let radius_px = quantize_radius(app.eq_diameter_arcsec * 0.5 * scale * ppp);
        let real = observation.real_vision;
        let gain = if real {
            vision::gain_for(planet, app, jd)
        } else {
            disk_gain(system.planet_magnitude, app.eq_diameter_arcsec, scale)
        };
        let minute = (jd * 1_440.0).round() as i64;
        let busy = self.sync(
            ui.ctx(),
            planet,
            app,
            minute,
            radius_px,
            gain,
            real,
            observation.north_down,
        );
        if busy {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(40));
        }
        let painter = ui.painter().clone();
        let center = sky.center();
        let mut label_color = LABEL;
        if real {
            if let Some(backdrop) = vision::sky_behind(
                planet,
                Viewpoint::Earth,
                app,
                jd,
                vision::longitude_deg(observation.when),
            ) {
                painter.rect_filled(sky, 0.0, backdrop.color);
                if backdrop.bright {
                    label_color = egui::Color32::from_rgb(16, 22, 40);
                }
            }
            let radius = (app.eq_diameter_arcsec * 0.5 * scale) as f32;
            let clip = painter.with_clip_rect(sky);
            vision::paint_glare(
                &clip,
                ui.ctx(),
                center,
                radius,
                gain,
                app.illuminated,
                vision::glare_of(planet, gain, app.illuminated),
            );
        }
        if let Some(cached) = &self.cache {
            let [w, h] = cached.texture.size();
            let display = egui::vec2(w as f32 / ppp as f32, h as f32 / ppp as f32);
            let rect = egui::Rect::from_center_size(center, display);
            painter.image(
                cached.texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
        let sign = if observation.north_down { 1.0 } else { -1.0 };
        for moon in &system.moons {
            if let Some(shadow) = moon.shadow {
                paint_shadow(&painter, center, sign, scale, &shadow);
            }
        }
        let moons: Vec<Satellite> = system.moons.clone();
        for moon in &moons {
            self.paint_moon(
                &painter,
                ui.ctx(),
                center,
                sign,
                scale,
                ppp,
                app,
                moon,
                real,
                label_color,
            );
        }
        paint_scale(&painter, sky, scale, label_color);
    }

    fn sync(
        &mut self,
        ctx: &egui::Context,
        planet: Body,
        app: &Appearance,
        minute: i64,
        radius_px: i32,
        gain: f64,
        real: bool,
        north_down: bool,
    ) -> bool {
        let gain_q = if real {
            vision::quantize_gain(gain)
        } else {
            (gain * 40.0).round() as i32
        };
        for done in self.pump.poll() {
            if self.token == done.token {
                if let Some(key) = self.pending.clone() {
                    if key.axis == !real {
                        self.install(
                            ctx,
                            key,
                            gain_q,
                            gain,
                            real,
                            north_down,
                            planet,
                            app.illuminated,
                            done.image,
                        );
                    }
                }
            }
        }
        let stamp = bodies::texture_stamp(planet);
        let key = DiskKey {
            body: planet,
            minute,
            radius_px,
            stamp,
            axis: !real,
        };
        if let Some(cached) = &mut self.cache {
            if cached.key == key {
                if cached.north_down != north_down || cached.gain_q != gain_q || cached.real != real
                {
                    let color = grade_disk(
                        &cached.north_up,
                        gain,
                        real,
                        north_down,
                        planet,
                        app.illuminated,
                    );
                    cached.texture.set(color, egui::TextureOptions::LINEAR);
                    cached.north_down = north_down;
                    cached.gain_q = gain_q;
                    cached.real = real;
                }
                return false;
            }
        }
        let same = self.pending.as_ref() == Some(&key);
        if !same {
            self.token = self.pump.bump_token();
            let texture = self.map_for(planet, stamp);
            self.pump.submit(DiskJob {
                token: self.token,
                id: 1,
                orientation: app.orientation,
                flattening: app.flattening,
                albedo: planet.albedo(),
                texture,
                texture_path: None,
                lon_offset_turns: planet.texture_lon_offset(),
                bump: None,
                bump_path: None,
                rings: planet.ring_zones(),
                draw_axis: !real,
                radius_px: f64::from(radius_px),
                max_side: 1000,
            });
            self.pending = Some(key);
        }
        true
    }

    fn map_for(&mut self, body: Body, stamp: Option<(u64, u64)>) -> Option<Arc<RgbaImage>> {
        if self.map_body == Some(body) && self.map_stamp == stamp {
            return self.map.clone();
        }
        let image = stamp.and_then(|_| bodies::load_texture(body)).map(Arc::new);
        self.map_body = Some(body);
        self.map_stamp = stamp;
        self.map = image.clone();
        image
    }

    fn install(
        &mut self,
        ctx: &egui::Context,
        key: DiskKey,
        gain_q: i32,
        gain: f64,
        real: bool,
        north_down: bool,
        planet: Body,
        illuminated: f64,
        north_up: RgbaImage,
    ) {
        let color = grade_disk(&north_up, gain, real, north_down, planet, illuminated);
        if let Some(cached) = &mut self.cache {
            cached.texture.set(color, egui::TextureOptions::LINEAR);
            cached.key = key;
            cached.north_up = north_up;
            cached.gain_q = gain_q;
            cached.real = real;
            cached.north_down = north_down;
        } else {
            let texture = ctx.load_texture("satellite-planet", color, egui::TextureOptions::LINEAR);
            self.cache = Some(Cached {
                key,
                north_up,
                gain_q,
                real,
                north_down,
                texture,
            });
        }
    }
}

fn grade_disk(
    src: &RgbaImage,
    gain: f64,
    real: bool,
    north_down: bool,
    body: Body,
    illuminated: f64,
) -> egui::ColorImage {
    if real {
        vision::present_disk(
            src,
            north_down,
            Some(gain),
            vision::cast_of(body),
            vision::glare_of(body, gain, illuminated),
        )
    } else {
        colorize(src, gain, north_down)
    }
}

fn quantize_radius(radius_px: f64) -> i32 {
    let step = 2.0_f64;
    (radius_px / step).round().max(1.0) as i32 * step as i32
}

/// 1 à la magnitude 5, 0 au-delà de 16. L'étirement en puissance garde
/// Ganymède clair et un satellite de magnitude 15 à peine visible.
pub(crate) fn point_gain(magnitude: f64) -> f64 {
    if magnitude > MAG_LIMIT {
        return 0.0;
    }
    let flux = 10_f64.powf(-0.4 * (magnitude - 5.0));
    flux.powf(0.3).clamp(0.0, 1.0)
}

fn disk_gain(planet_mag: f64, diameter_arcsec: f64, px_per_arcsec: f64) -> f64 {
    let radius = (diameter_arcsec * 0.5).max(0.05);
    let area = std::f64::consts::PI * radius * radius;
    let pixel = (1.0 / px_per_arcsec.max(1e-4)).powi(2);
    let per_pixel = planet_mag + 2.5 * area.log10() - 2.5 * pixel.log10();
    point_gain(per_pixel).clamp(0.32, 1.0)
}

fn colorize(src: &RgbaImage, gain: f64, north_down: bool) -> egui::ColorImage {
    let owned;
    let image = if north_down {
        owned = flip_180(src);
        &owned
    } else {
        src
    };
    let mut pixels = image.as_raw().clone();
    for px in pixels.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        for channel in &mut px[..3] {
            *channel = (f64::from(*channel) * gain).round().clamp(0.0, 255.0) as u8;
        }
    }
    egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        &pixels,
    )
}

fn flip_180(src: &RgbaImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            dst.put_pixel(w - 1 - x, h - 1 - y, *src.get_pixel(x, y));
        }
    }
    dst
}

fn paint_shadow(
    painter: &egui::Painter,
    center: egui::Pos2,
    sign: f32,
    scale: f64,
    shadow: &crate::satellites::Shadow,
) {
    let diameter = shadow.diameter_arcsec * scale;
    if diameter < 1.2 {
        return;
    }
    let at = place(center, sign, scale, shadow.east_arcsec, shadow.north_arcsec);
    painter.circle_filled(
        at,
        (diameter as f32 * 0.5).max(1.2),
        egui::Color32::from_black_alpha(170),
    );
}

impl SatelliteWindow {
    fn paint_moon(
        &mut self,
        painter: &egui::Painter,
        ctx: &egui::Context,
        center: egui::Pos2,
        sign: f32,
        scale: f64,
        ppp: f64,
        app: &Appearance,
        moon: &Satellite,
        real: bool,
        label_color: egui::Color32,
    ) {
        let gain = if real {
            vision::real_point_gain(moon.magnitude)
        } else {
            point_gain(moon.magnitude)
        };
        if moon.hidden || gain <= 0.0 {
            return;
        }
        let at = place(center, sign, scale, moon.east_arcsec, moon.north_arcsec);
        let diameter_px = moon.diameter_arcsec * scale * ppp;
        if diameter_px >= 3.0 {
            let radius_px = (diameter_px * 0.5).round().max(2.0) as i32;
            let gain_q = if real {
                vision::quantize_gain(gain)
            } else {
                (gain * 40.0).round() as i32
            };
            let texture = self.moon_disk(ctx, moon, app, radius_px, gain_q, gain, real);
            let ppp_f = ppp as f32;
            let size = texture.size();
            let display = egui::vec2(size[0] as f32 / ppp_f, size[1] as f32 / ppp_f);
            painter.image(
                texture.id(),
                egui::Rect::from_center_size(at, display),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            let radius = if real {
                let t = (gain / (1.0 + gain)) as f32;
                1.3 + 1.8 * t
            } else {
                1.7 + 1.3 * gain as f32
            };
            let color = if real {
                vision::tone_rgb(moon.color, gain)
            } else {
                tint(moon.color, gain)
            };
            painter.circle_filled(at, radius, color);
        }
        let offset = at - center;
        let label_at = if offset.length() < 8.0 {
            at + egui::vec2(10.0, -8.0)
        } else {
            at + offset.normalized() * 12.0
        };
        painter.text(
            label_at,
            egui::Align2::LEFT_CENTER,
            moon.name,
            egui::FontId::proportional(12.0),
            label_color,
        );
    }

    fn moon_disk(
        &mut self,
        ctx: &egui::Context,
        moon: &Satellite,
        app: &Appearance,
        radius_px: i32,
        gain_q: i32,
        gain: f64,
        real: bool,
    ) -> egui::TextureHandle {
        if let Some(found) = self.moon_tex.iter().find(|tex| {
            tex.name == moon.name
                && tex.radius_px == radius_px
                && tex.gain_q == gain_q
                && tex.real == real
        }) {
            return found.texture.clone();
        }
        let image = disk::render_disk(
            &DiskInput {
                orientation: &app.orientation,
                flattening: 0.0,
                albedo: moon.color,
                texture: None,
                lon_offset_turns: 0.0,
                bump: None,
                rings: &[],
                draw_axis: false,
            },
            f64::from(radius_px),
            96,
        );
        let color = if real {
            vision::present_disk(&image, false, Some(gain), vision::Cast::Neutral, vision::Glare::None)
        } else {
            let tinted = tint_disk(&image, gain);
            egui::ColorImage::from_rgba_unmultiplied(
                [tinted.width() as usize, tinted.height() as usize],
                tinted.as_raw(),
            )
        };
        if let Some(found) = self.moon_tex.iter_mut().find(|tex| tex.name == moon.name) {
            found.texture.set(color, egui::TextureOptions::LINEAR);
            found.radius_px = radius_px;
            found.gain_q = gain_q;
            found.real = real;
            return found.texture.clone();
        }
        let texture = ctx.load_texture(
            format!("sat-{}", moon.name),
            color,
            egui::TextureOptions::LINEAR,
        );
        self.moon_tex.push(MoonTex {
            name: moon.name.to_owned(),
            radius_px,
            gain_q,
            real,
            texture: texture.clone(),
        });
        texture
    }
}

fn tint_disk(src: &RgbaImage, gain: f64) -> RgbaImage {
    let mut dst = src.clone();
    for px in dst.pixels_mut() {
        if px[3] == 0 {
            continue;
        }
        *px = Rgba([
            (f64::from(px[0]) * gain).round().clamp(0.0, 255.0) as u8,
            (f64::from(px[1]) * gain).round().clamp(0.0, 255.0) as u8,
            (f64::from(px[2]) * gain).round().clamp(0.0, 255.0) as u8,
            px[3],
        ]);
    }
    dst
}

fn tint(color: [u8; 3], gain: f64) -> egui::Color32 {
    let ch = |c: u8| (f64::from(c) * gain).round().clamp(0.0, 255.0) as u8;
    egui::Color32::from_rgb(ch(color[0]), ch(color[1]), ch(color[2]))
}

fn place(center: egui::Pos2, sign: f32, scale: f64, east: f64, north: f64) -> egui::Pos2 {
    center
        + egui::vec2(
            sign * east as f32 * scale as f32,
            sign * north as f32 * scale as f32,
        )
}

fn paint_scale(painter: &egui::Painter, sky: egui::Rect, px_per_arcsec: f64, color: egui::Color32) {
    let spans = [10.0, 20.0, 30.0, 60.0, 120.0, 300.0, 600.0];
    let max_w = f64::from(sky.width()) * 0.42;
    let mut span = 10.0;
    for candidate in spans {
        if candidate * px_per_arcsec <= max_w {
            span = candidate;
        }
    }
    let bar = (span * px_per_arcsec) as f32;
    let x1 = sky.right() - 16.0;
    let x0 = x1 - bar;
    let y = sky.bottom() - 22.0;
    let stroke = egui::Stroke::new(1.5, color);
    painter.line_segment([egui::pos2(x0, y), egui::pos2(x1, y)], stroke);
    painter.line_segment([egui::pos2(x0, y - 4.0), egui::pos2(x0, y + 4.0)], stroke);
    painter.line_segment([egui::pos2(x1, y - 4.0), egui::pos2(x1, y + 4.0)], stroke);
    painter.text(
        egui::pos2((x0 + x1) * 0.5, y - 6.0),
        egui::Align2::CENTER_BOTTOM,
        format!("{}″", fr_num(span, 0)),
        egui::FontId::proportional(12.0),
        color,
    );
}

fn fr_num(value: f64, digits: usize) -> String {
    format!("{:.*}", digits, value).replace('.', ",")
}
