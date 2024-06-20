use std::{
    any::{Any, TypeId},
    collections::HashMap,
    rc::Rc,
};

use slotmap::{new_key_type, SlotMap};
use tracing::warn;

use super::{component::Component, transform::Transform};

#[derive(Debug)]
pub struct EntityHierarchy {
    entities: Vec<Entity>,
}

new_key_type! {
    pub struct ComponentId;
}
#[derive(Debug, Default)]
pub struct Entity {
    pub name: String,
    components: SlotMap<ComponentId, Box<dyn Component>>,
    type_map: HashMap<TypeId, ComponentId>,
    children: Vec<Entity>,
}

impl Entity {
    pub fn new(name: String) -> Self {
        Self {
            name,
            components: SlotMap::default(),
            children: Vec::default(),
            type_map: HashMap::default(),
        }
    }
    pub fn add_component<T: Component>(&mut self, component: T) {
        self.components.insert(Box::new(component));
    }

    pub fn new_component<T: Component + Default>(&mut self) {
        let component = T::default();
        self.add_component(component);
    }
    pub fn get_component<T: Component>(&self) -> Option<&T> {
        let id = self.type_map.get(&TypeId::of::<T>())?;
        let component: &dyn Any = self.components.get(*id)?;
        component.downcast_ref()
    }
    pub fn get_component_mut<T: Component>(&mut self) -> Option<&mut T> {
        let id = self.type_map.get(&TypeId::of::<T>())?;
        let component: &mut dyn Any = self.components.get_mut(*id)?;
        component.downcast_mut()
    }
    pub fn has_component<T: Component>(&self) -> bool {
        self.type_map.contains_key(&TypeId::of::<T>())
    }

    pub fn add_child(&mut self, child: Self) {
        self.children.push(child);
    }
}
