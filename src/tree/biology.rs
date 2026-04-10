use crate::tree::{Tree, TreeConfig};


pub(crate) fn update_auxin(tree: &mut Tree, config: &TreeConfig) {
    let mut auxin_values = vec![0.0f32; tree.nodes.len()];
    compute_auxin(tree,0, &mut auxin_values, config);
    for (index, value) in auxin_values.iter().enumerate() {
        tree.nodes[index].auxin_received = *value;
    }
}

fn compute_auxin(tree: &Tree, node_index: usize, values: &mut Vec<f32>, config: &TreeConfig) -> f32 {
    let node = &tree.nodes[node_index];
    let children = node.children.clone();

    let mut received = 0.0;
    for child in children {
        received += compute_auxin(tree, child, values, config);
    }

    let produced = if node.node_type.is_active_wood() { config.auxin_production } else { 0.0 };
    let consumed = node.length * node.thickness * config.auxin_consumption_rate;
    let surplus = (produced + received - consumed).max(0.0);

    values[node_index] = received;
    surplus
}

pub(crate) fn update_pipes(tree: &mut Tree) {
    let mut pipe_values = vec![0.0f32; tree.nodes.len()];
    compute_pipes(tree,0, &mut pipe_values);
    for (index, value) in pipe_values.iter().enumerate() {
        tree.nodes[index].thickness = value.max(tree.nodes[index].thickness);
    }
}

fn compute_pipes(tree: &mut Tree, node_index: usize, values: &mut Vec<f32>) -> f32 {
    let node = &tree.nodes[node_index];
    let children = node.children.clone();

    let mut sum_pipes = 0.0;
    if children.is_empty() {
        sum_pipes = 1.0; // leaf node contributes 1 pipe
    } else {
        for child in children {
            sum_pipes += compute_pipes(tree, child, values).powf(2.0);
        }
        sum_pipes = sum_pipes.sqrt();
    }
    values[node_index] = sum_pipes;
    sum_pipes
}