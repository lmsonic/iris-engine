use glam::{Affine3A, Quat, Vec3};

use crate::renderer::gui::{quat_edit, vec3_edit};

use super::{component::Component, entity::ComponentId};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub parent: Option<ComponentId>,
}
// TODO: transform hierarchies

impl Transform {
    pub const fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            parent: None,
        }
    }

    pub fn local_transform(&self) -> Affine3A {
        Affine3A::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn set_parent(&mut self, parent: ComponentId) {
        self.parent = Some(parent);
    }
}

impl Component for Transform {
    fn gui(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Transform", |ui| {
            vec3_edit(ui, &mut self.position, "Translation");
            quat_edit(ui, &mut self.rotation, glam::EulerRot::XYZ);
            vec3_edit(ui, &mut self.scale, "Scale");
        });
    }
}
