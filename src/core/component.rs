use downcast_rs::{impl_downcast, Downcast};
use egui::Ui;

pub trait Component: Downcast {
    fn gui(&mut self, _ui: &mut Ui) {}
}
impl_downcast!(Component);
