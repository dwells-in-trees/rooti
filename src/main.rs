use slint::*;
use i_slint_backend_winit::WinitWindowAccessor;
use display_info::DisplayInfo;
use std::{error::Error };

const CLOCK_INTERVAL_MS: u64 = 5;
const BRANCH_THRESHOLD: f32 = 75.0;
const BRANCH_ELEVATION: f32 = 30.0;
const BRANCH_RANDOM_VARIATION: f32 = 12.0;
const GROWTH_RATE_LENGTH: f32 = 1.0;
const AUXIN_PRODUCTION: f32 = 1.0;
const AUXIN_THRESHOLD: f32 = 0.1;
const AUXIN_CONSUMPTION_RATE: f32 = 0.05;
const MIN_ACTIVATION_AGE: u32 = 100;
const GRAVITROPISM_THRESHOLD: f32 = 80.0; // degrees from 90° before correction kicks in
const GRAVITROPISM_RATE: f32 = 0.1;
const NODE_ACTIVITY_PROBABILITY: f32 = 0.30;

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
    is_active: bool,
    next_bud_left: bool,
    auxin_received: f32,
    age: u32,
}

impl TreeNode {
    fn new(parent: Option<usize>, elevation: f32, azimuth: f32, length: f32, is_active: bool, next_bud_left: bool) -> Self {
        Self { parent, children: Vec::new(), length, thickness: 1.0, elevation, azimuth, is_active, next_bud_left, auxin_received: 0.0, age: 0 }
    }
}


impl Tree {
    fn new() -> Self {
        Self { nodes: vec![TreeNode::new(None, 90.0, 200.0, 1.0, true, false)] }
    }


