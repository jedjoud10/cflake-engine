use crate::{EventSet, Settings};
use audio::player::AudioPlayer;
use ecs::EcsManager;
use getset::*;
use globals::GlobalsSet;
use gui::GUIManager;
use input::InputManager;
use io::IOManager;
use others::Time;
use physics::PhysicsSimulation;
use rendering::pipeline::{Pipeline, SceneRenderer};

// The current state of the world
#[derive(Clone, Copy)]
pub enum WorldState {
    StartingUp,
    Running,
    Exit,
    Paused,
}

// The whole world that stores our managers and data
#[derive(Getters, CopyGetters, Setters)]
pub struct World {
    // User
    pub input: InputManager,
    pub io: IOManager,

    // Rendering
    pub pipeline: Pipeline,
    pub renderer: SceneRenderer,
    pub gui: GUIManager,

    // Logic
    pub state: WorldState,
    pub ecs: EcsManager,
    pub events: EventSet,
    pub globals: GlobalsSet,
    pub physics: PhysicsSimulation,

    // Other
    pub time: Time,
    pub settings: Settings,
    pub audio: AudioPlayer,
}

// World implementation
impl World {
    // Create a new world
    pub fn new(settings: Settings, io: IOManager, mut pipeline: Pipeline, renderer: SceneRenderer) -> Self {
        let gui = gui::GUIManager::new(&mut pipeline);
        let mut world = World {
            input: Default::default(),
            time: Default::default(),
            gui,
            ecs: EcsManager::default(),
            globals: Default::default(),
            io,
            settings: Default::default(),
            pipeline,
            renderer,
            state: WorldState::StartingUp,
            audio: Default::default(),
            physics: PhysicsSimulation::new(),
            events: Default::default(),
        };
        // Just set the game settings and we are done
        world.settings = settings;
        println!("World init done!");
        world
    }
    // Called each frame
    pub fn update(&mut self, delta: f32) {
        self.state = WorldState::Running;
        // Update the timings
        self.time.update(delta);

        // Update game logic (this includes rendering the world)
        self.pipeline.start_frame(&mut self.renderer, self.time.delta(), self.time.elapsed());
        self.gui.begin_frame(self.pipeline.window().context().window());

        // Read the systems
        let (systems, settings) = self.ecs.ready(self.time.current().as_ref().unwrap().count);
        let systems = systems.borrow();

        // Get events
        let event_set = self.events.ecs.clone();
        let events = event_set.events();
        EcsManager::execute_systems::<Self>(systems, self, &events, settings);

        // Late update
        self.pipeline.end_frame();
        self.input.late_update();
    }
    // We must destroy the world
    pub fn destroy(&mut self) {
        // Quit the saver loader
        self.io.quit();
    }
}
