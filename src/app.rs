use slint::*;
use i_slint_backend_winit::{WinitWindowAccessor, winit::platform::windows::WindowExtWindows};
use display_info::DisplayInfo;
use std::{ error::Error, rc::Rc, cell::RefCell };
use crate::{ render, tree };

include_modules!();

pub fn run() -> Result<(), Box<dyn Error>> {

    const CLOCK_INTERVAL_MS: u64 = 5;

    // Declare App and create weak references for the event loop and timer
    let app = AppWindow::new()?;
    let settings= SettingsWindow::new()?;
    let weak_for_winit = app.as_weak();
    let weak_for_timer = app.as_weak();
    let weak_settings = settings.as_weak();

    // Declare timer
    let timer = Timer::default();

    // Get display information and calculate the number of columns and rows based on the primary display's width, height, and scale factor
    let displays = DisplayInfo::all()?;
    let primary = displays.iter().find(|d| d.is_primary).expect("No primary display found");
    let width = primary.width;
    let height = primary.height;
    let scale = primary.scale_factor;

    let root_x = (width as f32 / scale) / 2.0;
    let root_y: f32 = (height as f32 *0.95) / scale;
    let origin = (root_x, root_y);

    // Invoke a function from the event loop to maximize the window and disable cursor hit-testing
    invoke_from_event_loop(move || {
        let app = weak_for_winit.unwrap();
        app.window().set_maximized(true);
        app.window().with_winit_window(|winit_win| {
            winit_win.set_cursor_hittest(false).unwrap();
            winit_win.set_skip_taskbar(true);
        });
    })?;

    // Create a tree data structure
    let tree = Rc::new(RefCell::new(tree::Tree::new()));
    let tree_for_timer = tree.clone();
    let tree_for_reset = tree.clone();

    // callback for resetting tree
    settings.on_reset_tree(move || {
        *tree_for_reset.borrow_mut() = tree::Tree::new();
    });

    // start the slint timer
    timer.start(TimerMode::Repeated, std::time::Duration::from_millis(CLOCK_INTERVAL_MS), move || {
        let app = weak_for_timer.unwrap();
        let settings = weak_settings.unwrap();

        // Refresh the tree config for slint render loop
        let tree_config = tree::TreeConfig {
            node_activity_probability: settings.get_node_activity_probability(),
            branch_threshold: settings.get_branch_threshold(),
            branch_elevation: settings.get_branch_elevation(),
            branch_random_variation: settings.get_branch_random_variation(),
            growth_rate_length: settings.get_growth_rate_length(),
            auxin_production: settings.get_auxin_production(),
            auxin_threshold: settings.get_auxin_threshold(),
            auxin_consumption_rate: settings.get_auxin_consumption_rate(),
            min_activation_age: settings.get_min_activation_age() as u32,
            gravitropism_threshold: settings.get_gravitropism_threshold(),
            gravitropism_rate: settings.get_gravitropism_rate(),
            leaf_shape: tree::node::LeafShape::Ginko,
            leaf_placement: tree::node::LeafPlacement::AtBranchPoints,
        };

        if !settings.get_paused() {
            let _events = tree::growth::tick(&mut *tree_for_timer.borrow_mut(), &tree_config);
        }

        let _config_for_render = render::RenderConfig {
            branch_thickness_factor: settings.get_branch_thickness_factor(),
        };

        let (segments, leaves) = render::slint_bridge::tree_to_model(&tree_for_timer.borrow(), &tree_config, &_config_for_render, origin);
        app.set_leaves(leaves);
        app.set_segments(segments);
    });

    // Run the app
    let _s = settings.show();
    app.run()?;
    Ok(())
}