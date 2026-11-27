//! La Lune seule, aussi grande que le cadre, à l'instant partagé.

use std::sync::Arc;
use std::time::Duration;

use image::RgbaImage;

use crate::animation;
use crate::bodies::{self, Body, HeightMap};
use crate::ephemeris::{self, Appearance};
use crate::panel::Panel;
use crate::raster_pump::{self, DiskJob, RasterPump};
use crate::state::{self, Observation, Viewpoint};
use crate::vision;

const HEADER_H: f32 = 118.0;
const LABEL_H: f32 = 58.0;
/// Côté maximum du raster. Au-delà, le décodage d'une carte 8k reste fluide.
const MAX_SIDE: i32 = 1800;
const SKY_GREEN: egui::Color32 = egui::Color32::from_rgb(40, 220, 80);
const LABEL: egui::Color32 = egui::Color32::from_rgb(235, 235, 240);
const DIM: egui::Color32 = egui::Color32::from_rgb(180, 190, 205);
/// Diamètre angulaire qui remplit le cadre : 35′.
const FRAME_ARCSEC: f64 = 35.0 * 60.0;
/// `1` est la taille à l'ouverture. Au-delà, le disque est déjà cadré : on grossit dedans.
const ZOOM_MIN: f64 = 1.0;
const ZOOM_MAX: f64 = 8.0;
const RADIUS_STEP: f64 = 16.0;
const RESET_ZOOM: &str = "Grossissement par défaut";

pub struct MoonPanel {
    zoom: f64,
    map_stamp: Option<(u64, u64)>,
    map_loaded: bool,
    map: Option<Arc<RgbaImage>>,
    bump_stamp: Option<(u64, u64)>,
    bump_loaded: bool,
    bump: Option<Arc<HeightMap>>,
    raster: Option<Cached>,
    pump: RasterPump,
    requested: Option<(u64, RasterKey)>,
}

impl Default for MoonPanel {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            map_stamp: None,
            map_loaded: false,
            map: None,
            bump_stamp: None,
            bump_loaded: false,
            bump: None,
            raster: None,
            pump: RasterPump::new(),
            requested: None,
        }
    }
}

struct Cached {
    key: RasterKey,
    north_down: bool,
    /// −1 : rendu détaillé. Sinon gain réel quantifié.
    grade_q: i32,
    north_up: RgbaImage,
    texture: egui::TextureHandle,
}

#[derive(Clone, PartialEq, Eq)]
struct RasterKey {
    minute: i64,
    radius_px: i32,
    stamp: Option<(u64, u64)>,
    bump: Option<(u64, u64)>,
    /// Trait bleu du pôle, absent en vision réelle.
    axis: bool,
}

impl Panel for MoonPanel {
    fn id(&self) -> &'static str {
        "moon"
    }

    fn label(&self) -> &'static str {
        "Lune"
    }

    fn ui(&mut self, ui: &mut egui::Ui, observation: &mut Observation) {
        let full = ui.available_rect_before_wrap();
        if full.width() < 200.0 || full.height() < 240.0 {
            return;
        }
        ui.allocate_rect(full, egui::Sense::hover());

        let content = animation::content_rect(full);
        let jd = ephemeris::julian_day_utc(observation.when.with_timezone(&chrono::Utc));
        let app = ephemeris::appearance(Body::Moon, jd);
        let painter = ui.painter().clone();
        paint_header(&painter, full, observation);

        let room = image_room(content);
        let hover = ui.interact(room, ui.scope_id().with("moon-zoom"), egui::Sense::hover());
        magnify(ui, hover.hovered(), &mut self.zoom, ZOOM_MAX);

        let cb = egui::Rect::from_min_size(
            egui::pos2(full.left() + 16.0, full.top() + 86.0),
            egui::vec2(240.0, 22.0),
        );
        ui.put(
            cb,
            egui::Checkbox::new(&mut observation.north_down, "Nord vers le bas"),
        );
        place_reset_button(ui, reset_zoom_rect(ui, full), &mut self.zoom);

        let real = observation.real_vision;
        let lon = vision::longitude_deg(observation.when);
        let backdrop = if real {
            vision::sky_behind(Body::Moon, Viewpoint::Earth, &app, jd, lon)
        } else {
            None
        };
        if let Some(sky) = backdrop {
            painter.rect_filled(room, 0.0, sky.color);
        }

        let ppp = f64::from(ui.ctx().pixels_per_point());
        let side_px = f64::from(room.width().min(room.height())) * ppp;
        let base_radius = moon_radius_px(side_px, app.eq_diameter_arcsec);
        let radius_px = zoomed_radius_px(side_px, app.eq_diameter_arcsec, self.zoom);
        let minute = (jd * 1440.0).round() as i64;
        let grade = if real {
            Some(vision::gain_for(Body::Moon, &app, jd))
        } else {
            None
        };
        let busy = self.sync(
            ui.ctx(),
            &app,
            minute,
            radius_px,
            observation.north_down,
            real,
            grade,
        );
        if busy {
            observation.waiting_for_image = true;
            ui.ctx().request_repaint_after(Duration::from_millis(40));
        }

        if let Some(cached) = self.raster.as_ref() {
            let [w, h] = cached.texture.size();
            let ppp_f = ui.ctx().pixels_per_point();
            let cached_radius = f64::from(cached.key.radius_px).max(1.0);
            let desired = f64::from(base_radius) * self.zoom;
            let factor = (desired / cached_radius) as f32;
            let display = egui::vec2(w as f32 / ppp_f * factor, h as f32 / ppp_f * factor);
            let image_rect = egui::Rect::from_center_size(room.center(), display);
            if let Some(gain) = grade {
                let radius = display.x * 0.5 / Body::Moon.canvas_extent() as f32;
                let clip = ui.painter().with_clip_rect(room);
                vision::paint_glare(
                    &clip,
                    ui.ctx(),
                    image_rect.center(),
                    radius,
                    gain,
                    app.illuminated,
                    vision::glare_of(Body::Moon, gain, app.illuminated),
                );
            }
            ui.painter().with_clip_rect(room).image(
                cached.texture.id(),
                image_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            if ui
                .input(|i| i.pointer.hover_pos())
                .is_some_and(|pos| image_rect.contains(pos))
            {
                hover.on_hover_text(tooltip(&app));
            }
        }
        paint_labels(
            &painter,
            content,
            &app,
            backdrop.is_some_and(|sky| sky.bright),
        );
        if busy {
            raster_pump::paint_busy(ui.painter(), content);
        }
        animation::bar(ui, observation, full);
    }
}

