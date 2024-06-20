use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use super::component::Component;

#[derive(Debug)]
pub struct Entity {
    pub name: String,
    components: HashMap<TypeId, Box<dyn Component>>,
    children: Vec<Entity>,
}

impl Entity {
    pub fn new(name: String) -> Self {
        Self {
            name,
            components: HashMap::default(),
            children: Vec::default(),
        }
    }
    pub fn add_component<T: Component>(&mut self, component: T) {
        self.components
            .insert(TypeId::of::<T>(), Box::new(component));
    }
    pub fn new_component<T: Component + Default>(&mut self) {
        let component = T::default();
        self.components
            .insert(TypeId::of::<T>(), Box::new(component));
    }
    pub fn get_component<T: Component>(&self) -> Option<&T> {
        let component: &dyn Any = self.components.get(&TypeId::of::<T>())?;
        component.downcast_ref()
    }
    pub fn get_component_mut<T: Component>(&mut self) -> Option<&mut T> {
        let component: &mut dyn Any = self.components.get_mut(&TypeId::of::<T>())?;
        component.downcast_mut()
    }
    pub fn has_component<T: Component>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }

    pub fn children(&self) -> &[Self] {
        &self.children
    }
    pub fn add_child(&mut self, child: Self) {
        self.children.push(child);
    }
}
