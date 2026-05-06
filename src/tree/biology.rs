use crate::tree::{Tree, TreeConfig, NULL_IDX};


pub(crate) fn update_auxin(tree: &mut Tree, config: &TreeConfig) {
    let mut auxin_values = vec![0.0f32; tree.nodes.len()];
    compute_auxin(tree,0, &mut auxin_values, config);
    for (index, value) in auxin_values.iter().enumerate() {
        tree.nodes[index].auxin_received = *value;
    }
}

fn compute_auxin(tree: &Tree, node_index: usize, values: &mut Vec<f32>, config: &TreeConfig) -> f32 {
    let node =  &tree.nodes[node_index];
    let first_child = node.first_child;

    let mut received = 0.0;
    let mut cursor = first_child;
    while cursor != NULL_IDX {
        let next = tree.nodes[cursor as usize].next_sibling;
        received += compute_auxin(tree, cursor as usize, values, config);
        cursor = next;
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
    let node =  &tree.nodes[node_index];
    let first_child = node.first_child;

    let mut sum_pipes = 0.0;
    if first_child == NULL_IDX {
        sum_pipes = 1.0; // leaf node contributes 1 pipe
    } else {
        let mut cursor = first_child;
        while cursor != NULL_IDX {
            let next = tree.nodes[cursor as usize].next_sibling;
            sum_pipes += compute_pipes(tree, cursor as usize, values).powf(2.0);
            cursor = next;
        }
        sum_pipes = sum_pipes.sqrt();
    }
    values[node_index] = sum_pipes;
    sum_pipes
}