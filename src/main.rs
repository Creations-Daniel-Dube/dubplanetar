// ***************************************************
// * Créations Daniel Dubé            Daniel Dubé    *
// * Dernières Modifications -->   2026-11-27        *
// ***************************************************

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod animation;
mod bodies;
mod disk;
mod ephemeris;
mod moon_panel;
mod panel;
mod planets_panel;
mod raster_pump;
mod satellite_window;
mod satellites;
mod soleil_panel;
mod state;
mod sun_image;
mod time_panel;
mod vision;

use egui::Color32;
use moon_panel::MoonPanel;
use panel::Panel;
use planets_panel::PlanetsPanel;
use soleil_panel::SoleilPanel;
use state::Observation;
use std::sync::Arc;
use time_panel::DatePanel;

const TITLE: &str = concat!("dubPlanetar v", env!("CARGO_PKG_VERSION"));

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1380.0, 860.0])
        .with_min_inner_size([960.0, 640.0])
        .with_title(TITLE);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../images/dubPlanetar.png"))
    {
        viewport = viewport.with_icon(Arc::new(icon));
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "dubPlanetar",
        options,
        Box::new(|cc| Ok(Box::new(DubPlanetar::new(cc)))),
    )
}

struct DubPlanetar {
    observation: Observation,
    panels: Vec<Box<dyn Panel>>,
    current: usize,
}

impl DubPlanetar {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(8, 12, 32);
        visuals.window_fill = Color32::from_rgb(0, 0, 40);
        cc.egui_ctx.set_visuals(visuals);
        Self {
            observation: Observation::new(),
            // Un onglet de plus : implémenter `Panel` et l'ajouter ici.
            panels: vec![
                Box::new(DatePanel),
                Box::new(PlanetsPanel::default()),
                Box::new(MoonPanel::default()),
                Box::new(SoleilPanel::default()),
            ],
            current: 1,
        }
    }
}

impl eframe::App for DubPlanetar {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let entries: Vec<(&'static str, &'static str)> = self
            .panels
            .iter()
            .map(|panel| (panel.id(), panel.label()))
            .collect();
        let mut current = self.current;
        egui::Panel::left("menu")
            .exact_size(248.0)
            .resizable(false)
            .show(ui, |ui| menu(ui, &entries, &mut current));
        self.current = current;

        self.observation.waiting_for_image = false;
        let sky = Color32::from_rgb(0, 0, 52);
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).fill(sky))
            .show(ui, |ui| {
                ui.add_space(6.0);
                ui.checkbox(
                    &mut self.observation.real_vision,
                    "Vision réelle des objets",
                );
                ui.add_space(2.0);
                self.panels[self.current].ui(ui, &mut self.observation);
            });
        animation::tick(ui.ctx(), &mut self.observation);
    }
}

fn menu(ui: &mut egui::Ui, entries: &[(&'static str, &'static str)], current: &mut usize) {
    ui.add_space(10.0);
    ui.heading("Menu");
    ui.add_space(10.0);
    for (index, (id, label)) in entries.iter().enumerate() {
        ui.push_id(id, |ui| {
            let width = ui.available_width();
            if ui
                .add_sized(
                    egui::vec2(width, 32.0),
                    egui::Button::selectable(*current == index, *label),
                )
                .clicked()
            {
                *current = index;
            }
        });
        ui.add_space(4.0);
    }
}
