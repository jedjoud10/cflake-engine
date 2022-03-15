use crate::{Settings, WorldState};
use audio::player::AudioPlayer;
use ecs::ECSManager;
use globals::GlobalsSet;
use gui::GUIManager;
use input::InputManager;
use io::IOManager;
use others::Time;
use physics::PhysicsSimulation;
use rendering::pipeline::{Pipeline, SceneRenderer};

// The whole world that stores our managers and data
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
    pub ecs: ECSManager<Self>,
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
            ecs: ECSManager::<Self>::default(),
            globals: Default::default(),
            io,
            settings: Default::default(),
            pipeline,
            renderer,
            state: WorldState::StartingUp,
            audio: Default::default(),
            physics: PhysicsSimulation::new(),
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
        self.pipeline.start_frame(&mut self.renderer, self.time.delta, self.time.elapsed);
        self.gui.begin_frame(self.pipeline.window().context().window());

        let (systems, settings) = self.ecs.ready();
        let systems = systems.borrow();
        ECSManager::<World>::execute_systems(systems, self, settings);

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
