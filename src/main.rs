use slint::*;
use i_slint_backend_winit::WinitWindowAccessor;
use display_info::DisplayInfo;
use std::{error::Error };


struct TreeConfig {
    // Consistent simulation parameters
    CLOCK_INTERVAL_MS: u64,
    NODE_ACTIVITY_PROBABILITY: f32,

    // Tree specific parameters
    BRANCH_THRESHOLD: f32,
    BRANCH_ELEVATION: f32,
    BRANCH_THICKNESS_FACTOR: f32,
    BRANCH_RANDOM_VARIATION: f32,
    GROWTH_RATE_LENGTH: f32,
    AUXIN_PRODUCTION: f32,
    AUXIN_THRESHOLD: f32,
    AUXIN_CONSUMPTION_RATE: f32,
    MIN_ACTIVATION_AGE: u32,
    GRAVITROPISM_THRESHOLD: f32, // degrees from 90° before correction kicks in
    GRAVITROPISM_RATE: f32,
}

impl TreeConfig {
    fn defualt() -> Self {
        Self {
            CLOCK_INTERVAL_MS: 5,
            NODE_ACTIVITY_PROBABILITY: 0.30,
            BRANCH_THRESHOLD: 50.0,
            BRANCH_ELEVATION: 40.0,
            BRANCH_THICKNESS_FACTOR: 1.0,
            BRANCH_RANDOM_VARIATION: 15.0,
            GROWTH_RATE_LENGTH: 1.0,
            AUXIN_PRODUCTION: 10.0,
            AUXIN_THRESHOLD: 0.1,
            AUXIN_CONSUMPTION_RATE: 0.05,
            MIN_ACTIVATION_AGE: 100,
            GRAVITROPISM_THRESHOLD: 80.0, // degrees from 90° before correction kicks in
            GRAVITROPISM_RATE: 0.1,
        }
    }
}

struct Tree {
    nodes: Vec<TreeNode>,
}

struct TreeNode {
    parent: Option<usize>,
    children: Vec<usize>,
    length: f32,
    thickness: f32,
    elevation: f32,
    azimuth: f32,
    age: u32,
    auxin_received: f32,
    node_type: NodeType,
}

#[derive(Clone, Copy)]
enum NodeType {
    Wood {
        is_active: bool,
        left_node: bool,
    },
    Leaf {
        size: f32,
    },
}

impl NodeType {
    fn is_active_wood(&self) -> bool {
        matches!(self, NodeType::Wood { is_active: true, .. })
    }

    fn is_inactive_wood(&self) -> bool {
        matches!(self, NodeType::Wood { is_active: false, .. })
    }

    fn left_node(&self) -> bool {
        matches!(self, NodeType::Wood { left_node: true, .. })
    }

    fn set_active(&mut self, active: bool) {
        if let NodeType::Wood { is_active, .. } = self {
            *is_active = active;
        }
    }

    fn is_leaf(&self) -> bool {
        matches!(self, NodeType::Leaf { .. })
    }

    fn is_wood(&self) -> bool {
        matches!(self, NodeType::Wood { .. })
    }
}

enum LeafPlacement {
    AtBranchPoints,
    AtBranchTips,
    AlongSegments,
}

impl TreeNode {
    fn new(parent: Option<usize>, node_type: NodeType, elevation: f32, azimuth: f32, length: f32) -> Self {
        Self { parent, children: Vec::new(), length, thickness: 0.5, elevation, azimuth, age: 0, node_type, auxin_received: 0.0 }
    }
}


impl Tree {
    fn new() -> Self {
        Self { nodes: vec![TreeNode::new(None, NodeType::Wood { is_active: true, left_node: false }, 90.0, 200.0, 1.0)] }
    }


