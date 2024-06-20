use std::rc::Rc;

use glam::{Affine3A, Quat, Vec3};

use crate::renderer::gui::{quat_edit, vec3_edit};

use super::component::Component;

#[derive(Debug)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub parent: Option<Rc<Transform>>,
}

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

    pub fn global_transform(&self) -> Affine3A {
        self.parent_global_transform() * self.local_transform()
    }

    fn parent_global_transform(&self) -> Affine3A {
        self.parent
            .as_ref()
            .map_or(Affine3A::IDENTITY, |parent| parent.global_transform())
    }

    pub fn set_global_transform(&mut self, global_transform: Affine3A) {
        let world_to_parent = self.parent_global_transform().inverse();
        let global_transform = world_to_parent * global_transform;
        let (scale, rotation, position) = global_transform.to_scale_rotation_translation();
        self.scale = scale;
        self.rotation = rotation;
        self.position = position;
    }

    pub fn set_global_position(&mut self, position: Vec3) {
        let mut global_transform = self.global_transform();
        global_transform.translation = position.into();
        self.set_global_transform(global_transform);
    }
    pub fn set_global_rotation(&mut self, rotation: Quat) {
        let (scale, _, translation) = self.global_transform().to_scale_rotation_translation();

        self.set_global_transform(Affine3A::from_scale_rotation_translation(
            scale,
            rotation,
            translation,
        ));
    }
    pub fn set_global_scale(&mut self, scale: Vec3) {
        let (_, rotation, translation) = self.global_transform().to_scale_rotation_translation();

        self.set_global_transform(Affine3A::from_scale_rotation_translation(
            scale,
            rotation,
            translation,
        ));
    }

    pub fn set_parent(&mut self, parent: &Rc<Self>) {
        let global_transform = self.global_transform();
        self.parent = Some(parent.clone());
        // Maintain previous global transform
        self.set_global_transform(global_transform);
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
