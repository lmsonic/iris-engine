use std::fmt::Debug;

use egui::Ui;

pub trait Component: 'static + Debug {
    fn gui(&mut self, _ui: &mut Ui) {}
}
