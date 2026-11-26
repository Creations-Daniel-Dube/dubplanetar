//! Bandeau d'animation partagé par les vues Planètes et Lune.

use std::time::{Duration, Instant};

use chrono::{DateTime, Datelike, Duration as DateDuration, Local, Timelike};

use crate::state::{self, Observation, Playback, StepUnit};

/// Hauteur réservée sous le ciel pour le bandeau.
pub const RESERVE_H: f32 = 72.0;
const STEP_EVERY: Duration = Duration::from_millis(700);

const FILL: egui::Color32 = egui::Color32::from_rgb(4, 10, 28);
const EDGE: egui::Color32 = egui::Color32::from_rgb(30, 150, 60);
const TITLE: egui::Color32 = egui::Color32::from_rgb(40, 220, 80);
const LABEL: egui::Color32 = egui::Color32::from_rgb(220, 228, 236);
const PILL: egui::Color32 = egui::Color32::from_rgb(16, 28, 52);
const PILL_ON: egui::Color32 = egui::Color32::from_rgb(40, 220, 80);
const PILL_ON_TEXT: egui::Color32 = egui::Color32::from_rgb(8, 16, 28);

/// Avance la date si le pas est dû, puis redemande une image.
pub fn tick(ctx: &egui::Context, observation: &mut Observation) {
    if observation.playback == Playback::Stopped {
        return;
    }
    if observation.waiting_for_image {
        ctx.request_repaint_after(Duration::from_millis(40));
        return;
    }
    let now = Instant::now();
    if let Some(due) = observation.next_step {
        if now < due {
            ctx.request_repaint_after(due.saturating_duration_since(now));
            return;
        }
    }
    if apply_step(observation) {
        observation.next_step = Some(now + STEP_EVERY);
        ctx.request_repaint();
        ctx.request_repaint_after(STEP_EVERY);
    }
}

/// Un pas dans le sens en cours. `false` aux bornes d'année : la lecture s'arrête.
pub fn apply_step(observation: &mut Observation) -> bool {
    let backward = observation.playback == Playback::Backward;
    match step_time(
        observation.when,
        observation.step_amount,
        observation.step_unit,
        backward,
    ) {
        Some(next) => {
            observation.when = next;
            true
        }
        None => {
            observation.playback = Playback::Stopped;
            observation.next_step = None;
            false
        }
    }
}

/// Pas civil. L'heure et la minute restent. `None` si l'année sort de 1000–2500.
pub fn step_time(
    when: DateTime<Local>,
    amount: i32,
    unit: StepUnit,
    backward: bool,
) -> Option<DateTime<Local>> {
    let amount = amount.clamp(1, unit.max_amount());
    let delta = if backward { -amount } else { amount };
    let (year, month, day) = match unit {
        StepUnit::Days => {
            let date = when
                .date_naive()
                .checked_add_signed(DateDuration::days(i64::from(delta)))?;
            (date.year(), date.month(), date.day())
        }
        StepUnit::Months => {
            let (year, month) = shift_months(when.year(), when.month(), delta)?;
            (year, month, when.day())
        }
        StepUnit::Years => {
            let year = when.year().checked_add(delta)?;
            (year, when.month(), when.day())
        }
    };
    if !(state::YEAR_MIN..=state::YEAR_MAX).contains(&year) {
        return None;
    }
    Some(state::with_civil(
        when,
        year,
        month,
        day,
        when.hour(),
        when.minute(),
    ))
}

fn shift_months(year: i32, month: u32, delta: i32) -> Option<(i32, u32)> {
    let index = i32::try_from(month).ok()? - 1 + delta;
    let year = year.checked_add(index.div_euclid(12))?;
    let month = u32::try_from(index.rem_euclid(12)).ok()? + 1;
    Some((year, month))
}

pub fn content_rect(full: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        full.min,
        egui::pos2(full.right(), full.bottom() - RESERVE_H),
    )
}

