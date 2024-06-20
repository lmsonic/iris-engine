use std::{any::Any, fmt::Debug};

use egui::Ui;

pub trait Component: Any + Debug {
    fn gui(&mut self, _ui: &mut Ui) {}
}
