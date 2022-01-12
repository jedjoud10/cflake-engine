// 


lazy_static! {
    pub(crate) static ref INPUT_MANAGER: OwnedContext<InputManager> =  OwnedContext::default();
    pub(crate) static ref UI_MANAGER: OwnedContext<UIManager> =  OwnedContext::default();
    pub(crate) static ref ECS_MANAGER: OwnedContext<ECSManager> = OwnedContext::default();
    
    // Miscs
    pub(crate) static ref CUSTOM_DATA: OwnedContext<CustomWorldData> =  OwnedContext::default();
    pub(crate) static ref TIME: OwnedContext<Time> =  OwnedContext::default();
    pub(crate) static ref IO: OwnedContext<SaverLoader> =  OwnedContext::default();
    pub(crate) static ref CONFIG_FILE: OwnedContext<GameConfig> =  OwnedContext::default();
}
// Create a new world
pub fn new(author_name: &str, app_name: &str)  {
    let saver_loader = IO.borrow_mut();
    *saver_loader = SaverLoader::new(author_name, app_name);
}
// When the world started initializing
pub fn start_world(glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
    println!("Starting world...");
    // Load the default stuff
    crate::global::input::create_key_cache();
    crate::global::input::bind_key(Keys::F4, "toggle_console", MapType::Button);
    crate::global::input::bind_key(Keys::Enter, "enter", MapType::Button);
    crate::global::input::bind_key(Keys::F2, "debug", MapType::Button);

    // Create some default UI that prints some default info to the screen
    let mut root = ui::Root::new(1);
    // ----Add the elements here----

    // Create a text element
    for x in 0..2 {
        let text_element_1 = ui::Element::new()
            .set_coordinate_system(ui::CoordinateType::Pixel)
            .set_position(veclib::Vector2::Y * 40.0 * x as f32)
            .set_text("", 40.0);
        root.add_element(text_element_1);
    }

    // Set this as the default root
    crate::global::ui::add_root("default", root);

    // Create the default root for the console
    let mut console_root = ui::Root::new(64);
    let console_panel = ui::Element::new()
        .set_coordinate_system(ui::CoordinateType::Factor)
        .set_color(veclib::Vector4::new(0.0, 0.0, 0.0, 0.7));
    let console_panel_id = console_root.add_element(console_panel);
    let console_text = ui::Element::new()
        .set_coordinate_system(ui::CoordinateType::Pixel)
        .set_position(veclib::Vector2::ZERO)
        .set_size(veclib::Vector2::ONE)
        .set_text("text", 30.0);
    let console_text_id = console_root.add_element(console_text);
    ui::Element::attach(&mut console_root, console_panel_id, vec![console_text_id]);
    console_root.visible = false;
    crate::global::ui::add_root("console", console_root);
    window_commands::hide_cursor(window);
    // Load the config file for this world
    let config_file_copy = crate::global::io::create_config_file();
    // Apply the config file's data to the rendering window
    window_commands::set_fullscreen(config_file_copy.fullscreen, glfw, window);
    window_commands::set_vsync(config_file_copy.vsync);
    println!("Hello world from MainThread! Must call initalization callback!");
}

// Update the console
fn update_console() {
    /*
    // Check if we should start key registering if the console is active
    if self.input_manager.map_pressed_uncheck("toggle_console") || (self.input_manager.map_pressed_uncheck("enter") && self.input_manager.keys_reg_active()) {
        match self.input_manager.toggle_keys_reg() {
            Some(x) => {
                // Hide the console
                let console_root = self.ui_manager.get_root_mut("console");
                console_root.visible = false;
                self.debug.console.detect_command(x);
            }
            None => {
                // Enable the console
                let console_root = self.ui_manager.get_root_mut("console");
                console_root.visible = true;
            }
        }
    }

    // Update the console everytime
    match self.input_manager.full_sentence.as_ref() {
        Some(x) => {
            let console_text = self.ui_manager.get_root_mut("console").get_element_mut(2);
            let console_string = format!("Com: '{}'", x.clone().as_str());
            console_text.update_text(console_string.as_str(), 40.0);
        }
        None => {
            // We don't have to update anything
        }
    }
    */
}
// When we want to close the application
pub fn kill_world(pipeline_data: PipelineStartData) {
    // Killing world
    println!("Killing world!");
}