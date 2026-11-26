//! Mercure à Neptune, à la même échelle angulaire, à un instant donné.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use image::RgbaImage;

use crate::animation;
use crate::bodies::{self, Body};
use crate::ephemeris::{self, Appearance};
use crate::panel::Panel;
use crate::raster_pump::{self, DiskJob, RasterPump};
use crate::state::Observation;

const HEADER_H: f32 = 118.0;
const LABEL_H: f32 = 54.0;
const SKY_GREEN: egui::Color32 = egui::Color32::from_rgb(40, 220, 80);
const LABEL: egui::Color32 = egui::Color32::from_rgb(235, 235, 240);
const DIM: egui::Color32 = egui::Color32::from_rgb(180, 190, 205);

pub struct PlanetsPanel {
    cache: HashMap<Body, Cached>,
    maps: HashMap<Body, StoredMap>,
    pump: RasterPump,
    requested: Option<Requested>,
}

impl Default for PlanetsPanel {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
            maps: HashMap::new(),
            pump: RasterPump::new(),
            requested: None,
        }
    }
}

struct Requested {
    token: u64,
    keys: HashMap<Body, RasterKey>,
}

struct StoredMap {
    stamp: Option<(u64, u64)>,
    image: Option<Arc<RgbaImage>>,
}

struct Cached {
    key: RasterKey,
    north_down: bool,
    north_up: RgbaImage,
    texture: egui::TextureHandle,
}

#[derive(Clone, PartialEq, Eq)]
struct RasterKey {
    minute: i64,
    radius_px: i32,
    stamp: Option<(u64, u64)>,
}

impl Panel for PlanetsPanel {
    fn id(&self) -> &'static str {
        "planets-scale"
    }

    fn label(&self) -> &'static str {
        "Planètes à l'échelle"
    }

    fn ui(&mut self, ui: &mut egui::Ui, observation: &mut Observation) {
        let full = ui.available_rect_before_wrap();
        if full.width() < 200.0 || full.height() < 240.0 {
            return;
        }
        ui.allocate_rect(full, egui::Sense::hover());

        let content = animation::content_rect(full);
        let jd = ephemeris::julian_day_utc(observation.when.with_timezone(&chrono::Utc));
        let slots = layout_slots(content);
        let views: Vec<(Body, Appearance)> = slots
            .iter()
            .map(|slot| (slot.body, ephemeris::appearance(slot.body, jd)))
            .collect();

        let ppp = f64::from(ui.ctx().pixels_per_point());
        let mut scale = f64::MAX;
        for (slot, (body, app)) in slots.iter().zip(views.iter()) {
            let (area_w, area_h) = image_room(slot.rect);
            let half_arc = app.eq_diameter_arcsec * 0.5 * body.canvas_extent();
            if half_arc <= 0.0 {
                continue;
            }
            let fit = f64::from(area_w.min(area_h)) * 0.5 / half_arc;
            scale = scale.min(fit);
        }
        if scale == f64::MAX {
            scale = 1.0;
        }
        for (body, app) in &views {
            let canvas = app.eq_diameter_arcsec * body.canvas_extent() * scale * ppp;
            if canvas > 960.0 {
                scale *= 960.0 / canvas;
            }
        }

        let painter = ui.painter().clone();
        paint_header(&painter, full, observation, scale);
        paint_rows(&painter, content);

        let cb = egui::Rect::from_min_size(
            egui::pos2(full.left() + 16.0, full.top() + 86.0),
            egui::vec2(240.0, 22.0),
        );
        ui.put(
            cb,
            egui::Checkbox::new(&mut observation.north_down, "Nord vers le bas"),
        );

        let minute = (jd * 1440.0).round() as i64;
        let mut specs = Vec::new();
        for (slot, (body, app)) in slots.iter().zip(views.iter()) {
            let radius_px = (app.eq_diameter_arcsec * 0.5 * scale * ppp)
                .round()
                .max(1.0) as i32;
            specs.push((*body, *app, radius_px, slot));
        }
        let busy = self.sync(ui.ctx(), &specs, minute, observation.north_down);
        if busy {
            observation.waiting_for_image = true;
            ui.ctx().request_repaint_after(Duration::from_millis(40));
        }
        for (body, app, _, slot) in &specs {
            if let Some(cached) = self.cache.get(body) {
                let [w, h] = cached.texture.size();
                let ppp_f = ui.ctx().pixels_per_point();
                let display = egui::vec2(w as f32 / ppp_f, h as f32 / ppp_f);
                let (_, area_h) = image_room(slot.rect);
                let center = egui::pos2(slot.rect.center().x, slot.rect.top() + 4.0 + area_h * 0.5);
                let image_rect = egui::Rect::from_center_size(center, display);
                let response = ui.put(
                    image_rect,
                    egui::Image::from_texture(&cached.texture).fit_to_exact_size(display),
                );
                response.on_hover_text(tooltip(app));
            }
            paint_labels(&painter, slot, app);
        }
        if busy {
            raster_pump::paint_busy(ui.painter(), content);
        }
        animation::bar(ui, observation, full);
    }
}

