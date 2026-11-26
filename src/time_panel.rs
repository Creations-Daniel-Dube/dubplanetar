//! Réglage de l'instant utilisé par la vue des planètes.

use chrono::{Datelike, Timelike};

use crate::panel::Panel;
use crate::state::{self, Observation};

pub struct DatePanel;

impl Panel for DatePanel {
    fn id(&self) -> &'static str {
        "date-time"
    }

    fn label(&self) -> &'static str {
        "Date et heure"
    }

    fn ui(&mut self, ui: &mut egui::Ui, observation: &mut Observation) {
        ui.add_space(8.0);
        ui.heading("Date et heure");
        ui.add_space(6.0);
        ui.label("L'instant choisi sert à la vue « Planètes à l'échelle ».");
        ui.add_space(18.0);

        let mut year = observation.when.year();
        let mut month = observation.when.month();
        let mut day = observation.when.day();
        let mut hour = observation.when.hour();
        let mut minute = observation.when.minute();
        let before = (year, month, day, hour, minute);

        egui::Grid::new("date-grid")
            .num_columns(2)
            .spacing(egui::vec2(18.0, 12.0))
            .show(ui, |ui| {
                ui.label("Année");
                ui.add(egui::DragValue::new(&mut year).range(state::YEAR_MIN..=state::YEAR_MAX));
                ui.end_row();

                ui.label("Mois");
                egui::ComboBox::from_id_salt("mois")
                    .selected_text(state::MONTHS[(month as usize).saturating_sub(1)])
                    .show_ui(ui, |ui| {
                        for (index, name) in state::MONTHS.iter().enumerate() {
                            ui.selectable_value(&mut month, (index as u32) + 1, *name);
                        }
                    });
                ui.end_row();

                ui.label("Jour");
                let dim = state::days_in_month(year, month);
                ui.add(egui::DragValue::new(&mut day).range(1u32..=dim));
                ui.end_row();

                ui.label("Heure");
                ui.add(egui::DragValue::new(&mut hour).range(0u32..=23));
                ui.end_row();

                ui.label("Minute");
                ui.add(egui::DragValue::new(&mut minute).range(0u32..=59));
                ui.end_row();
            });

        ui.add_space(16.0);
        let now = ui.add_sized(egui::vec2(180.0, 34.0), egui::Button::new("Maintenant"));
        if now.clicked() {
            observation.when = state::truncate_minute(chrono::Local::now());
        } else if (year, month, day, hour, minute) != before {
            observation.when = state::with_civil(observation.when, year, month, day, hour, minute);
        }

        ui.add_space(22.0);
        ui.label(
            egui::RichText::new(state::format_local(observation.when))
                .size(22.0)
                .color(egui::Color32::from_rgb(50, 230, 90)),
        );
        ui.label(format!(
            "Heure locale  ·  UTC : {}",
            state::format_utc(observation.when)
        ));
    }
}