impl MoonPanel {
    fn sync(
        &mut self,
        ctx: &egui::Context,
        app: &Appearance,
        minute: i64,
        radius_px: i32,
        north_down: bool,
        real: bool,
        grade: Option<f64>,
    ) -> bool {
        for done in self.pump.poll() {
            let Some((token, key)) = self.requested.clone() else {
                continue;
            };
            if done.token == token && key.axis == !real {
                self.map = done.loaded_texture.clone();
                self.map_stamp = key.stamp;
                self.map_loaded = true;
                self.bump = done.loaded_bump.clone();
                self.bump_stamp = key.bump;
                self.bump_loaded = true;
                self.install(ctx, key, north_down, grade, app.illuminated, done.image);
            }
        }

        let stamp = bodies::texture_stamp(Body::Moon);
        let bump_stamp = bodies::bump_stamp(Body::Moon);
        let grade_q = grade.map(vision::quantize_gain).unwrap_or(-1);
        let key = RasterKey {
            minute,
            radius_px,
            stamp,
            bump: bump_stamp,
            axis: !real,
        };
        if let Some(cached) = &mut self.raster {
            if cached.key == key {
                if cached.north_down != north_down || cached.grade_q != grade_q {
                    let color = vision::present_disk(
                        &cached.north_up,
                        north_down,
                        grade,
                        vision::Cast::Neutral,
                        moon_glare(grade, app.illuminated),
                    );
                    cached.texture.set(color, egui::TextureOptions::LINEAR);
                    cached.north_down = north_down;
                    cached.grade_q = grade_q;
                }
                return false;
            }
        }
        let already = self
            .requested
            .as_ref()
            .is_some_and(|(_, requested)| requested == &key);
        if !already {
            let token = self.pump.bump_token();
            let texture_ready = self.map_loaded && self.map_stamp == stamp;
            let bump_ready = self.bump_loaded && self.bump_stamp == bump_stamp;
            let texture = if texture_ready {
                self.map.clone()
            } else {
                None
            };
            let bump = if bump_ready { self.bump.clone() } else { None };
            let texture_path = if texture_ready {
                None
            } else {
                bodies::texture_file(Body::Moon)
            };
            let bump_path = if bump_ready {
                None
            } else {
                bodies::bump_file(Body::Moon)
            };
            self.pump.submit(DiskJob {
                token,
                id: 0,
                orientation: app.orientation,
                flattening: app.flattening,
                albedo: Body::Moon.albedo(),
                texture,
                texture_path,
                lon_offset_turns: Body::Moon.texture_lon_offset(),
                bump,
                bump_path,
                rings: &[],
                draw_axis: !real,
                radius_px: f64::from(radius_px),
                max_side: MAX_SIDE,
            });
            self.requested = Some((token, key));
        }
        true
    }