    fn tick(&mut self) -> Vec<TreeEvent> {
        let mut events = Vec::new();
        
        self.update_auxin();
        self.update_pipes();

        // iterate through all nodes and generate growth events
        for (index, node) in self.nodes.iter().enumerate() {
            events.push(TreeEvent::Grow(index));
            if node.is_active && node.length >= BRANCH_THRESHOLD && node.children.is_empty() {
                // continue current limb
                let offset = if node.next_bud_left { -BRANCH_ELEVATION } else { BRANCH_ELEVATION };
                events.push(TreeEvent::Branch{ 
                    parent: index, 
                    elevation: node.elevation + rand::random_range(-BRANCH_RANDOM_VARIATION..=BRANCH_RANDOM_VARIATION), 
                    azimuth: node.azimuth,
                    is_active: true, 
                    next_bud_left: !node.next_bud_left,
                });
                events.push(TreeEvent::Branch {
                    parent: index, 
                    elevation: node.elevation + offset + rand::random_range(-BRANCH_RANDOM_VARIATION..=BRANCH_RANDOM_VARIATION), 
                    azimuth: node.azimuth,
                    is_active: false, 
                    next_bud_left: !node.next_bud_left,
                });
                events.push(TreeEvent::Deactivate(index))
            } else if !node.is_active && node.age >= MIN_ACTIVATION_AGE {
                if let Some(parent_idx) = node.parent {
                    if self.nodes[parent_idx].auxin_received <= AUXIN_THRESHOLD {
                        events.push(TreeEvent::Activate(index));
                    }
                }
            } else if node.is_active && node.auxin_received > AUXIN_THRESHOLD {
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
        
        let produced = if node.is_active { AUXIN_PRODUCTION } else { 0.0 };
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
        for (i, node) in self.nodes.iter().enumerate() {
            if node.children.is_empty() && node.thickness > 1.0 {
                println!("WARNING: leaf node {} has thickness {}", i, node.thickness);
            }
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
                sum_pipes = sum_pipes.sqrt();
            }
        }
        values[node_index] = sum_pipes;
        sum_pipes
    }

    fn apply_events(&mut self, events: &[TreeEvent]) {
        for event in events {
            match event {
                TreeEvent::Grow(index) => {
                    self.nodes[*index].age += 1;
                    if self.nodes[*index].is_active  && self.nodes[*index].children.is_empty() {
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
                TreeEvent::Branch { parent, elevation, azimuth, is_active, next_bud_left } => {
                    self.add_node(*parent, *elevation, *azimuth, 1.0, *is_active, *next_bud_left);
                }
                TreeEvent::Activate(index) => {
                    self.nodes[*index].is_active = true;
                }
                TreeEvent::Deactivate(index) => {
                    self.nodes[*index].is_active = false;
                }

            }
        }
    }

    fn add_node(&mut self, parent_index: usize, elevation: f32, azimuth: f32, length: f32, is_active: bool, next_bud_left: bool) {
        let new_node_index = self.nodes.len();
        self.nodes.push(TreeNode::new(Some(parent_index), elevation, azimuth, length, is_active, next_bud_left));
        self.nodes[parent_index].children.push(new_node_index);
    }
}

enum TreeEvent {
    Grow(usize),
    Branch { parent: usize, elevation: f32, azimuth: f32, is_active: bool, next_bud_left: bool },
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

fn tree_to_model(tree: &Tree, origin: (f32, f32)) -> ModelRc<SegmentData> {
    let mut segments = Vec::new();
    nodes_to_segments(tree, 0, origin, &mut segments);
    let slint_segments: Vec<SegmentData> = segments.iter().map(SegmentData::from).collect();
    ModelRc::new(VecModel::from(slint_segments))
}

/// Recursively converts tree nodes to segments for rendering. 
/// 
/// Each node is represented as a rectangle (segment) with thickness based on the number of pipes. The function calculates the end position of each segment based on the node's length and elevation, and then computes the corners of the rectangle to create a visually appealing branch. 
/// It also applies an overlap for child segments to ensure they connect smoothly to their parent segments.
fn nodes_to_segments(tree: &Tree, node_index: usize, pos: (f32, f32), cells: &mut Vec<Segment>) {
    let node = &tree.nodes[node_index];
    let thickness = node.thickness*5.0; // scale thickness for better visibility
    let elevation_rad = node.elevation.to_radians();

    // calculate end position of the segment based on length and elevation
    let end = (pos.0 + (node.length as f32 * elevation_rad.cos()), pos.1 - (node.length as f32 * elevation_rad.sin()));
    let dx = end.0 - pos.0;
    let dy = end.1 - pos.1;
    let len = (dx*dx + dy*dy).sqrt();
    // perpendicular unit vector
    let px = -dy / len * (thickness / 2.0);
    let py = dx / len * (thickness / 2.0);

    // compute corners of the rectangle representing the branch segment
    let (c1, c2, c3, c4) = if node.elevation > 90.0 {
        ((pos.0 - px, pos.1 - py), (pos.0 + px, pos.1 + py),
        (end.0 + px, end.1 + py), (end.0 - px, end.1 - py))
    } else {
        ((pos.0 + px, pos.1 + py), (pos.0 - px, pos.1 - py),
        (end.0 - px, end.1 - py), (end.0 + px, end.1 + py))
    };

    let cross = (c2.0 - c1.0) * (c3.1 - c2.1) - (c2.1 - c1.1) * (c3.0 - c2.0);
    if cross > 0.0 {
        // swap to reverse winding
        cells.push(Segment {
            x1: pos.0 - px, y1: pos.1 - py,
            x2: pos.0 + px, y2: pos.1 + py,
            x3: end.0 + px, y3: end.1 + py,
            x4: end.0 - px, y4: end.1 - py,
            radius: thickness / 2.0,
            sweep: false,
        });
    } else {
        cells.push(Segment {
            x1: pos.0 + px, y1: pos.1 + py,
            x2: pos.0 - px, y2: pos.1 - py,
            x3: end.0 - px, y3: end.1 - py,
            x4: end.0 + px, y4: end.1 + py,
            radius: thickness / 2.0,
            sweep: true,
        });
    }

    let parent_thickness = tree.nodes[node_index].thickness;

    for child in &node.children {
        let child_node = &tree.nodes[*child];
        let child_elevation_rad = child_node.elevation.to_radians();
        let overlap = parent_thickness / 2.0;
        let child_start = (
            end.0 - overlap * child_elevation_rad.cos(),
            end.1 + overlap * child_elevation_rad.sin(),
        );
        nodes_to_segments(tree, *child, child_start, cells);
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

    // Create a tree data structure and add some nodes to it
    let mut tree = Tree::new(); 

     // Start the timer to update the glyph text every second with a random character
    timer.start(TimerMode::Repeated, std::time::Duration::from_millis(CLOCK_INTERVAL_MS), move || {
        let app = weak_for_timer.unwrap();
        let _events = tree.tick();
        app.set_segments(tree_to_model(&tree, origin));
    }); 

    // Run the app
    app.run()?;
    Ok(())
}