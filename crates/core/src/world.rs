use rendering::PipelineStartData;

use crate::{data::World, GameConfig, WorldTaskReceiver};

// World implementation
impl World {
    // Create a new world
    pub fn new(author_name: &str, app_name: &str, pipeline_data: PipelineStartData) -> Self  {
        let mut world = World {
            input: Default::default(),
            time: Default::default(),
            ui: Default::default(),
            ecs: ecs::ECSManager::new(|thread_index| {
                // This is ran on every thread in the ECS thread pool
                rendering::init_coms();
                crate::sender::init_coms();
            }),
            io: io::SaverLoader::new(author_name, app_name),
            config: Default::default(),
            pipeline: pipeline_data.pipeline.clone(),
            pipeline_thread: pipeline_data
        };
        world.init();
        world
    }
    // Initialize the world
    fn init(&mut self) {
        println!("Initializing world...");
        // Load the default stuff
        self.input.create_key_cache();
        self.input.bind_key(input::Keys::F4, "toggle_console", input::MapType::Button);
        self.input.bind_key(input::Keys::Enter, "enter", input::MapType::Button);
        self.input.bind_key(input::Keys::F2, "debug", input::MapType::Button);

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
        self.ui.add_root("default", root);

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
        self.ui.add_root("console", console_root);

        // Load the config file (create it if it doesn't exist already)
        self.io.create_default("config\\game_config.json", &crate::GameConfig::default());
        // Then load
        let config: GameConfig = self.io.load("config\\game_config.json");
        self.config = config;
        // Apply the config file's data to the rendering window
        // TODO
        
        println!("World init done!");
    }
    // We update the world for one frame
    pub fn update(&mut self, task_receiver: &mut WorldTaskReceiver) {
        
    }
    // We must destroy the world
    pub fn destroy(&mut self) {

    }
}