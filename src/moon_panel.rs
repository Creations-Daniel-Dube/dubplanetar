//! La Lune seule, aussi grande que le cadre, à l'instant partagé.

use std::sync::Arc;
use std::time::Duration;

use image::RgbaImage;

use crate::animation;
use crate::bodies::{self, Body, HeightMap};
use crate::ephemeris::{self, Appearance};
use crate::panel::Panel;
use crate::raster_pump::{self, DiskJob, RasterPump};
use crate::state::{self, Observation};

const HEADER_H: f32 = 118.0;
const LABEL_H: f32 = 58.0;
/// Côté maximum du raster. Au-delà, le décodage d'une carte 8k reste fluide.
const MAX_SIDE: i32 = 1800;
const SKY_GREEN: egui::Color32 = egui::Color32::from_rgb(40, 220, 80);
const LABEL: egui::Color32 = egui::Color32::from_rgb(235, 235, 240);
const DIM: egui::Color32 = egui::Color32::from_rgb(180, 190, 205);
/// Diamètre angulaire qui remplit le cadre : 35′.
const FRAME_ARCSEC: f64 = 35.0 * 60.0;

pub struct MoonPanel {
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
    north_up: RgbaImage,
    texture: egui::TextureHandle,
}

#[derive(Clone, PartialEq, Eq)]
struct RasterKey {
    minute: i64,
    radius_px: i32,
    stamp: Option<(u64, u64)>,
    bump: Option<(u64, u64)>,
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

        let cb = egui::Rect::from_min_size(
            egui::pos2(full.left() + 16.0, full.top() + 86.0),
            egui::vec2(240.0, 22.0),
        );
        ui.put(
            cb,
            egui::Checkbox::new(&mut observation.north_down, "Nord vers le bas"),
        );

        let room = image_room(content);
        let ppp = f64::from(ui.ctx().pixels_per_point());
        let side_px = f64::from(room.width().min(room.height())) * ppp;
        let radius_px = moon_radius_px(side_px, app.eq_diameter_arcsec);
        let minute = (jd * 1440.0).round() as i64;
        let busy = self.sync(ui.ctx(), &app, minute, radius_px, observation.north_down);
        if busy {
            observation.waiting_for_image = true;
            ui.ctx().request_repaint_after(Duration::from_millis(40));
        }

        if let Some(cached) = self.raster.as_ref() {
            let [w, h] = cached.texture.size();
            let ppp_f = ui.ctx().pixels_per_point();
            let display = egui::vec2(w as f32 / ppp_f, h as f32 / ppp_f);
            let center = room.center();
            let image_rect = egui::Rect::from_center_size(center, display);
            let response = ui.put(
                image_rect,
                egui::Image::from_texture(&cached.texture).fit_to_exact_size(display),
            );
            response.on_hover_text(tooltip(&app));
        }
        paint_labels(&painter, content, &app);
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
    ) -> bool {
        for done in self.pump.poll() {
            let Some((token, key)) = self.requested.clone() else {
                continue;
            };
            if done.token == token {
                self.map = done.loaded_texture.clone();
                self.map_stamp = key.stamp;
                self.map_loaded = true;
                self.bump = done.loaded_bump.clone();
                self.bump_stamp = key.bump;
                self.bump_loaded = true;
                self.install(ctx, key, north_down, done.image);
            }
        }

        let stamp = bodies::texture_stamp(Body::Moon);
        let bump_stamp = bodies::bump_stamp(Body::Moon);
        let key = RasterKey {
            minute,
            radius_px,
            stamp,
            bump: bump_stamp,
        };
        if let Some(cached) = &mut self.raster {
            if cached.key == key {
                if cached.north_down != north_down {
                    let color = color_image(&cached.north_up, north_down);
                    cached.texture.set(color, egui::TextureOptions::LINEAR);
                    cached.north_down = north_down;
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
                draw_axis: true,
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
        north_up: RgbaImage,
    ) {
        let color = color_image(&north_up, north_down);
        if let Some(cached) = &mut self.raster {
            cached.texture.set(color, egui::TextureOptions::LINEAR);
            cached.key = key;
            cached.north_down = north_down;
            cached.north_up = north_up;
        } else {
            let texture = ctx.load_texture("moon-disk", color, egui::TextureOptions::LINEAR);
            self.raster = Some(Cached {
                key,
                north_down,
                north_up,
                texture,
            });
        }
    }
}

fn moon_radius_px(room_side_px: f64, diameter_arcsec: f64) -> i32 {
    let span = (diameter_arcsec / FRAME_ARCSEC).clamp(0.05, 1.0);
    (room_side_px * 0.5 / Body::Moon.canvas_extent() * span)
        .round()
        .max(1.0) as i32
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
        "Telle que vue dans un télescope, le Nord vers le bas."
    } else {
        "Telle que vue avec le Nord vers le haut."
    };
    painter.text(
        egui::pos2(full.left() + 16.0, full.top() + 64.0),
        egui::Align2::LEFT_TOP,
        subtitle,
        egui::FontId::proportional(14.0),
        DIM,
    );
}

fn paint_labels(painter: &egui::Painter, full: egui::Rect, app: &Appearance) {
    let x = full.center().x;
    let y = full.bottom() - LABEL_H;
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
}
