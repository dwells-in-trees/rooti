use slint::*;
use i_slint_backend_winit::WinitWindowAccessor;
use display_info::DisplayInfo;
use std::error::Error;

const BRANCH_THRESHOLD: i32 = 50;
const BRANCH_ELEVATION: f32 = 30.0;

struct Tree {
    nodes: Vec<TreeNode>,
}

struct TreeNode {
    parent: Option<usize>,
    children: Vec<usize>,
    length: i32,
    elevation: f32,
    azimuth: f32,
    is_active: bool,
    next_bud_left: bool,
}

impl TreeNode {
    fn new(parent: Option<usize>, elevation: f32, azimuth: f32, length: i32, is_active: bool, next_bud_left: bool) -> Self {
        Self { parent, children: Vec::new(), length, elevation, azimuth, is_active, next_bud_left }
    }
}


impl Tree {
    fn new() -> Self {
        Self { nodes: vec![TreeNode::new(None, 90.0, 200.0, 1, true, false)] }
    }

    fn tick(&mut self) -> Vec<TreeEvent> {
        let mut events = Vec::new();

        for (index, node) in self.nodes.iter().enumerate() {
            if node.is_active {
                events.push(TreeEvent::Grow(index));

                let offset = if node.next_bud_left { -BRANCH_ELEVATION } else { BRANCH_ELEVATION };
                if node.length >= BRANCH_THRESHOLD {
                    // continutation segment
                    events.push(TreeEvent::Branch {
                        parent: index,
                        elevation: node.elevation + rand::random_range(-5.0..=5.0),
                        azimuth: node.azimuth,
                        is_active: true,
                        next_bud_left: node.next_bud_left,
                    });
                    // branch segment
                    events.push(TreeEvent::Branch {
                        parent: index,
                        elevation: node.elevation + offset + rand::random_range(-5.0..=5.0),
                        azimuth: node.azimuth,
                        is_active: true,
                        next_bud_left: !node.next_bud_left,
                    });
                    // deactivate the parent node
                    events.push(TreeEvent::Deactivate(index));
                }

            }
        }

        self.apply_events(&events);
        events
    }

    fn apply_events(&mut self, events: &[TreeEvent]) {
        for event in events {
            match event {
                TreeEvent::Grow(index) => {
                    self.nodes[*index].length += 1;
                }
                TreeEvent::Branch { parent, elevation, azimuth, is_active, next_bud_left } => {
                    self.add_node(*parent, *elevation, *azimuth, 1, *is_active, *next_bud_left);
                }
                TreeEvent::Deactivate(index) => {
                    self.nodes[*index].is_active = false;
                }
                TreeEvent::ToggleBudSide(index) => {
                    self.nodes[*index].next_bud_left = !self.nodes[*index].next_bud_left;
                }
            }
        }
    }

    fn add_node(&mut self, parent_index: usize, elevation: f32, azimuth: f32, length: i32, is_active: bool, next_bud_left: bool) {
        let new_node_index = self.nodes.len();
        self.nodes.push(TreeNode::new(Some(parent_index), elevation, azimuth, length, is_active, next_bud_left));
        self.nodes[parent_index].children.push(new_node_index);
    }
}

enum TreeEvent {
    Grow(usize),
    Branch { parent: usize, elevation: f32, azimuth: f32, is_active: bool, next_bud_left: bool },
    Deactivate(usize),
    ToggleBudSide(usize),
}

struct Segment {
    start: (f32, f32),
    end: (f32, f32),
}

impl From<&Segment> for SegmentData {
    fn from(s: &Segment) -> Self {
        SegmentData {
            start_x: s.start.0 as f32,
            start_y: s.start.1 as f32,
            end_x: s.end.0 as f32,
            end_y: s.end.1 as f32,
        }
    }
}

fn tree_to_model(tree: &Tree, origin: (f32, f32)) -> ModelRc<SegmentData> {
    let mut segments = Vec::new();
    nodes_to_segments(tree, 0, origin, &mut segments);
    let slint_segments: Vec<SegmentData> = segments.iter().map(SegmentData::from).collect();
    ModelRc::new(VecModel::from(slint_segments))
}

fn nodes_to_segments(tree: &Tree, node_index: usize, pos: (f32, f32), cells: &mut Vec<Segment>) {
    let node = &tree.nodes[node_index];
    let elevation_rad = node.elevation.to_radians();
    let end = (pos.0 + (node.length as f32 * elevation_rad.cos()), pos.1 - (node.length as f32 * elevation_rad.sin()));
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
    timer.start(TimerMode::Repeated, std::time::Duration::from_millis(100), move || {
        let app = weak_for_timer.unwrap();
        let _events = tree.tick();
        app.set_segments(tree_to_model(&tree, origin));
    }); 

    // Run the app
    app.run()?;
    Ok(())
}