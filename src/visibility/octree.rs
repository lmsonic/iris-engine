use std::collections::HashSet;

use glam::Vec3;

use crate::renderer::model::InstancedModel;

use super::{
    bounding_volume::{aabb::Aabb, obb::Obb},
    frustum::Frustum,
};

#[derive(Debug)]
pub struct Octree {
    root: Octant,
    depth: usize,
}

impl Octree {
    pub fn build(models: &[InstancedModel], depth: usize) -> Self {
        let mut root = Octant::from_models(models);
        root.build_children(models, depth);
        Self { root, depth }
    }

    pub fn visible_models(&self, frustum: Frustum) -> HashSet<usize> {
        self.root.visible_models(frustum, self.depth)
    }
}

#[derive(Debug)]
pub struct Octant {
    aabb: Aabb,
    model_ids: HashSet<usize>,
    children: Vec<Octant>,
}

impl Octant {
    fn from_models(models: &[InstancedModel]) -> Self {
        let aabb = Aabb::from_obb_array(models.iter().map(InstancedModel::bounding_box));

        let model_ids = (0..models.len()).collect();
        Self {
            aabb,
            model_ids,
            children: vec![],
        }
    }

    fn build_children(&mut self, models: &[InstancedModel], depth: usize) {
        println!("{} at depth {depth}", self.model_ids.len());
        if depth == 0 {
            return;
        }
        let half_size = self.aabb.size * 0.5;
        self.children = vec![];
        for i in 0..8 {
            let center = self.aabb.center + octant_child_center(i) * half_size;
            let child_aabb = Aabb::new(center, half_size);
            println!("{:?} {:?}", self.aabb, child_aabb);
            assert!(self.aabb.contains_aabb(child_aabb));
            let mut child_model_ids = HashSet::new();
            for id in &self.model_ids {
                let model = &models[*id];
                let obb = model.bounding_box();
                if child_aabb.contains_obb(obb) {
                    child_model_ids.insert(*id);
                }
            }
            if !child_model_ids.is_empty() {
                let mut octant = Self {
                    aabb: child_aabb,
                    model_ids: child_model_ids,
                    children: vec![],
                };
                octant.build_children(models, depth - 1);
                self.children.push(octant);
            }
        }
    }

    fn visible_models(&self, frustum: Frustum, depth: usize) -> HashSet<usize> {
        if depth == 0 || self.children.is_empty() {
            // Leaf node
            println!("{} at depth {depth}", self.model_ids.len());
            return self.model_ids.clone();
        }
        let mut visible_models = HashSet::new();
        for child in &self.children {
            if frustum.intersect_bounding_box(child.aabb) {
                visible_models.extend(child.visible_models(frustum, depth - 1));
                println!("{} at depth {depth}", visible_models.len());
            }
        }
        visible_models
    }
}

#[allow(clippy::multiple_inherent_impl)]
impl Aabb {
    fn from_obb_array(aabbs: impl Iterator<Item = Obb>) -> Self {
        let mut min = Vec3::MIN;
        let mut max = Vec3::MAX;
        for obb in aabbs {
            let box_min = obb.min();
            let box_max = obb.max();
            min.x = f32::min(min.x, box_min.x);
            min.y = f32::min(min.y, box_min.y);
            min.z = f32::min(min.z, box_min.z);
            max.x = f32::max(max.x, box_max.x);
            max.y = f32::max(max.y, box_max.y);
            max.z = f32::max(max.z, box_max.z);
        }

        Aabb::from_min_max(min, max)
    }
    fn split(&self) -> [Self; 8] {
        todo!()
    }
}

const fn octant_child_center(index: usize) -> Vec3 {
    let x = if (index & 0b001) == 0 { 1.0 } else { -1.0 };
    let y = if (index & 0b010) == 0 { 1.0 } else { -1.0 };
    let z = if (index & 0b100) == 0 { 1.0 } else { -1.0 };
    Vec3::new(x, y, z)
}
