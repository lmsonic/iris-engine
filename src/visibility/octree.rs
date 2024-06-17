use glam::Vec3;
use itertools::Itertools;

use crate::renderer::model::InstancedModel;

use super::{bounding_volume::Aabb, frustum::Frustum};

#[derive(Debug, Clone, Copy)]
pub struct OctantId(pub usize);

#[derive(Debug)]
pub struct Octree<'a> {
    root: Octant<'a>,
    depth: usize,
}

impl<'a> Octree<'a> {
    pub fn new(models: &'a [InstancedModel], depth: usize) -> Self {
        let mut min = Vec3::INFINITY;
        let mut max = Vec3::NEG_INFINITY;
        for model in models {
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
        let aabb = Aabb::from_min_max(min, max);

        let models = models.iter().collect_vec();

        let root = Octant::new(models, aabb, depth);

        Self { root, depth }
    }

    pub fn visible_models(&self, frustum: Frustum) -> Vec<&InstancedModel> {
        self.root.visible_models(frustum, self.depth)
    }
}

#[derive(Debug)]
pub struct Octant<'a> {
    aabb: Aabb,
    models: Vec<&'a InstancedModel>,
    children: [Option<Box<Octant<'a>>>; 8],
}

impl<'a> Octant<'a> {
    pub fn new(mut models: Vec<&'a InstancedModel>, aabb: Aabb, depth: usize) -> Self {
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
                children[i] = Some(Box::new(Octant::new(
                    children_models,
                    child_aabb,
                    depth - 1,
                )));
            }
        }
        println!(" {} models at depth {depth} ", models.len());
        Self {
            aabb,
            models,
            children,
        }
    }
    pub fn visible_models(&self, frustum: Frustum, depth: usize) -> Vec<&InstancedModel> {
        let mut visible_models = vec![];
        if depth == 0 {
            // Leaf node
            visible_models.extend_from_slice(&self.models);
            return visible_models;
        }
        if frustum.contains_bounding_box(self.aabb) {
            for child in self.children.iter().flatten() {
                if frustum.contains_bounding_box(child.aabb) {
                    visible_models.append(&mut child.visible_models(frustum, depth - 1));
                    println!(" {} visible models at depth {depth} ", visible_models.len());
                }
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
    let octave_size = aabb.size / 2.0;
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
