use std::{any::Any, marker::PhantomData, sync::OnceLock};

use assets_manager::{source::FileSystem, AssetCache};
use slotmap::{new_key_type, SlotMap};

use super::image::Image;

new_key_type! {
    pub struct ResourceKey;
}
#[derive(Debug)]
pub struct ResourceHandle<T: 'static + Send + Sync> {
    key: ResourceKey,
    phantom: PhantomData<T>,
}

static ResourceManager: OnceLock<ResourceManager> = OnceLock::new();

impl<T: 'static + Send + Sync> ResourceHandle<T> {
    const fn new(key: ResourceKey) -> Self {
        Self {
            key,
            phantom: PhantomData,
        }
    }
}
pub struct ResourceManager {
    pub external_assets: AssetCache<FileSystem>,
    resources: SlotMap<ResourceKey, Box<dyn 'static + Send + Sync>>,
}

impl ResourceManager {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            external_assets: AssetCache::new("assets")?,
            resources: SlotMap::default(),
        })
    }
    pub fn save_resource<T: 'static + Send + Sync>(&mut self, resource: T) -> ResourceHandle<T> {
        let id = self.resources.insert(Box::new(resource));
        ResourceHandle::new(id)
    }
    pub fn load_resource<T: 'static + Send + Sync>(
        &self,
        handle: &ResourceHandle<T>,
    ) -> Option<&T> {
        let resource: &dyn Any = self.resources.get(handle.key)?;
        resource.downcast_ref()
    }
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn load_resource_mut<T: 'static + Send + Sync>(
        &mut self,
        handle: &mut ResourceHandle<T>,
    ) -> Option<&mut T> {
        let resource: &mut dyn Any = self.resources.get_mut(handle.key)?;
        resource.downcast_mut()
    }
    #[allow(clippy::needless_pass_by_value)]
    pub fn remove_resource<T: 'static + Send + Sync>(&mut self, handle: ResourceHandle<T>) {
        self.resources.remove(handle.key);
    }
}
