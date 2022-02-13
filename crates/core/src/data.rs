use crate::GameSettings;
use ecs::ECSManager;
use globals::GlobalCollection;
use input::InputManager;
use io::SaverLoader;
use others::Time;
use rendering::pipeline::PipelineContext;
use gui::GUIManager;

// The whole world that stores our managers and data
pub struct World {
    pub input: InputManager,
    pub time: Time,
    pub gui: GUIManager,
    pub ecs: ECSManager<Self>,
    pub globals: GlobalCollection,
    pub io: SaverLoader,
    pub settings: GameSettings,
    pub pipeline: PipelineContext,
}
