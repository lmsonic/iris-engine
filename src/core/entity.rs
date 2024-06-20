use std::{any::TypeId, collections::HashMap};

use slotmap::{new_key_type, SlotMap};

use super::component::Component;

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
        let id = self.components.insert(Box::new(component));
        self.type_map.insert(TypeId::of::<T>(), id);
    }

    pub fn new_component<T: Component + Default>(&mut self) {
        let component = T::default();
        self.add_component(component);
    }
    pub fn get_component<T: Component>(&self) -> Option<&T> {
        let id = self.type_map.get(&TypeId::of::<T>())?;
        let component = self.components.get(*id)?;
        component.downcast_ref()
    }
    pub fn get_component_mut<T: Component>(&mut self) -> Option<&mut T> {
        let id = self.type_map.get(&TypeId::of::<T>())?;
        let component = self.components.get_mut(*id)?;
        component.downcast_mut()
    }
    pub fn has_component<T: Component>(&self) -> bool {
        self.type_map.contains_key(&TypeId::of::<T>())
    }

    pub fn add_child(&mut self, child: Self) {
        self.children.push(child);
    }
}
