//! Le Soleil à l'heure de l'ordinateur, disque plein, taille selon la distance.

use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use chrono::{Local, Utc};
use image::{Rgba, RgbaImage};

use crate::ephemeris::{self, SunApparent};
use crate::panel::Panel;
use crate::state::{self, Observation, Viewpoint};
use crate::sun_image::{self, Acquired};
use crate::vision;

const HEADER_H: f32 = 118.0;
const LABEL_H: f32 = 58.0;
const SKY_GREEN: egui::Color32 = egui::Color32::from_rgb(40, 220, 80);
const LABEL: egui::Color32 = egui::Color32::from_rgb(235, 235, 240);
const DIM: egui::Color32 = egui::Color32::from_rgb(180, 190, 205);
/// `1` est la taille à l'ouverture. On ne descend pas en dessous.
const ZOOM_MIN: f64 = 1.0;
/// Même gros plan que la Lune, et au moins de quoi remplir le cadre.
const ZOOM_AT_LEAST: f64 = 8.0;
const ZOOM_CAP: f64 = 32.0;
const RESET_ZOOM: &str = "Grossissement par défaut";

pub struct SoleilPanel {
    zoom: f64,
    incoming: Receiver<Acquired>,
    photo: Option<RgbaImage>,
    status: Status,
    texture: Option<egui::TextureHandle>,
    shown_north_down: bool,
}

enum Status {
    Loading,
    Live,
    Cached,
    Failed(String),
}

impl Default for SoleilPanel {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(sun_image::acquire());
        });
        Self {
            zoom: 1.0,
            incoming: rx,
            photo: None,
            status: Status::Loading,
            texture: None,
            shown_north_down: false,
        }
    }
}

impl Panel for SoleilPanel {
    fn id(&self) -> &'static str {
        "soleil"
    }

    fn label(&self) -> &'static str {
        "Soleil"
    }

    fn ui(&mut self, ui: &mut egui::Ui, observation: &mut Observation) {
        let full = ui.available_rect_before_wrap();
        if full.width() < 200.0 || full.height() < 240.0 {
            return;
        }
        ui.allocate_rect(full, egui::Sense::hover());
        self.poll();

        let now = Local::now();
        let jd = ephemeris::julian_day_utc(now.with_timezone(&Utc));
        let sun = ephemeris::sun_apparent_from(jd, observation.viewpoint);
        let painter = ui.painter().clone();
        paint_header(
            &painter,
            full,
            now,
            observation.north_down,
            observation.viewpoint,
            &self.status,
        );

        let room = image_room(full);
        let mercury = ephemeris::sun_apparent_from(jd, Viewpoint::Mercury);
        let opening = sun_diameter_points(
            room.width().min(room.height()),
            sun.eq_diameter_arcsec,
            mercury.eq_diameter_arcsec,
        );
        let zoom_max = sun_zoom_limit(room.width().min(room.height()), opening);
        self.zoom = self.zoom.clamp(ZOOM_MIN, zoom_max);
        let hover = ui.interact(room, ui.scope_id().with("sun-zoom"), egui::Sense::hover());
        magnify(ui, hover.hovered(), &mut self.zoom, zoom_max);

        let cb = egui::Rect::from_min_size(
            egui::pos2(full.left() + 16.0, full.top() + 86.0),
            egui::vec2(240.0, 22.0),
        );
        ui.put(
            cb,
            egui::Checkbox::new(&mut observation.north_down, "Nord vers le bas"),
        );
        let reset = reset_zoom_rect(ui, full);
        place_reset_button(ui, reset, &mut self.zoom);
        viewpoint_picker(
            ui,
            observation,
            full.left() + 268.0,
            full.top() + 84.0,
            reset.left() - 12.0,
        );

        self.sync_texture(ui.ctx(), observation.north_down);
        if let Some(texture) = self.texture.as_ref() {
            let side = shown_sun_diameter(
                room.width().min(room.height()),
                sun.eq_diameter_arcsec,
                mercury.eq_diameter_arcsec,
                self.zoom,
            );
            let display = egui::vec2(side, side);
            let image_rect = egui::Rect::from_center_size(room.center(), display);
            if observation.real_vision {
                let clip = ui.painter().with_clip_rect(room);
                vision::paint_halo(&clip, image_rect.center(), side * 0.5, 40.0, 1.0, true);
            }
            ui.painter().with_clip_rect(room).image(
                texture.id(),
                image_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            if ui
                .input(|i| i.pointer.hover_pos())
                .is_some_and(|pos| image_rect.contains(pos))
            {
                hover.on_hover_text(tooltip(&sun));
            }
        } else if matches!(self.status, Status::Loading) {
            paint_notice(ui.painter(), room, "Téléchargement de l'image SDO…");
        }
        paint_labels(&painter, full, &sun);

        let wait = if matches!(self.status, Status::Loading) {
            Duration::from_millis(100)
        } else {
            Duration::from_secs(1)
        };
        ui.ctx().request_repaint_after(wait);
    }
}