impl PlanetsPanel {
    fn sync(
        &mut self,
        ctx: &egui::Context,
        specs: &[(Body, Appearance, i32, &Slot)],
        minute: i64,
        north_down: bool,
    ) -> bool {
        let ready = self.pump.poll();
        for done in ready {
            let Some(body) = body_from_id(done.id) else {
                continue;
            };
            let Some(key) = self
                .requested
                .as_ref()
                .filter(|requested| requested.token == done.token)
                .and_then(|requested| requested.keys.get(&body))
                .cloned()
            else {
                continue;
            };
            self.install(ctx, body, key, north_down, done.image);
        }

        let mut missing = Vec::new();
        for (body, app, radius_px, _) in specs {
            let stamp = bodies::texture_stamp(*body);
            let key = RasterKey {
                minute,
                radius_px: *radius_px,
                stamp,
            };
            if let Some(cached) = self.cache.get_mut(body) {
                if cached.key == key {
                    if cached.north_down != north_down {
                        let color = color_image(&cached.north_up, north_down);
                        cached.texture.set(color, egui::TextureOptions::LINEAR);
                        cached.north_down = north_down;
                    }
                    continue;
                }
            }
            missing.push((*body, key, *app, *radius_px));
        }
        if missing.is_empty() {
            return false;
        }
        let already = self.requested.as_ref().is_some_and(|requested| {
            missing
                .iter()
                .all(|(body, key, _, _)| requested.keys.get(body) == Some(key))
        });
        if !already {
            let token = self.pump.bump_token();
            let mut keys = HashMap::new();
            for (body, key, app, radius_px) in &missing {
                let texture = self.map_for(*body, key.stamp);
                keys.insert(*body, key.clone());
                self.pump.submit(DiskJob {
                    token,
                    id: body_id(*body),
                    orientation: app.orientation,
                    flattening: app.flattening,
                    albedo: body.albedo(),
                    texture,
                    lon_offset_turns: body.texture_lon_offset(),
                    bump: None,
                    rings: body.ring_zones(),
                    draw_axis: true,
                    radius_px: f64::from(*radius_px),
                    max_side: 1000,
                });
            }
            self.requested = Some(Requested { token, keys });
        }
        true
    }

    fn map_for(&mut self, body: Body, stamp: Option<(u64, u64)>) -> Option<Arc<RgbaImage>> {
        if let Some(stored) = self.maps.get(&body) {
            if stored.stamp == stamp {
                return stored.image.clone();
            }
        }
        let image = stamp.and_then(|_| bodies::load_texture(body)).map(Arc::new);
        self.maps.insert(
            body,
            StoredMap {
                stamp,
                image: image.clone(),
            },
        );
        image
    }

    fn install(
        &mut self,
        ctx: &egui::Context,
        body: Body,
        key: RasterKey,
        north_down: bool,
        north_up: RgbaImage,
    ) {
        let color = color_image(&north_up, north_down);
        if let Some(cached) = self.cache.get_mut(&body) {
            cached.texture.set(color, egui::TextureOptions::LINEAR);
            cached.key = key;
            cached.north_down = north_down;
            cached.north_up = north_up;
        } else {
            let texture = ctx.load_texture(
                format!("planet-{}", body.file_stem()),
                color,
                egui::TextureOptions::LINEAR,
            );
            self.cache.insert(
                body,
                Cached {
                    key,
                    north_down,
                    north_up,
                    texture,
                },
            );
        }
    }
}

