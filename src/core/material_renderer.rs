use super::{component::Component, material::Material, resources::ResourceHandle};

pub struct MaterialRenderer {
    pub material: ResourceHandle<dyn Material>,
}
impl Component for MaterialRenderer {}