    fn install(
        &mut self,
        ctx: &egui::Context,
        key: RasterKey,
        north_down: bool,
        grade: Option<f64>,
        illuminated: f64,
        north_up: RgbaImage,
    ) {
        let color = vision::present_disk(
            &north_up,
            north_down,
            grade,
            vision::Cast::Neutral,
            moon_glare(grade, illuminated),
        );
        let grade_q = grade.map(vision::quantize_gain).unwrap_or(-1);
        if let Some(cached) = &mut self.raster {
            cached.texture.set(color, egui::TextureOptions::LINEAR);
            cached.key = key;
            cached.north_down = north_down;
            cached.grade_q = grade_q;
            cached.north_up = north_up;
        } else {
            let texture = ctx.load_texture("moon-disk", color, egui::TextureOptions::LINEAR);
            self.raster = Some(Cached {
                key,
                north_down,
                grade_q,
                north_up,
                texture,
            });
        }
    }
}

fn moon_glare(grade: Option<f64>, illuminated: f64) -> vision::Glare {
    match grade {
        Some(gain) => vision::glare_of(Body::Moon, gain, illuminated),
        None => vision::Glare::None,
    }
}

fn moon_radius_px(room_side_px: f64, diameter_arcsec: f64) -> i32 {
    let span = (diameter_arcsec / FRAME_ARCSEC).clamp(0.05, 1.0);
    (room_side_px * 0.5 / Body::Moon.canvas_extent() * span)
        .round()
        .max(1.0) as i32
}

/// Rayon demandé au raster. À `zoom = 1` c'est le rayon d'ouverture, sans quantification,
/// pour que le premier dessin reste celui d'aujourd'hui. Au-delà, pas de 16 px, plafonné
/// au côté `MAX_SIDE`.
fn zoomed_radius_px(room_side_px: f64, diameter_arcsec: f64, zoom: f64) -> i32 {
    let base = moon_radius_px(room_side_px, diameter_arcsec);
    let zoom = zoom.clamp(ZOOM_MIN, ZOOM_MAX);
    if zoom <= 1.0 + 1e-9 {
        return base;
    }
    quantize_radius(f64::from(base) * zoom).min(max_disk_radius())
}

fn quantize_radius(radius_px: f64) -> i32 {
    ((radius_px / RADIUS_STEP).round().max(1.0) as i32) * RADIUS_STEP as i32
}

fn max_disk_radius() -> i32 {
    let mut lo = 1;
    let mut hi = MAX_SIDE;
    let mut best = 1;
    while lo <= hi {
        let mid = (lo + hi) / 2;
        if disk_fits(mid) {
            best = mid;
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    best
}

fn disk_fits(radius_px: i32) -> bool {
    let extent = Body::Moon.canvas_extent();
    let eq = f64::from(radius_px).max(0.75);
    let half = (eq * extent).ceil() as i32 + 1;
    half * 2 + 1 <= MAX_SIDE
}

fn magnify(ui: &egui::Ui, hovered: bool, zoom: &mut f64, zoom_max: f64) {
    if !hovered {
        return;
    }
    let wheel = ui.input(|i| i.zoom_delta());
    if (wheel - 1.0).abs() > 0.001 {
        *zoom = (*zoom * f64::from(wheel)).clamp(ZOOM_MIN, zoom_max);
    }
    let scroll = ui.input(|i| i.smooth_scroll_delta.y);
    if scroll.abs() > 0.0 {
        let factor = (1.0 + f64::from(scroll) * 0.001_2).clamp(0.75, 1.3);
        *zoom = (*zoom * factor).clamp(ZOOM_MIN, zoom_max);
    }
}

fn reset_zoom_rect(ui: &egui::Ui, full: egui::Rect) -> egui::Rect {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let galley = ui
        .painter()
        .layout_no_wrap(RESET_ZOOM.to_owned(), font, egui::Color32::WHITE);
    let pad = ui.spacing().button_padding;
    let size = galley.size() + pad * 2.0;
    egui::Rect::from_min_size(
        egui::pos2(full.right() - 16.0 - size.x, full.top() + 86.0),
        egui::vec2(size.x, size.y.max(22.0)),
    )
}

fn place_reset_button(ui: &mut egui::Ui, rect: egui::Rect, zoom: &mut f64) {
    let enabled = (*zoom - 1.0).abs() > 1.0e-3;
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.add_enabled_ui(enabled, |ui| {
            if ui
                .add_sized(rect.size(), egui::Button::new(RESET_ZOOM))
                .clicked()
            {
                *zoom = 1.0;
            }
        });
    });
}

fn image_room(full: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::pos2(full.left() + 12.0, full.top() + HEADER_H),
        egui::pos2(full.right() - 12.0, full.bottom() - LABEL_H - 8.0),
    )
}