fn body_id(body: Body) -> u64 {
    match body {
        Body::Mercury => 0,
        Body::Venus => 1,
        Body::Mars => 2,
        Body::Jupiter => 3,
        Body::Saturn => 4,
        Body::Uranus => 5,
        Body::Neptune => 6,
        Body::Moon => 7,
    }
}

fn body_from_id(id: u64) -> Option<Body> {
    Some(match id {
        0 => Body::Mercury,
        1 => Body::Venus,
        2 => Body::Mars,
        3 => Body::Jupiter,
        4 => Body::Saturn,
        5 => Body::Uranus,
        6 => Body::Neptune,
        7 => Body::Moon,
        _ => return None,
    })
}

fn color_image(north_up: &RgbaImage, north_down: bool) -> egui::ColorImage {
    let owned;
    let image = if north_down {
        owned = flip_180(north_up);
        &owned
    } else {
        north_up
    };
    egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
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

struct Slot {
    body: Body,
    rect: egui::Rect,
}

fn layout_slots(full: egui::Rect) -> Vec<Slot> {
    let top = full.top() + HEADER_H;
    let body = egui::Rect::from_min_max(
        egui::pos2(full.left() + 8.0, top),
        full.right_bottom() - egui::vec2(8.0, 8.0),
    );
    let h1 = body.height() * 0.23;
    let h2 = body.height() * 0.40;
    let h3 = body.height() - h1 - h2;
    let row1 = egui::Rect::from_min_size(body.min, egui::vec2(body.width(), h1));
    let row2 = egui::Rect::from_min_size(
        egui::pos2(body.left(), row1.bottom()),
        egui::vec2(body.width(), h2),
    );
    let row3 = egui::Rect::from_min_size(
        egui::pos2(body.left(), row2.bottom()),
        egui::vec2(body.width(), h3),
    );
    let half = row1.width() * 0.5;
    let quarter = row3.width() * 0.25;
    let mut slots = vec![
        Slot {
            body: Body::Mercury,
            rect: egui::Rect::from_min_size(row1.min, egui::vec2(half, row1.height())),
        },
        Slot {
            body: Body::Mars,
            rect: egui::Rect::from_min_size(
                egui::pos2(row1.left() + half, row1.top()),
                egui::vec2(half, row1.height()),
            ),
        },
        Slot {
            body: Body::Venus,
            rect: row2,
        },
    ];
    for (index, body) in [Body::Jupiter, Body::Saturn, Body::Uranus, Body::Neptune]
        .into_iter()
        .enumerate()
    {
        slots.push(Slot {
            body,
            rect: egui::Rect::from_min_size(
                egui::pos2(row3.left() + index as f32 * quarter, row3.top()),
                egui::vec2(quarter, row3.height()),
            ),
        });
    }
    slots
}

fn image_room(slot: egui::Rect) -> (f32, f32) {
    (
        (slot.width() - 8.0).max(1.0),
        (slot.height() - LABEL_H - 6.0).max(1.0),
    )
}

fn paint_header(painter: &egui::Painter, full: egui::Rect, observation: &Observation, scale: f64) {
    painter.text(
        egui::pos2(full.left() + 16.0, full.top() + 8.0),
        egui::Align2::LEFT_TOP,
        "Les planètes à l'échelle",
        egui::FontId::proportional(24.0),
        SKY_GREEN,
    );
    painter.text(
        egui::pos2(full.left() + 16.0, full.top() + 40.0),
        egui::Align2::LEFT_TOP,
        format!(
            "{}   ·   heure locale",
            crate::state::format_local(observation.when)
        ),
        egui::FontId::proportional(16.0),
        LABEL,
    );
    let subtitle = if observation.north_down {
        "Telles que vues dans un télescope, le Nord vers le bas."
    } else {
        "Telles que vues avec le Nord vers le haut."
    };
    painter.text(
        egui::pos2(full.left() + 16.0, full.top() + 64.0),
        egui::Align2::LEFT_TOP,
        subtitle,
        egui::FontId::proportional(14.0),
        DIM,
    );
    paint_scale(painter, full, scale);
}

fn paint_scale(painter: &egui::Painter, full: egui::Rect, points_per_arcsec: f64) {
    let max_w = (full.width() - 380.0).max(70.0);
    let mut span = 60.0_f64;
    while span * points_per_arcsec > f64::from(max_w) && span > 10.0 {
        span -= 10.0;
    }
    let bar_w = (span * points_per_arcsec) as f32;
    let x1 = full.right() - 18.0;
    let x0 = x1 - bar_w;
    let y = full.top() + 28.0;
    painter.text(
        egui::pos2(x0 - 10.0, y),
        egui::Align2::RIGHT_CENTER,
        "Échelle",
        egui::FontId::proportional(14.0),
        LABEL,
    );
    let stroke = egui::Stroke::new(1.5, LABEL);
    painter.line_segment([egui::pos2(x0, y), egui::pos2(x1, y)], stroke);
    let mut tick = 0.0;
    while tick <= span + 0.1 {
        let x = x0 + (tick / span) as f32 * bar_w;
        painter.line_segment([egui::pos2(x, y - 5.0), egui::pos2(x, y + 5.0)], stroke);
        painter.text(
            egui::pos2(x, y + 8.0),
            egui::Align2::CENTER_TOP,
            format!("{tick:.0}\""),
            egui::FontId::proportional(12.0),
            LABEL,
        );
        tick += 10.0;
    }
}

fn paint_rows(painter: &egui::Painter, full: egui::Rect) {
    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 150, 60));
    let top = full.top() + HEADER_H;
    let height = full.bottom() - 8.0 - top;
    for fraction in [0.0_f32, 0.23, 0.63] {
        let y = top + height * fraction;
        painter.line_segment(
            [
                egui::pos2(full.left() + 12.0, y),
                egui::pos2(full.right() - 12.0, y),
            ],
            stroke,
        );
    }
}