    fn tick(&mut self) -> Vec<TreeEvent> {
        let mut events = Vec::new();
        
        self.update_auxin();
        self.update_pipes();

        // iterate through all nodes and generate growth events
        for (index, node) in self.nodes.iter().enumerate() {
            events.push(TreeEvent::Grow(index));
            if node.node_type.is_active_wood() && node.length >= BRANCH_THRESHOLD && node.children.is_empty() {

                let offset = if node.node_type.left_node() { -BRANCH_ELEVATION } else { BRANCH_ELEVATION };
                
                // Capture the randomized elevation for the continuation segment first
                let continuation_elevation = node.elevation + rand::random_range(-BRANCH_RANDOM_VARIATION..=BRANCH_RANDOM_VARIATION);
                let branch_elevation = node.elevation + offset + rand::random_range(-BRANCH_RANDOM_VARIATION..=BRANCH_RANDOM_VARIATION);
                let leaf_elevation = 2.0 * node.elevation -branch_elevation + rand::random_range(-BRANCH_RANDOM_VARIATION..=BRANCH_RANDOM_VARIATION); // opposite side of the branch

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
                if node.age >= MIN_ACTIVATION_AGE {
                    if let Some(parent_idx) = node.parent {
                        if self.nodes[parent_idx].auxin_received <= AUXIN_THRESHOLD {
                            events.push(TreeEvent::Activate(index));
                        }
                    }
                }
            } else if node.node_type.is_active_wood() && node.auxin_received > AUXIN_THRESHOLD {
                events.push(TreeEvent::Deactivate(index));
            }
        }
        self.apply_events(&events);
        events
    }

    fn update_auxin(&mut self) {
        let mut auxin_values = vec![0.0f32; self.nodes.len()];
        self.compute_auxin(0, &mut auxin_values);
        for (index, value) in auxin_values.iter().enumerate() {
            self.nodes[index].auxin_received = *value;
        }
    }

    fn compute_auxin(&self, node_index: usize, values: &mut Vec<f32>) -> f32 {
        let node = &self.nodes[node_index];
        let children = node.children.clone();
        
        let mut received = 0.0;
        for child in children {
            received += self.compute_auxin(child, values);
        }
        
        let produced = if node.node_type.is_active_wood() { AUXIN_PRODUCTION } else { 0.0 };
        let consumed = node.length * node.thickness * AUXIN_CONSUMPTION_RATE;
        let surplus = (produced + received - consumed).max(0.0);
        
        values[node_index] = received;
        surplus
    }

    fn update_pipes(&mut self) {
        let mut pipe_values = vec![0.0f32; self.nodes.len()];
        self.compute_pipes(0, &mut pipe_values);
        for (index, value) in pipe_values.iter().enumerate() {
            self.nodes[index].thickness = value.max(self.nodes[index].thickness);
        }
    }

    fn compute_pipes(&mut self, node_index: usize, values: &mut Vec<f32>) -> f32 {
        let node = &self.nodes[node_index];
        let children = node.children.clone();

        let mut sum_pipes = 0.0;
        if children.is_empty() {
            sum_pipes = 1.0; // leaf node contributes 1 pipe
        } else {
            for child in children {
                sum_pipes += self.compute_pipes(child, values).powf(2.0);
            }
            sum_pipes = sum_pipes.sqrt();
        }
        values[node_index] = sum_pipes;
        sum_pipes
    }

    fn apply_events(&mut self, events: &[TreeEvent]) {
        for event in events {
            match event {
                TreeEvent::Grow(index) => {
                    self.nodes[*index].age += 1;
                    if self.nodes[*index].node_type.is_active_wood() && self.nodes[*index].children.is_empty() {
                        if rand::random_range(0.0..1.0) < NODE_ACTIVITY_PROBABILITY {
                            self.nodes[*index].length += GROWTH_RATE_LENGTH;
                        
                            // Gravitropic correction
                            let deviation = self.nodes[*index].elevation - 90.0;
                            if deviation.abs() > GRAVITROPISM_THRESHOLD {
                                let excess = deviation.abs() - GRAVITROPISM_THRESHOLD;
                                let max_excess = 90.0 - GRAVITROPISM_THRESHOLD; // how far past threshold is possible
                                let strength = excess / max_excess; // 0.0 at threshold, 1.0 at horizontal
                                let correction = GRAVITROPISM_RATE * strength;
                                if deviation > 0.0 {
                                    self.nodes[*index].elevation -= correction;
                                } else {
                                    self.nodes[*index].elevation += correction;
                                }
                            }
                        }
                    }
                }
                TreeEvent::Branch { parent, node_type, elevation, azimuth } => {
                    let length = if node_type.is_leaf() { 0.0 } else { 1.0 };
                    self.add_node(*parent, node_type.clone(), *elevation, *azimuth, length);
                }
                TreeEvent::Activate(index) => {
                    if self.nodes[*index].node_type.is_inactive_wood() {
                        self.nodes[*index].node_type.set_active(true);
                    }
                }
                TreeEvent::Deactivate(index) => {
                    if self.nodes[*index].node_type.is_active_wood() {
                        self.nodes[*index].node_type.set_active(false);
                    }
                }

            }
        }
    }