impl SoleilPanel {
    fn poll(&mut self) {
        let Ok(result) = self.incoming.try_recv() else {
            return;
        };
        match result {
            Acquired::Live(image) => {
                self.status = Status::Live;
                self.photo = Some(image);
            }
            Acquired::Cached(image) => {
                self.status = Status::Cached;
                self.photo = Some(image);
            }
            Acquired::Missing(err) => {
                self.status = Status::Failed(err);
                self.photo = Some(plain_disk(512));
            }
        }
        self.texture = None;
    }

    fn sync_texture(&mut self, ctx: &egui::Context, north_down: bool) {
        let Some(photo) = self.photo.as_ref() else {
            return;
        };
        if let Some(texture) = &mut self.texture {
            if self.shown_north_down != north_down {
                texture.set(color_image(photo, north_down), egui::TextureOptions::LINEAR);
                self.shown_north_down = north_down;
            }
        } else {
            self.texture = Some(ctx.load_texture(
                "sun-disk",
                color_image(photo, north_down),
                egui::TextureOptions::LINEAR,
            ));
            self.shown_north_down = north_down;
        }
    }
}

/// Le Soleil vu de Mercure remplit le cadre. Les autres disques suivent le même rapport.
fn sun_diameter_points(room_side_points: f32, diameter_arcsec: f64, mercury_arcsec: f64) -> f32 {
    let span = (diameter_arcsec / mercury_arcsec.max(1.0)).clamp(0.0, 1.0);
    room_side_points * span as f32
}

/// Diamètre affiché. `zoom = 1` est l'ouverture ; un zoom plus petit y est ramené.
fn shown_sun_diameter(room_side: f32, diameter_arcsec: f64, mercury_arcsec: f64, zoom: f64) -> f32 {
    let opening = sun_diameter_points(room_side, diameter_arcsec, mercury_arcsec);
    let limit = sun_zoom_limit(room_side, opening);
    opening * zoom.clamp(ZOOM_MIN, limit) as f32
}

fn sun_zoom_limit(room_side: f32, opening_diameter: f32) -> f64 {
    let fill = if opening_diameter > 1.0 {
        f64::from(room_side / opening_diameter)
    } else {
        ZOOM_CAP
    };
    fill.max(ZOOM_AT_LEAST).min(ZOOM_CAP)
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

fn viewpoint_picker(ui: &mut egui::Ui, observation: &mut Observation, x: f32, y: f32, right: f32) {
    let width = (right - x).max(120.0);
    let rect = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(width, 26.0));
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.horizontal(|ui| {
            ui.label("Vu depuis");
            let combo = (ui.available_width() - 4.0).clamp(72.0, 150.0);
            egui::ComboBox::from_id_salt("sun-viewpoint")
                .selected_text(observation.viewpoint.label())
                .width(combo)
                .show_ui(ui, |ui| {
                    for choice in Viewpoint::ALL {
                        ui.selectable_value(&mut observation.viewpoint, choice, choice.label());
                    }
                });
        });
    });
}

fn paint_header(
    painter: &egui::Painter,
    full: egui::Rect,
    now: chrono::DateTime<Local>,
    north_down: bool,
    viewpoint: Viewpoint,
    status: &Status,
) {
    painter.text(
        egui::pos2(full.left() + 16.0, full.top() + 12.0),
        egui::Align2::LEFT_TOP,
        "Le Soleil",
        egui::FontId::proportional(22.0),
        SKY_GREEN,
    );
    painter.text(
        egui::pos2(full.left() + 16.0, full.top() + 42.0),
        egui::Align2::LEFT_TOP,
        format!(
            "{}  ·  heure de l'ordinateur  ·  vu depuis {}",
            state::format_local(now),
            viewpoint.depuis()
        ),
        egui::FontId::proportional(16.0),
        LABEL,
    );
    let orientation = if north_down {
        "Nord vers le bas."
    } else {
        "Nord vers le haut."
    };
    let source = match status {
        Status::Loading => "Téléchargement de l'image SDO…".to_owned(),
        Status::Live => "Image SDO/HMI continuum.".to_owned(),
        Status::Cached => "Dernière image SDO enregistrée.".to_owned(),
        Status::Failed(err) => clip_line(&format!("Image SDO indisponible : {err}")),
    };
    painter.text(
        egui::pos2(full.left() + 16.0, full.top() + 64.0),
        egui::Align2::LEFT_TOP,
        format!("{orientation}  {source}  Molette pour zoomer."),
        egui::FontId::proportional(14.0),
        DIM,
    );
}

