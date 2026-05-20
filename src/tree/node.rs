use crate::tree::NULL_IDX;
use crate::tree::rng::{hash_u64, PURPOSE_FIRST_BRANCH};
use crate::tree::species::{Species, SpeciesData};

pub struct Tree {
    pub(crate) species: Species,
    pub(crate) species_data: SpeciesData,
    pub(crate) nodes: Vec<TreeNode>,
    pub(crate) ticks: u64,
    pub(crate) seed: u64,
}

pub struct TreeNode {
    pub(crate) parent: u32,
    pub(crate) first_child: u32,
    pub(crate) next_sibling: u32,
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
    
    pub(crate) fn get_leaf_size(&self) -> f32 {
        if let NodeType::Leaf { size } = self {
            *size
        } else {
            0.0
        }
    }
    
    pub(crate) fn set_leaf_size(&mut self, new_size: f32) {
        if let NodeType::Leaf { size, .. } = self {
            *size = new_size;
        }
    }

    pub(crate) fn is_wood(&self) -> bool {
        matches!(self, NodeType::Wood { .. })
    }
}

impl TreeNode {
    fn new(parent: u32, node_type: NodeType, elevation: f32, azimuth: f32, length: f32) -> Self {
        Self { parent, first_child: NULL_IDX, next_sibling: NULL_IDX, length, thickness: 0.5, elevation, azimuth, age: 0, node_type, auxin_received: 0.0 }
    }
}

impl Tree {
    pub(crate) fn new(seed: u64, species: Species) -> Self {
        let first_branch_left = hash_u64(seed, 0, 0, PURPOSE_FIRST_BRANCH) & 1 == 0;
        let species_data = species.data();
        Self {
            species,
            species_data,
            nodes: vec![TreeNode::new(
                NULL_IDX,
                NodeType::Wood { is_active: true, left_node: first_branch_left },
                90.0, 200.0, 1.0)],
            ticks: 0,
            // Initialize random seed
            seed,
        }
    }

    pub(crate) fn add_node(&mut self, parent_index: u32, node_type: NodeType, elevation: f32, azimuth: f32, length: f32) {
        let new_node_index = self.nodes.len() as u32;
        let existing_first_child = self.nodes[parent_index as usize].first_child;
        self.nodes.push(TreeNode::new(parent_index, node_type, elevation, azimuth, length));
        self.nodes[parent_index as usize].first_child = new_node_index;
        self.nodes[new_node_index as usize].next_sibling = existing_first_child;
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
    pub(crate) branch_threshold_variation: f32,
}