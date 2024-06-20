use std::sync::OnceLock;

use assets_manager::AssetCache;
#[allow(non_upper_case_globals)]
pub static AssetManager: OnceLock<AssetCache> = OnceLock::new();
