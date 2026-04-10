use crate::tree::{biology, TreeConfig};
use super::node::{Tree, NodeType };

pub enum TreeEvent {
    Grow(usize),
    Branch { parent: usize, node_type: NodeType, elevation: f32, azimuth: f32 },
    Deactivate(usize),
    Activate(usize),
}

pub(crate) fn tick(tree: &mut Tree, config: &TreeConfig) -> Vec<TreeEvent> {
    let mut events = Vec::new();

    biology::update_auxin(tree, config);
    biology::update_pipes(tree);

    // iterate through all nodes and generate growth events
    for (index, node) in tree.nodes.iter().enumerate() {
        events.push(TreeEvent::Grow(index));
        if node.node_type.is_active_wood() && node.length >= config.branch_threshold && node.children.is_empty() {

            let offset = if node.node_type.left_node() { -config.branch_elevation } else { config.branch_elevation };

            // Capture the randomized elevation for the continuation segment first
            let continuation_elevation = node.elevation + rand::random_range(-config.branch_random_variation..=config.branch_random_variation);
            let branch_elevation = node.elevation + offset + rand::random_range(-config.branch_random_variation..=config.branch_random_variation);
            let leaf_elevation = 2.0 * node.elevation -branch_elevation + rand::random_range(-config.branch_random_variation..=config.branch_random_variation); // opposite side of the branch

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
            // Leaf sprouts from the elbow — opposite direction of the auxiliary branch
            events.push(TreeEvent::Branch {
                parent: index,
                node_type: NodeType::Leaf { size: 12.0 },
                elevation: leaf_elevation,  // opposite side from auxiliary branch, relative to parent
                azimuth: node.azimuth,
            });
            events.push(TreeEvent::Deactivate(index));
        } else if node.node_type.is_inactive_wood() {
            if node.age >= config.min_activation_age {
                if let Some(parent_idx) = node.parent {
                    if tree.nodes[parent_idx].auxin_received <= config.auxin_threshold {
                        events.push(TreeEvent::Activate(index));
                    }
                }
            }
        } else if node.node_type.is_active_wood() && node.auxin_received > config.auxin_threshold {
            events.push(TreeEvent::Deactivate(index));
        }
    }
    apply_events(tree, &events, config);
    events
}

fn apply_events(tree: &mut Tree, events: &[TreeEvent], config: &TreeConfig) {
    for event in events {
        match event {
            TreeEvent::Grow(index) => {
                tree.nodes[*index].age += 1;
                if tree.nodes[*index].node_type.is_active_wood() && tree.nodes[*index].children.is_empty() {
                    if rand::random_range(0.0..1.0) < config.node_activity_probability {
                        tree.nodes[*index].length += config.growth_rate_length;

                        // Gravitropic correction
                        let deviation = tree.nodes[*index].elevation - 90.0;
                        if deviation.abs() > config.gravitropism_threshold {
                            let excess = deviation.abs() - config.gravitropism_threshold;
                            let max_excess = 90.0 - config.gravitropism_threshold; // how far past threshold is possible
                            let strength = excess / max_excess; // 0.0 at threshold, 1.0 at horizontal
                            let correction = config.gravitropism_rate * strength;
                            if deviation > 0.0 {
                                tree.nodes[*index].elevation -= correction;
                            } else {
                                tree.nodes[*index].elevation += correction;
                            }
                        }
                    }
                }
            }
            TreeEvent::Branch { parent, node_type, elevation, azimuth } => {
                let length = if node_type.is_leaf() { 0.0 } else { 1.0 };
                tree.add_node(*parent, node_type.clone(), *elevation, *azimuth, length);
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