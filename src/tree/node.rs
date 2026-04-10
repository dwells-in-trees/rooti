pub struct Tree {
    pub(crate) nodes: Vec<TreeNode>,
}

pub struct TreeNode {
    pub(crate) parent: Option<usize>,
    pub(crate) children: Vec<usize>,
    pub(crate) length: f32,
    pub(crate) thickness: f32,
    pub(crate) elevation: f32,
    pub(crate) azimuth: f32,
    pub(crate) age: u32,
    pub(crate) auxin_received: f32,
    pub(crate) node_type: NodeType,
}

#[derive(Clone, Copy)]
pub enum NodeType {
    Wood {
        is_active: bool,
        left_node: bool,
    },
    Leaf {
        size: f32,
    },
}

impl NodeType {
    pub(crate) fn is_active_wood(&self) -> bool {
        matches!(self, NodeType::Wood { is_active: true, .. })
    }

    pub(crate) fn is_inactive_wood(&self) -> bool {
        matches!(self, NodeType::Wood { is_active: false, .. })
    }

    pub(crate) fn left_node(&self) -> bool {
        matches!(self, NodeType::Wood { left_node: true, .. })
    }

    pub(crate) fn set_active(&mut self, active: bool) {
        if let NodeType::Wood { is_active, .. } = self {
            *is_active = active;
        }
    }

    pub(crate) fn is_leaf(&self) -> bool {
        matches!(self, NodeType::Leaf { .. })
    }

    pub(crate) fn is_wood(&self) -> bool {
        matches!(self, NodeType::Wood { .. })
    }
}

pub(crate) enum LeafPlacement {
    AtBranchPoints,
    //AtBranchTips,
    //AlongSegments,
}

#[derive(Clone, Copy)]
pub(crate) enum LeafShape {
    Ginko,
}

impl TreeNode {
    fn new(parent: Option<usize>, node_type: NodeType, elevation: f32, azimuth: f32, length: f32) -> Self {
        Self { parent, children: Vec::new(), length, thickness: 0.5, elevation, azimuth, age: 0, node_type, auxin_received: 0.0 }
    }
}

impl Tree {
    pub(crate) fn new() -> Self {
        Self { nodes: vec![TreeNode::new(None, NodeType::Wood { is_active: true, left_node: false }, 90.0, 200.0, 1.0)] }
    }

    pub(crate) fn add_node(&mut self, parent_index: usize, node_type: NodeType, elevation: f32, azimuth: f32, length: f32) {
        let new_node_index = self.nodes.len();
        self.nodes.push(TreeNode::new(Some(parent_index), node_type, elevation, azimuth, length));
        self.nodes[parent_index].children.push(new_node_index);
    }

}

pub struct TreeConfig {
    // Consistent simulation parameters
    pub(crate) node_activity_probability: f32,

    // Tree specific parameters
    pub(crate) branch_threshold: f32,
    pub(crate) branch_elevation: f32,
    pub(crate) branch_random_variation: f32,
    pub(crate) growth_rate_length: f32,
    pub(crate) auxin_production: f32,
    pub(crate) auxin_threshold: f32,
    pub(crate) auxin_consumption_rate: f32,
    pub(crate) min_activation_age: u32,
    pub(crate) gravitropism_threshold: f32, // degrees from 90° before correction kicks in
    pub(crate) gravitropism_rate: f32,
    pub(crate) leaf_shape: LeafShape,
    pub(crate) leaf_placement: LeafPlacement,
}