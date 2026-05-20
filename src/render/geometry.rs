use crate::render::{ leaves, RenderConfig };
use crate::tree::node::{ Tree, NodeType };
use crate::tree::{ TreeConfig, NULL_IDX };

pub(crate) struct Segment {
    pub(crate) x1: f32, pub(crate) y1: f32,
    pub(crate) x2: f32, pub(crate) y2: f32,
    pub(crate) x3: f32, pub(crate) y3: f32,
    pub(crate) x4: f32, pub(crate) y4: f32,
    pub(crate) radius: f32,
    pub(crate) sweep: bool,
}

pub(crate) struct Leaf {
    // Petiole start
    pub(crate) px: f32, pub(crate) py: f32,
    // Base (petiole end / fan origin)
    pub(crate) bx: f32, pub(crate) by: f32,
    // Stem to bottom left arc endpoint
    pub(crate) blx: f32, pub(crate) bly: f32,
    pub(crate) bl_radius: f32,
    // Bottom left corner
    pub(crate) lcx: f32, pub(crate) lcy: f32,
    // Left corner arc endpoint
    pub(crate) lax: f32, pub(crate) lay: f32,
    pub(crate) la_radius: f32,
    // Divot left cubic endpoint + controls
    pub(crate) dlx: f32, pub(crate) dly: f32,
    pub(crate) dl_c1x: f32, pub(crate) dl_c1y: f32,
    pub(crate) dl_c2x: f32, pub(crate) dl_c2y: f32,
    // Divot right cubic endpoint + controls
    pub(crate) drx: f32, pub(crate) dry: f32,
    pub(crate) dr_c1x: f32, pub(crate) dr_c1y: f32,
    pub(crate) dr_c2x: f32, pub(crate) dr_c2y: f32,
    // Right corner arc endpoint
    pub(crate) rax: f32, pub(crate) ray: f32,
    pub(crate) ra_radius: f32,
    // Bottom right corner
    pub(crate) rcx: f32, pub(crate) rcy: f32,
    // Stem to base arc endpoint
    pub(crate) srx: f32, pub(crate) sry: f32,
    pub(crate) sr_radius: f32,
    // Arc sweeps
    pub(crate) stem_sweep: bool,
    pub(crate) corner_sweep: bool,
}

pub(crate) fn nodes_to_renderables(tree: &Tree, node_index: usize, tree_config: &TreeConfig, render_config: &RenderConfig, pos: (f32, f32), segments: &mut Vec<Segment>, leaves: &mut Vec<Leaf>) {
    let node = &tree.nodes[node_index];
    let first_child = node.first_child;
    let elevation_rad = node.elevation.to_radians();
    let end = (pos.0 + (node.length * elevation_rad.cos()), pos.1 - (node.length * elevation_rad.sin()));
    if node.node_type.is_wood() {
        let thickness = node.thickness * render_config.branch_thickness_factor; // scale thickness for better visibility

        // calculate end position of the segment based on length and elevation
        let dx = end.0 - pos.0;
        let dy = end.1 - pos.1;
        let len = (dx*dx + dy*dy).sqrt();
        // perpendicular unit vector
        let px = -dy / len * (thickness / 2.0);
        let py = dx / len * (thickness / 2.0);

        // compute corners of the rectangle representing the branch segment
        let x1 = pos.0 + px;
        let y1 = pos.1 + py;
        let x2 = pos.0 - px;
        let y2 = pos.1 - py;
        let x3 = end.0 - px;
        let y3 = end.1 - py;

        let cross = (x2 - x1) * (y3 - y2) - (y2 - y1) * (x3 - x2);
        segments.push(Segment {
            x1, y1, x2, y2, x3, y3,
            x4: end.0 + px, y4: end.1 + py,
            radius: thickness / 2.0,
            sweep: cross >= 0.0,
        });

    } else if node.node_type.is_leaf() {
        let parent = &tree.nodes[node.parent as usize];
        let visual_half = parent.thickness * render_config.branch_thickness_factor / 2.0;
        let parent_rad = parent.elevation.to_radians();

        let (start_x, start_y) = if let NodeType::Wood { left_node: true, .. } = parent.node_type {
            // Bottom-left corner
            (pos.0 - visual_half * parent_rad.sin(),
             pos.1 - visual_half * parent_rad.cos())
        } else {
            // Bottom-right corner
            (pos.0 + visual_half * parent_rad.sin(),
             pos.1 + visual_half * parent_rad.cos())
        };
        
        leaves.push(leaves::leaf_to_points(start_x, start_y, node.elevation, &tree.species_data.leaf_shape, node.node_type.get_leaf_size()));
    }
    
    let mut cursor = first_child;
    while cursor != NULL_IDX {
        let next = tree.nodes[cursor as usize].next_sibling;
        nodes_to_renderables(tree, cursor as usize, tree_config, render_config, end, segments, leaves);
        cursor = next;
    }
}