fn paint_labels(painter: &egui::Painter, slot: &Slot, app: &Appearance) {
    let x = slot.rect.center().x;
    let y = slot.rect.bottom() - LABEL_H + 2.0;
    let clip = painter.with_clip_rect(slot.rect);
    clip.text(
        egui::pos2(x, y),
        egui::Align2::CENTER_TOP,
        slot.body.name_fr(),
        egui::FontId::proportional(16.0),
        LABEL,
    );
    clip.text(
        egui::pos2(x, y + 18.0),
        egui::Align2::CENTER_TOP,
        format!("{}\"", fr_num(app.eq_diameter_arcsec, 1)),
        egui::FontId::proportional(14.0),
        LABEL,
    );
    let percent = (app.illuminated * 100.0).round() as i32;
    clip.text(
        egui::pos2(x, y + 36.0),
        egui::Align2::CENTER_TOP,
        format!("{percent} %"),
        egui::FontId::proportional(13.0),
        DIM,
    );
}

fn tooltip(app: &Appearance) -> String {
    format!(
        "Distance : {} UA\nDiamètre : {}\"\nÉclairée : {} %\nPhase : {}°\nLatitude sub-terrestre : {}°\nAxe du pôle : {}°\nLimbe éclairé : {}°",
        fr_num(app.distance_au, 3),
        fr_num(app.eq_diameter_arcsec, 1),
        (app.illuminated * 100.0).round() as i32,
        fr_num(app.phase_angle_rad.to_degrees(), 1),
        fr_num(app.sub_earth_lat_rad.to_degrees(), 1),
        fr_num(wrap_degrees(app.pole_pa_rad.to_degrees()), 1),
        fr_num(wrap_degrees(app.bright_limb_pa_rad.to_degrees()), 1),
    )
}

fn wrap_degrees(mut degrees: f64) -> f64 {
    degrees %= 360.0;
    if degrees < 0.0 {
        degrees += 360.0;
    }
    degrees
}

fn fr_num(value: f64, digits: usize) -> String {
    format!("{:.*}", digits, value).replace('.', ",")
}
