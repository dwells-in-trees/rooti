use slint::*;
use i_slint_backend_winit::WinitWindowAccessor;
use display_info::DisplayInfo;
use std::error::Error;

struct Tree {
    nodes: Vec<TreeNode>,
}

struct TreeNode {
    parent: Option<usize>,
    children: Vec<usize>,
    length: i32,
    angle: f32,
}

impl TreeNode {
    fn new(parent: Option<usize>, angle: f32, length: i32) -> Self {
        Self { parent, children: Vec::new(), length, angle }
    }
}

impl Tree {
    fn new() -> Self {
        Self { nodes: vec![TreeNode::new(None, 90.0, 200)] }
    }

    fn grow_tree_node(&mut self, node_index: usize) {
        self.nodes[node_index].length += 1;
    }

    fn get_all_nodes(&self) -> &Vec<TreeNode> {
        &self.nodes
    }

    fn add_node(&mut self, parent_index: usize, angle: f32, length: i32) {
        let new_node_index = self.nodes.len();
        self.nodes.push(TreeNode::new(Some(parent_index), angle, length));
        self.nodes[parent_index].children.push(new_node_index);
    }
}

struct Segment {
    start: (f32, f32),
    end: (f32, f32),
}

fn nodes_to_segments(tree: &Tree, node_index: usize, pos: (f32, f32), cells: &mut Vec<Segment>) {
    let node = &tree.nodes[node_index];
    let angle_rad = node.angle.to_radians();
    let end = (pos.0 + (node.length as f32 * angle_rad.cos()), pos.1 - (node.length as f32 * angle_rad.sin()));
    cells.push(Segment { start: pos, end });

    for child in &node.children {
        nodes_to_segments(tree, *child, end, cells);
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

    let root_x = (width as f32 / scale) / 5.0;
    let root_y: f32 = height as f32 / scale;
    let origin = (root_x, root_y);

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
    tree.add_node(0, 45.0, 150);
    tree.add_node(0, 135.0, 150);

    let mut segments = Vec::new();
    nodes_to_segments(&tree, 0, origin, &mut segments);

    for segment in &segments {
        println!("Segment from ({}, {}) to ({}, {})", segment.start.0, segment.start.1, segment.end.0, segment.end.1);
    }

    use slint::{ModelRc, VecModel};

    let slint_segments: Vec<SegmentData> = segments.iter().map(|s| SegmentData {
        start_x: s.start.0,
        start_y: s.start.1,
        end_x: s.end.0,
        end_y: s.end.1,
    }).collect();

    let model = ModelRc::new(VecModel::from(slint_segments));
    app.set_segments(model);

/*     // Start the timer to update the glyph text every second with a random character
    timer.start(TimerMode::Repeated, std::time::Duration::from_millis(1000), move || {
        let app = weak_for_timer.unwrap();
        tree.grow_tree_node(0);
        let mut cells = Vec::new();
        nodes_to_2d_grid(&tree, 0, (0,0), &mut cells);
    }); */

    // Run the app
    app.run()?;
    Ok(())
}