    fn add_node(&mut self, parent_index: usize, node_type: NodeType, elevation: f32, azimuth: f32, length: f32) {
        let new_node_index = self.nodes.len();
        self.nodes.push(TreeNode::new(Some(parent_index), node_type, elevation, azimuth, length));
        self.nodes[parent_index].children.push(new_node_index);
    }

}

enum TreeEvent {
    Grow(usize),
    Branch { parent: usize, node_type: NodeType, elevation: f32, azimuth: f32 },
    Deactivate(usize),
    Activate(usize),
}

struct Segment {
    x1: f32, y1: f32,
    x2: f32, y2: f32,
    x3: f32, y3: f32,
    x4: f32, y4: f32,
    radius: f32,
    sweep: bool,
}

struct Leaf {
    // Petiole start
    px: f32, py: f32,
    // Base (petiole end / fan origin)
    bx: f32, by: f32,
    // Stem to bottom left arc endpoint
    blx: f32, bly: f32,
    bl_radius: f32,
    // Bottom left corner
    lcx: f32, lcy: f32,
    // Left corner arc endpoint
    lax: f32, lay: f32,
    la_radius: f32,
    // Divot left cubic endpoint + controls
    dlx: f32, dly: f32,
    dl_c1x: f32, dl_c1y: f32,
    dl_c2x: f32, dl_c2y: f32,
    // Divot right cubic endpoint + controls
    drx: f32, dry: f32,
    dr_c1x: f32, dr_c1y: f32,
    dr_c2x: f32, dr_c2y: f32,
    // Right corner arc endpoint
    rax: f32, ray: f32,
    ra_radius: f32,
    // Bottom right corner
    rcx: f32, rcy: f32,
    // Stem to base arc endpoint
    srx: f32, sry: f32,
    sr_radius: f32,
    // Arc sweeps
    stem_sweep: bool,
    corner_sweep: bool,
}

impl From<&Segment> for SegmentData {
    fn from(s: &Segment) -> Self {
        SegmentData {
            x1: s.x1, y1: s.y1,
            x2: s.x2, y2: s.y2,
            x3: s.x3, y3: s.y3,
            x4: s.x4, y4: s.y4,
            radius: s.radius,
            sweep: s.sweep,
        }
    }
}

impl From<&Leaf> for LeafData {
    fn from(l: &Leaf) -> Self {
        LeafData {
            px: l.px, py: l.py,
            bx: l.bx, by: l.by,
            blx: l.blx, bly: l.bly, bl_radius: l.bl_radius,
            lcx: l.lcx, lcy: l.lcy,
            lax: l.lax, lay: l.lay, la_radius: l.la_radius,
            dlx: l.dlx, dly: l.dly, dl_c1x: l.dl_c1x, dl_c1y: l.dl_c1y, dl_c2x: l.dl_c2x, dl_c2y: l.dl_c2y,
            drx: l.drx, dry: l.dry, dr_c1x: l.dr_c1x, dr_c1y: l.dr_c1y, dr_c2x: l.dr_c2x, dr_c2y: l.dr_c2y,
            rax: l.rax, ray: l.ray, ra_radius: l.ra_radius,
            rcx: l.rcx, rcy: l.rcy,
            srx: l.srx, sry: l.sry, sr_radius: l.sr_radius,
            stem_sweep: l.stem_sweep,
            corner_sweep: l.corner_sweep,
        }
    }
}

fn tree_to_model(tree: &Tree, origin: (f32, f32)) -> (ModelRc<SegmentData>, ModelRc<LeafData>) {
    let mut segments = Vec::new();
    let mut leaves = Vec::new();
    nodes_to_renderables(tree, 0, origin, &mut segments, &mut leaves);
    let slint_segments: Vec<SegmentData> = segments.iter().map(SegmentData::from).collect();
    let slint_leaves: Vec<LeafData> = leaves.iter().map(LeafData::from).collect();
    (ModelRc::new(VecModel::from(slint_segments)), ModelRc::new(VecModel::from(slint_leaves)))
}