fn paint_header(painter: &egui::Painter, full: egui::Rect, observation: &Observation) {
    painter.text(
        egui::pos2(full.left() + 16.0, full.top() + 12.0),
        egui::Align2::LEFT_TOP,
        "La Lune",
        egui::FontId::proportional(22.0),
        SKY_GREEN,
    );
    painter.text(
        egui::pos2(full.left() + 16.0, full.top() + 42.0),
        egui::Align2::LEFT_TOP,
        format!("{}  ·  heure locale", state::format_local(observation.when)),
        egui::FontId::proportional(16.0),
        LABEL,
    );
    let subtitle = if observation.north_down {
        "Telle que vue dans un télescope, le Nord vers le bas. Molette pour zoomer."
    } else {
        "Telle que vue avec le Nord vers le haut. Molette pour zoomer."
    };
    painter.text(
        egui::pos2(full.left() + 16.0, full.top() + 64.0),
        egui::Align2::LEFT_TOP,
        subtitle,
        egui::FontId::proportional(14.0),
        DIM,
    );
}

fn paint_labels(painter: &egui::Painter, full: egui::Rect, app: &Appearance, bright_sky: bool) {
    let x = full.center().x;
    let y = full.bottom() - LABEL_H;
    if bright_sky {
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(x - 88.0, y - 4.0),
                egui::pos2(x + 88.0, y + 56.0),
            ),
            6.0,
            egui::Color32::from_rgba_unmultiplied(4, 8, 24, 220),
        );
    }
    let arcmin = app.eq_diameter_arcsec / 60.0;
    painter.text(
        egui::pos2(x, y),
        egui::Align2::CENTER_TOP,
        "Lune",
        egui::FontId::proportional(16.0),
        LABEL,
    );
    painter.text(
        egui::pos2(x, y + 18.0),
        egui::Align2::CENTER_TOP,
        format!("{}′", fr_num(arcmin, 1)),
        egui::FontId::proportional(14.0),
        LABEL,
    );
    let percent = (app.illuminated * 100.0).round() as i32;
    painter.text(
        egui::pos2(x, y + 36.0),
        egui::Align2::CENTER_TOP,
        format!("{percent} %"),
        egui::FontId::proportional(13.0),
        DIM,
    );
}

fn tooltip(app: &Appearance) -> String {
    let km = app.distance_au * 149_597_870.7;
    format!(
        "Distance : {} km\nDiamètre : {}′\nÉclairée : {} %\nPhase : {}°\nLatitude sub-terrestre : {}°",
        fr_num(km, 0),
        fr_num(app.eq_diameter_arcsec / 60.0, 1),
        (app.illuminated * 100.0).round() as i32,
        fr_num(app.phase_angle_rad.to_degrees(), 1),
        fr_num(app.sub_observer_lat_rad.to_degrees(), 1),
    )
}

fn fr_num(value: f64, digits: usize) -> String {
    format!("{:.*}", digits, value).replace('.', ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_35_arcmin_moon_fills_the_frame() {
        let room = 1_220.0;
        let full = (room * 0.5 / Body::Moon.canvas_extent()).round() as i32;
        assert_eq!(moon_radius_px(room, FRAME_ARCSEC), full);
    }

    #[test]
    fn radius_follows_the_angular_diameter() {
        let room = 1_220.0;
        let full = moon_radius_px(room, FRAME_ARCSEC) as f64;
        let near = moon_radius_px(room, 1_980.0);
        let far = moon_radius_px(room, 1_740.0);
        assert!(near > far);
        assert_eq!(near, (full * 1_980.0 / FRAME_ARCSEC).round() as i32);
        assert_eq!(far, (full * 1_740.0 / FRAME_ARCSEC).round() as i32);
    }

    #[test]
    fn zoom_one_is_the_opening_radius_and_two_doubles_it_until_the_cap() {
        let room = 480.0;
        let base = moon_radius_px(room, FRAME_ARCSEC);
        assert_eq!(zoomed_radius_px(room, FRAME_ARCSEC, 1.0), base);
        assert_eq!(zoomed_radius_px(room, FRAME_ARCSEC, 0.25), base);
        let doubled = zoomed_radius_px(room, FRAME_ARCSEC, 2.0);
        assert!((f64::from(doubled) - f64::from(base) * 2.0).abs() <= 8.0);
        assert!(doubled <= max_disk_radius());

        let wide = 4_000.0;
        let capped = zoomed_radius_px(wide, FRAME_ARCSEC, 8.0);
        assert_eq!(capped, max_disk_radius());
        assert!(disk_fits(capped));
        assert!(!disk_fits(capped + 1));
    }
}
