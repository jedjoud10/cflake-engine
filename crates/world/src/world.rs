use crate::{Settings, WorldState};
use audio::player::AudioPlayer;
use ecs::ECSManager;
use globals::GlobalsCollection;
use gui::GUIManager;
use input::InputManager;
use io::IOManager;
use others::Time;
use physics::PhysicsSimulation;
use rendering::pipeline::Pipeline;
use std::sync::Arc;

// The whole world that stores our managers and data
pub struct World {
    pub input: InputManager,
    pub time: Time,
    pub gui: GUIManager,
    pub ecs: ECSManager<Self>,
    pub globals: GlobalsCollection,
    pub io: IOManager,
    pub settings: Settings,
    pub pipeline: Pipeline,
    pub state: WorldState,
    pub audio: AudioPlayer,
    pub physics: PhysicsSimulation,
}

// World implementation
impl World {
    // Create a new world
    pub fn new(settings: Settings, io: IOManager, mut pipeline: Pipeline) -> Self {
        let gui = gui::GUIManager::new(&mut pipeline);
        let mut world = World {
            input: Default::default(),
            time: Default::default(),
            gui,
            ecs: ECSManager::<Self>::default(),
            globals: Default::default(),
            io,
            settings: Default::default(),
            pipeline,
            state: WorldState::StartingUp,
            audio: Default::default(),
            physics: PhysicsSimulation::new(),
        };
        // Just set the game settings and we are done
        world.settings = settings;
        println!("World init done!");
        world
    }
    // Update the ECS systems
    fn update_ecs(&mut self) {
        let system_count = self.ecs.count_systems();
        // Loop for every system and update it
        for system_index in 0..system_count {
            let execution_data = {
                let system = &self.ecs.get_systems()[system_index];
                system.run_system(&self.ecs)
            };
            // Actually execute the system now
            execution_data.run(self);
            {
                // Clear
                let system = &self.ecs.get_systems()[system_index];
                system.clear();
                self.time.update_current_frame_time();
            }
        }
        // Finish update
        self.ecs.finish_update();
    }
    // Called each frame
    pub fn update(&mut self) {
        self.state = WorldState::Running;
        // Update game logic (this includes rendering the world)
        self.update_ecs();
    }
    // We must destroy the world
    pub fn destroy(&mut self) {
        // Quit the saver loader
        self.io.quit();
    }
}