fn nodes_to_renderables(tree: &Tree, node_index: usize, pos: (f32, f32), segments: &mut Vec<Segment>, leaves: &mut Vec<Leaf>) {
    let node = &tree.nodes[node_index];
    let elevation_rad = node.elevation.to_radians();
    let end = (pos.0 + (node.length as f32 * elevation_rad.cos()), pos.1 - (node.length as f32 * elevation_rad.sin()));
    if node.node_type.is_wood() {
        let thickness = node.thickness * BRANCH_THICKNESS_FACTOR; // scale thickness for better visibility

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
        let parent = &tree.nodes[node.parent.unwrap()];
        let visual_half = parent.thickness * BRANCH_THICKNESS_FACTOR / 2.0;
        let parent_rad = parent.elevation.to_radians();
        let start_x = pos.0 + visual_half * parent_rad.cos();
        let start_y = pos.1 - visual_half * parent_rad.sin();
        leaves.push(leaf_to_points(start_x, start_y, node.elevation, 12.0));
    }

    for child in &node.children {
        nodes_to_renderables(tree, *child, end, segments, leaves);
    }
}


fn leaf_to_points(x: f32, y: f32, angle: f32, size: f32) -> Leaf {
    let cos_a = angle.to_radians().cos();
    let sin_a = angle.to_radians().sin();

    // Transform a normalized point to screen coordinates
    let t = |nx: f32, ny: f32| -> (f32, f32) {
        (x + (-ny * cos_a + nx * sin_a) * size,
        y + ( ny * sin_a + nx * cos_a) * size)
    };

    let (px, py) = t(0.0, 0.0);           // petiole start at origin
    let (bx, by) = t(0.0, -0.3125);       // fan base
    let (blx, bly) = t(-0.25, -0.5625);
    let (lcx, lcy) = t(-0.875, -0.5625);
    let (lax, lay) = t(-1.0, -0.6875);
    let (dlx, dly) = t(0.0, -1.1875);
    let (dl_c1x, dl_c1y) = t(-0.875, -1.3125);
    let (dl_c2x, dl_c2y) = t(0.0, -1.5625);
    let (drx, dry) = t(1.0, -0.6875);
    let (dr_c1x, dr_c1y) = t(0.0, -1.5625);
    let (dr_c2x, dr_c2y) = t(0.875, -1.3125);
    let (rax, ray) = t(0.875, -0.5625);
    let (rcx, rcy) = t(0.25, -0.5625);
    let (srx, sry) = t(0.0, -0.3125);

    Leaf {
        px, py,
        bx, by,
        blx, bly, bl_radius: 0.25 * size,
        lcx, lcy,
        lax, lay, la_radius: 0.125 * size,
        dlx, dly, dl_c1x, dl_c1y, dl_c2x, dl_c2y,
        drx, dry, dr_c1x, dr_c1y, dr_c2x, dr_c2y,
        rax, ray, ra_radius: 0.125 * size,
        rcx, rcy,
        srx, sry, sr_radius: 0.25 * size,
        stem_sweep: false,
        corner_sweep: true,
    }
}

slint::include_modules!();
fn main() -> Result<(), Box<dyn Error>> {
    // Decalare App and create weak references for the event loop and timer
    let app = AppWindow::new()?;
    let weak_for_winit = app.as_weak();
    let weak_for_timer = app.as_weak();

    // Declare timer
    let timer = slint::Timer::default();

    // Get display information and calculate the number of columns and rows based on the primary display's width, height, and scale factor
    let displays = DisplayInfo::all().unwrap();
    let primary = displays.iter().find(|d| d.is_primary).expect("No primary display found");
    let width = primary.width;
    let height = primary.height;
    let scale = primary.scale_factor;

    let root_x = (width as f32 / scale) / 2.0;
    let root_y: f32 = (height as f32 *0.95) as f32 / scale;
    let origin = (root_x as f32, root_y as f32);

    // Invoke a function from the event loop to maximize the window and disable cursor hittesting
    slint::invoke_from_event_loop(move || {
        let app = weak_for_winit.unwrap();
        app.window().set_maximized(true);
        app.window().with_winit_window(|winit_win| {
            winit_win.set_cursor_hittest(false).unwrap();
        });        
    }).unwrap();

    // Create a tree data structure
    let mut tree = Tree::new(); 

     // start the slint timer
    timer.start(TimerMode::Repeated, std::time::Duration::from_millis(CLOCK_INTERVAL_MS), move || {
        let app = weak_for_timer.unwrap();
        let _events = tree.tick();
        let (segments, leaves) = tree_to_model(&tree, origin);
        app.set_leaves(leaves);
        app.set_segments(segments);
    }); 

    // Run the app
    app.run()?;
    Ok(())
}