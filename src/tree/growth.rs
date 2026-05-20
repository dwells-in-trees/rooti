use crate::tree::{biology, TreeConfig, NULL_IDX};
use super::node::{Tree, NodeType };
use crate::tree::rng::{grows, noise_f32, PURPOSE_CONTINUATION_NOISE, PURPOSE_BRANCH_NOISE, PURPOSE_LEAF_NOISE, PURPOSE_BRANCH_THRESHOLD};

pub enum TreeEvent {
    Grow(usize),
    Branch { parent: usize, node_type: NodeType, elevation: f32, azimuth: f32 },
    Deactivate(usize),
    Activate(usize),
}

pub(crate) fn tick(tree: &mut Tree, config: &TreeConfig) -> Vec<TreeEvent> {
    let mut events = Vec::new();

    // increment tree tick-count
    tree.ticks += 1;

    // before computing growth events, compute auxin and pipe size across tree
    biology::update_auxin(tree, config);
    biology::update_pipes(tree);

    // grab seed and ticks before entering loop
    let seed = tree.seed;
    let ticks = tree.ticks;

    // iterate through all nodes and generate growth events
    for (index, node) in tree.nodes.iter().enumerate() {
        events.push(TreeEvent::Grow(index));

        let threshold_offset = noise_f32(seed, 0, index as u32, PURPOSE_BRANCH_THRESHOLD, config.branch_threshold_variation);
        let effective_threshold = config.branch_threshold + threshold_offset;

        if node.node_type.is_active_wood() && node.length >= effective_threshold && node.first_child == NULL_IDX {

            // Determine appropriate direction for new branch
            let offset = if node.node_type.left_node() { -config.branch_elevation } else { config.branch_elevation };

            // Compute elevation angles for continuation segments, branches, and leaves based on seed and tick count
            let continuation_elevation = node.elevation
                + noise_f32(seed, ticks, index as u32, PURPOSE_CONTINUATION_NOISE, config.branch_random_variation);
            let branch_elevation = node.elevation + offset
                + noise_f32(seed, ticks, index as u32, PURPOSE_BRANCH_NOISE, config.branch_random_variation);
            let leaf_elevation = 2.0 * node.elevation - branch_elevation
                + noise_f32(seed, ticks, index as u32, PURPOSE_LEAF_NOISE, config.branch_random_variation); // opposite side of the branch

            events.push(TreeEvent::Branch {
                parent: index,
                elevation: continuation_elevation,
                azimuth: node.azimuth,
                node_type: NodeType::Wood { is_active: true, left_node: !node.node_type.left_node() },
            });
            events.push(TreeEvent::Branch {
                parent: index,
                elevation: branch_elevation,
                azimuth: node.azimuth,
                node_type: NodeType::Wood { is_active: false, left_node: node.node_type.left_node() },
            });
            // ------------- LEAF NODES -------------- 
            events.push(TreeEvent::Branch {
                parent: index,
                node_type: NodeType::Leaf { size: tree.species_data.leaf_size_min },
                elevation: leaf_elevation,  // opposite side from auxiliary branch, relative to parent
                azimuth: node.azimuth,
            });
            events.push(TreeEvent::Deactivate(index));
        } else if node.node_type.is_inactive_wood() {
            if node.age >= config.min_activation_age {
                let parent_idx = node.parent;
                if parent_idx != NULL_IDX && tree.nodes[parent_idx as usize].auxin_received <= config.auxin_threshold {
                    events.push(TreeEvent::Activate(index));
                }
            }
        } else if node.node_type.is_active_wood() && node.auxin_received > config.auxin_threshold {
            events.push(TreeEvent::Deactivate(index));
        } else if node.node_type.is_leaf() {
            // todo Implement leaf pruning if fully grown and above a certain age
        }
    }
    #[cfg(feature = "diagnostics")]
    crate::diagnostics::print_status(tree.nodes.len());
    
    apply_events(tree, &events, config);
    events
}

fn apply_events(tree: &mut Tree, events: &[TreeEvent], config: &TreeConfig) {


    let seed = tree.seed;
    let ticks = tree.ticks;

    for event in events {
        match event {
            TreeEvent::Grow(index) => {
                tree.nodes[*index].age += 1;
                if tree.nodes[*index].node_type.is_active_wood() &&    tree.nodes[*index].first_child == NULL_IDX {
                    if grows(seed, ticks, *index as u32, config.node_activity_probability) {
                        tree.nodes[*index].length += config.growth_rate_length;

                        // Gravitropic correction
                        let deviation = tree.nodes[*index].elevation - 90.0;
                        if deviation.abs() > config.gravitropism_threshold {
                            let excess = deviation.abs() - config.gravitropism_threshold;
                            let max_excess = 90.0 - config.gravitropism_threshold; // The maximum possible excess beyond the threshold
                            let strength = excess / max_excess; // 0.0 at threshold, 1.0 at horizontal
                            let correction = config.gravitropism_rate * strength;
                            if deviation > 0.0 {
                                tree.nodes[*index].elevation -= correction;
                            } else {
                                tree.nodes[*index].elevation += correction;
                            }
                        }
                    }
                } else if tree.nodes[*index].node_type.is_leaf() && tree.nodes[*index].node_type.get_leaf_size() <= tree.species_data.leaf_size_max {
                    let new_leaf_size: f32 = tree.nodes[*index].node_type.get_leaf_size() + 0.01;
                    tree.nodes[*index].node_type.set_leaf_size(new_leaf_size);
                }
            }
            TreeEvent::Branch { parent, node_type, elevation, azimuth } => {
                let length = if node_type.is_leaf() { 0.0 } else { 1.0 };
                tree.add_node(*parent as u32, node_type.clone(), *elevation, *azimuth, length);
            }
            TreeEvent::Activate(index) => {
                if tree.nodes[*index].node_type.is_inactive_wood() {
                    tree.nodes[*index].node_type.set_active(true);
                }
            }
            TreeEvent::Deactivate(index) => {
                if tree.nodes[*index].node_type.is_active_wood() {
                    tree.nodes[*index].node_type.set_active(false);
                }
            }
        }
    }
}