fn paint_labels(painter: &egui::Painter, full: egui::Rect, sun: &SunApparent) {
    let x = full.center().x;
    let y = full.bottom() - LABEL_H;
    let arcmin = sun.eq_diameter_arcsec / 60.0;
    painter.text(
        egui::pos2(x, y),
        egui::Align2::CENTER_TOP,
        "Soleil",
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
    painter.text(
        egui::pos2(x, y + 36.0),
        egui::Align2::CENTER_TOP,
        format!("{} UA", fr_num(sun.distance_au, 4)),
        egui::FontId::proportional(13.0),
        DIM,
    );
}

fn tooltip(sun: &SunApparent) -> String {
    format!(
        "Distance : {} UA\nDiamètre : {}′\nDisque entier, sans phase",
        fr_num(sun.distance_au, 4),
        fr_num(sun.eq_diameter_arcsec / 60.0, 1),
    )
}

fn fr_num(value: f64, digits: usize) -> String {
    format!("{:.*}", digits, value).replace('.', ",")
}

fn clip_line(text: &str) -> String {
    let count = text.chars().count();
    if count <= 96 {
        return text.to_owned();
    }
    let mut clipped: String = text.chars().take(95).collect();
    clipped.push('…');
    clipped
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

fn plain_disk(side: u32) -> RgbaImage {
    let mut image = RgbaImage::new(side, side);
    let radius = f64::from(side) / 2.0;
    let center = radius - 0.5;
    let r2 = radius * radius;
    for y in 0..side {
        for x in 0..side {
            let dx = f64::from(x) - center;
            let dy = f64::from(y) - center;
            if dx * dx + dy * dy <= r2 {
                image.put_pixel(x, y, Rgba([255, 244, 214, 255]));
            }
        }
    }
    image
}

fn paint_notice(painter: &egui::Painter, area: egui::Rect, message: &str) {
    let galley = painter.layout(
        message.to_owned(),
        egui::FontId::proportional(20.0),
        egui::Color32::from_rgb(235, 240, 245),
        800.0,
    );
    let pad = egui::vec2(18.0, 10.0);
    let rect = egui::Rect::from_center_size(area.center(), galley.size() + pad * 2.0);
    painter.rect(
        rect,
        8.0,
        egui::Color32::from_rgba_unmultiplied(4, 10, 28, 230),
        egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 150, 60)),
        egui::StrokeKind::Inside,
    );
    painter.galley(
        egui::pos2(
            rect.center().x - galley.size().x * 0.5,
            rect.center().y - galley.size().y * 0.5,
        ),
        galley,
        egui::Color32::WHITE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diameter_follows_angular_size() {
        let room = 800.0;
        let mercury = 6_000.0;
        let near = sun_diameter_points(room, 1_952.0, mercury);
        let far = sun_diameter_points(room, 1_888.0, mercury);
        assert!(near > far);
        let ratio = f64::from(near) / f64::from(far);
        assert!((ratio - 1_952.0 / 1_888.0).abs() < 1e-4);
        assert!(near < room && far < room);
    }

    #[test]
    fn mercury_fills_the_frame_and_sets_the_scale() {
        let room = 800.0;
        let mercury = 5_000.0;
        let full = sun_diameter_points(room, mercury, mercury);
        assert!((f64::from(full) - f64::from(room)).abs() < 0.1);
        let half = sun_diameter_points(room, mercury * 0.5, mercury);
        assert!((f64::from(half) - f64::from(room) * 0.5).abs() < 0.1);
    }

    #[test]
    fn default_zoom_is_the_opening_diameter() {
        let room = 800.0;
        let mercury = 5_000.0;
        let earth = 1_952.0;
        let opening = sun_diameter_points(room, earth, mercury);
        assert!((shown_sun_diameter(room, earth, mercury, 1.0) - opening).abs() < 0.1);
        assert!((shown_sun_diameter(room, earth, mercury, 0.25) - opening).abs() < 0.1);
        assert!(shown_sun_diameter(room, earth, mercury, 5.5) > opening * 2.0);
        let zoom = 1.0;
        assert!((shown_sun_diameter(room, earth, mercury, zoom) - opening).abs() < 0.1);
    }

    #[test]
    fn sun_zoom_fills_a_distant_disk_and_stops_at_32() {
        let room = 800.0;
        let mercury = 5_000.0;
        let tiny = sun_diameter_points(room, 40.0, mercury);
        assert!((sun_zoom_limit(room, tiny) - 32.0).abs() < 1e-9);
        let earth = sun_diameter_points(room, 1_952.0, mercury);
        assert!((sun_zoom_limit(room, earth) - 8.0).abs() < 1e-9);
        let full = sun_diameter_points(room, mercury, mercury);
        assert!((sun_zoom_limit(room, full) - 8.0).abs() < 1e-9);
    }
}