/// Bandeau en bas de `full`. Le ciel utile est [`content_rect`].
pub fn bar(ui: &mut egui::Ui, observation: &mut Observation, full: egui::Rect) {
    let rect = egui::Rect::from_min_max(
        egui::pos2(full.left() + 12.0, full.bottom() - RESERVE_H + 6.0),
        egui::pos2(full.right() - 12.0, full.bottom() - 8.0),
    );
    ui.painter().rect(
        rect,
        10.0,
        FILL,
        egui::Stroke::new(1.0, EDGE),
        egui::StrokeKind::Inside,
    );

    let mut bar_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(egui::vec2(14.0, 6.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center))
            .id_salt("animation-bar"),
    );
    bar_ui.spacing_mut().item_spacing.x = 8.0;
    bar_ui.label(egui::RichText::new("Animation").size(16.0).color(TITLE));
    bar_ui.add_space(10.0);

    if pill(
        &mut bar_ui,
        "Reculer",
        observation.playback == Playback::Backward,
    )
    .clicked()
    {
        start(observation, Playback::Backward, bar_ui.ctx());
    }
    if pill(
        &mut bar_ui,
        "Lecture",
        observation.playback == Playback::Forward,
    )
    .clicked()
    {
        start(observation, Playback::Forward, bar_ui.ctx());
    }
    if pill(&mut bar_ui, "1 fois", false).clicked() {
        once(observation);
        bar_ui.ctx().request_repaint();
    }
    if pill(&mut bar_ui, "Arrêt", false).clicked() {
        observation.playback = Playback::Stopped;
        observation.next_step = None;
    }

    bar_ui.add_space(18.0);
    bar_ui.label(egui::RichText::new("Saut").color(LABEL));
    let max_amount = observation.step_unit.max_amount();
    observation.step_amount = observation.step_amount.clamp(1, max_amount);
    bar_ui.add(
        egui::DragValue::new(&mut observation.step_amount)
            .range(1..=max_amount)
            .speed(0.25),
    );
    let mut unit = observation.step_unit;
    egui::ComboBox::from_id_salt("animation-step-unit")
        .selected_text(unit.label())
        .width(108.0)
        .show_ui(&mut bar_ui, |ui| {
            ui.selectable_value(&mut unit, StepUnit::Days, StepUnit::Days.label());
            ui.selectable_value(&mut unit, StepUnit::Months, StepUnit::Months.label());
            ui.selectable_value(&mut unit, StepUnit::Years, StepUnit::Years.label());
        });
    if unit != observation.step_unit {
        observation.step_unit = unit;
        observation.step_amount = observation.step_amount.clamp(1, unit.max_amount());
    }
}

/// Un seul pas en avant, sans lancer la lecture.
fn once(observation: &mut Observation) {
    observation.playback = Playback::Stopped;
    observation.next_step = None;
    if let Some(next) = step_time(
        observation.when,
        observation.step_amount,
        observation.step_unit,
        false,
    ) {
        observation.when = next;
    }
}

fn start(observation: &mut Observation, playback: Playback, ctx: &egui::Context) {
    if observation.playback == playback {
        return;
    }
    observation.playback = playback;
    observation.next_step = Some(Instant::now());
    ctx.request_repaint();
}

fn pill(ui: &mut egui::Ui, text: &str, active: bool) -> egui::Response {
    let (fill, color) = if active {
        (PILL_ON, PILL_ON_TEXT)
    } else {
        (PILL, LABEL)
    };
    ui.add(
        egui::Button::new(egui::RichText::new(text).color(color).size(14.0))
            .fill(fill)
            .corner_radius(8.0)
            .min_size(egui::vec2(92.0, 32.0)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};

    fn at(year: i32, month: u32, day: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(year, month, day, 14, 30, 0)
            .single()
            .unwrap_or_else(|| {
                NaiveDate::from_ymd_opt(year, month, day)
                    .unwrap()
                    .and_hms_opt(14, 30, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .earliest()
                    .unwrap()
            })
    }

    #[test]
    fn five_days_keeps_the_clock() {
        let next = step_time(at(2026, 3, 10), 5, StepUnit::Days, false).unwrap();
        assert_eq!(next.year(), 2026);
        assert_eq!(next.month(), 3);
        assert_eq!(next.day(), 15);
        assert_eq!(next.hour(), 14);
        assert_eq!(next.minute(), 30);
    }

    #[test]
    fn january_31_plus_one_month_lands_on_the_last_day_of_february() {
        let next = step_time(at(2026, 1, 31), 1, StepUnit::Months, false).unwrap();
        assert_eq!((next.year(), next.month(), next.day()), (2026, 2, 28));
        assert_eq!(next.hour(), 14);
    }

    #[test]
    fn playback_stops_at_year_2500() {
        let mut observation = Observation::new();
        observation.when = at(2500, 12, 31);
        observation.playback = Playback::Forward;
        observation.step_amount = 1;
        observation.step_unit = StepUnit::Days;
        assert!(!apply_step(&mut observation));
        assert_eq!(observation.playback, Playback::Stopped);
        assert!(observation.next_step.is_none());
        assert_eq!(observation.when.year(), 2500);
        assert_eq!(observation.when.day(), 31);
    }
}
