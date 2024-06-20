use glam::{Affine3A, Quat, Vec3};

use crate::renderer::gui::{quat_edit, vec3_edit};

use super::component::Component;

#[derive(Debug, Clone, Copy, Default)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    global_transform: Affine3A,
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            global_transform: Affine3A::from_scale_rotation_translation(scale, rotation, position),
        }
    }

    pub fn update_global_transform(&mut self, parent: Self) {
        self.global_transform = parent.global_transform * self.local_transform();
    }

    pub fn local_transform(&self) -> Affine3A {
        Affine3A::from_scale_rotation_translation(self.scale, self.rotation, self.position)
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
