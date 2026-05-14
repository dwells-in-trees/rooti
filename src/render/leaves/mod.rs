mod ginko;

use crate::render::geometry::Leaf;
use crate::tree::node::LeafShape;

pub fn leaf_to_points(x: f32, y: f32, angle: f32, shape: &LeafShape, size: f32) -> Leaf {
    match shape {
        LeafShape::Ginkgo => ginko::build(x, y, angle, size),
    }
}