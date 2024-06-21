use std::{any::TypeId, collections::HashMap};

use slotmap::{new_key_type, DenseSlotMap, SlotMap};

use super::{component::Component, transform::Transform};

new_key_type! {
    pub struct ComponentKey;
    pub struct EntityKey;
}

#[derive(Default)]
pub struct EntityHierarchy {
    pub entities: DenseSlotMap<EntityKey, Entity>,
}

impl EntityHierarchy {
    pub fn new() -> Self {
        Self {
            entities: DenseSlotMap::default(),
        }
    }
    pub fn add_entity(&mut self, name: String) -> EntityKey {
        let entity = Entity::new(name);
        self.entities.insert(entity)
    }
    pub fn remove_entity(&mut self, entity: EntityKey) -> Option<Entity> {
        self.entities.remove(entity)
    }

    pub fn update_transform_hierarchies(&mut self) {
        for (_, entity) in &mut self.entities {
            entity.update_transform_hierarchies();
        }
    }
}
// Only one component per entity
#[derive(Default)]
pub struct Entity {
    pub name: String,
    components: SlotMap<ComponentKey, Box<dyn Component>>,
    type_map: HashMap<TypeId, ComponentKey>,
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
    pub fn remove_component<T: Component>(&mut self) -> Option<Box<T>> {
        let id = self.type_map.get(&TypeId::of::<T>())?;
        let pop = self.components.remove(*id)?;
        pop.downcast().ok()
    }
    pub fn has_component<T: Component>(&self) -> bool {
        self.type_map.contains_key(&TypeId::of::<T>())
    }

    pub fn update_transform_hierarchies(&mut self) {
        // let Some(transform) = self.get_component::<Transform>() else {
        //     return;
        // };
        // for child_transform in self
        //     .children
        //     .iter_mut()
        //     .filter_map(|c| c.get_component_mut::<Transform>())
        // {
        //     child_transform.update_global_transform(*transform);
        // }
    }

    pub fn add_child(&mut self, child: Self) {
        self.children.push(child);
    }
}
