#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod animation;
mod bodies;
mod disk;
mod ephemeris;
mod moon_panel;
mod panel;
mod planets_panel;
mod state;
mod time_panel;

use egui::Color32;
use moon_panel::MoonPanel;
use panel::Panel;
use planets_panel::PlanetsPanel;
use state::Observation;
use time_panel::DatePanel;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1380.0, 860.0])
            .with_min_inner_size([960.0, 640.0])
            .with_title("Planètes à l'échelle"),
        ..Default::default()
    };
    eframe::run_native(
        "Planètes à l'échelle",
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

        let sky = Color32::from_rgb(0, 0, 52);
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).fill(sky))
            .show(ui, |ui| {
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
