use slotmap::{DefaultKey, SlotMap};

use crate::renderer::{material::Material, mesh::Mesh};

#[derive(Debug)]
pub struct ResourceManager {
    pub materials: SlotMap<DefaultKey, Box<dyn for<'a> Material<'a>>>,
    pub meshes: SlotMap<DefaultKey, Box<Mesh>>,
}
