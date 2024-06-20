use std::{any::Any, marker::PhantomData};

use assets_manager::{source::FileSystem, AssetCache};
use slotmap::{new_key_type, SlotMap};

new_key_type! {
    pub struct ResourceKey;
}

pub trait Resource: 'static + Send + Sync {}
#[derive(Debug)]
pub struct ResourceHandle<T: Resource + ?Sized> {
    key: ResourceKey,
    phantom: PhantomData<T>,
}

impl<T: Resource> ResourceHandle<T> {
    const fn new(key: ResourceKey) -> Self {
        Self {
            key,
            phantom: PhantomData,
        }
    }
}
pub struct ResourceManager {
    pub external_assets: AssetCache<FileSystem>,
    resources: SlotMap<ResourceKey, Box<dyn Resource>>,
}

impl ResourceManager {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            external_assets: AssetCache::new("assets")?,
            resources: SlotMap::default(),
        })
    }
    pub fn save_resource<T: Resource>(&mut self, resource: T) -> ResourceHandle<T> {
        let id = self.resources.insert(Box::new(resource));
        ResourceHandle::new(id)
    }
    pub fn load_resource<T: Resource>(&self, handle: &ResourceHandle<T>) -> Option<&T> {
        let resource: &dyn Any = self.resources.get(handle.key)?;
        resource.downcast_ref()
    }
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn load_resource_mut<T: Resource>(
        &mut self,
        handle: &mut ResourceHandle<T>,
    ) -> Option<&mut T> {
        let resource: &mut dyn Any = self.resources.get_mut(handle.key)?;
        resource.downcast_mut()
    }
    #[allow(clippy::needless_pass_by_value)]
    pub fn remove_resource<T: Resource>(
        &mut self,
        handle: ResourceHandle<T>,
    ) -> Option<Box<dyn Resource>> {
        self.resources.remove(handle.key)
    }
}
