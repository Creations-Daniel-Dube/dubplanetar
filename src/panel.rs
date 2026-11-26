//! Un panneau du menu. Un onglet de plus : un type, et une ligne dans le registre.

use crate::state::Observation;

pub trait Panel {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn ui(&mut self, ui: &mut egui::Ui, observation: &mut Observation);
}
