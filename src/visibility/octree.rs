use glam::Vec3;

use crate::renderer::model::Model;

use super::{bounding_volume::Aabb, frustum::Frustum};

#[derive(Debug, Clone, Copy)]
pub struct OctantId(pub usize);

#[derive(Debug, Clone)]
pub struct Octree<'a> {
    root: Octant<'a>,
    depth: usize,
}

impl<'a> Octree<'a> {
    pub fn new(models: Vec<&'a Model>, depth: usize) -> Self {
        let mut min = Vec3::INFINITY;
        let mut max = Vec3::NEG_INFINITY;
        for model in &models {
            let obb = model.bounding_box();
            let box_min = obb.min();
            let box_max = obb.max();
            min.x = f32::min(min.x, box_min.x);
            min.y = f32::min(min.y, box_min.y);
            min.z = f32::min(min.z, box_min.z);
            max.x = f32::max(max.x, box_max.x);
            max.y = f32::max(max.y, box_max.y);
            max.z = f32::max(max.z, box_max.z);
        }
        let aabb = Aabb::new(min, max);
        let root = Octant::new(models, aabb, depth);
        Self { root, depth }
    }

    pub fn visible_models(&self, frustum: Frustum) -> Vec<&Model> {
        self.root.visible_models(frustum, self.depth)
    }
}

#[derive(Debug, Clone)]
pub struct Octant<'a> {
    aabb: Aabb,
    models: Vec<&'a Model>,
    children: [Option<Box<Octant<'a>>>; 8],
}

impl<'a> Octant<'a> {
    pub fn new(models: Vec<&'a Model>, aabb: Aabb, depth: usize) -> Self {
        let depth = depth - 1;
        let children_aabb = split_aabb_in_8(aabb);
        let mut children: [Option<Box<Octant>>; 8] = Default::default();
        for (i, child_aabb) in children_aabb.into_iter().enumerate() {
            let mut children_models = vec![];

            for model in &models {
                let obb = model.bounding_box();
                if child_aabb.contains_obb(obb) {
                    children_models.push(*model);
                }
            }
            if children_models.is_empty() {
                children[i] = None;
            } else if depth > 0 {
                children[i] = Some(Box::new(Octant::new(children_models, child_aabb, depth)));
            }
        }

        Self {
            aabb,
            models,
            children,
        }
    }
    pub fn visible_models(&self, frustum: Frustum, depth: usize) -> Vec<&Model> {
        let mut visible_models = vec![];
        if depth == 0 {
            // Leaf node
            visible_models.copy_from_slice(&self.models);
        }
        let non_empty_children: Vec<_> = self.children.iter().flatten().collect();
        if non_empty_children.is_empty() {
            // Also leaf node
            visible_models.copy_from_slice(&self.models);
        }
        for child in non_empty_children {
            if frustum.contains_bounding_box(child.aabb) {
                visible_models.append(&mut child.visible_models(frustum, depth - 1));
            }
        }

        visible_models
    }
}
fn split_aabb_in_8(aabb: Aabb) -> [Aabb; 8] {
    let center = aabb.center;
    let half_size = 0.5 * aabb.size;
    // calculate centers
    let v1 = center + half_size; //+++
    let v2 = center + Vec3::new(half_size.x, half_size.y, -half_size.z); //++-
    let v3 = center + Vec3::new(half_size.x, -half_size.y, half_size.z); //+-+
    let v4 = center + Vec3::new(half_size.x, -half_size.y, -half_size.z); //+--
    let v5 = center + Vec3::new(-half_size.x, half_size.y, half_size.z); //-++
    let v6 = center + Vec3::new(-half_size.x, half_size.y, -half_size.z); //-+-
    let v7 = center + Vec3::new(-half_size.x, -half_size.y, half_size.z); //--+
    let v8 = center + Vec3::new(-half_size.x, -half_size.y, -half_size.z); //---
    let octave_size = aabb.size / 8.0;
    [
        Aabb::new(v1, octave_size),
        Aabb::new(v2, octave_size),
        Aabb::new(v3, octave_size),
        Aabb::new(v4, octave_size),
        Aabb::new(v5, octave_size),
        Aabb::new(v6, octave_size),
        Aabb::new(v7, octave_size),
        Aabb::new(v8, octave_size),
    ]
}
