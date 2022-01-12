use ecs::{ECSManager, ComponentManager};
use input::InputManager;
use io::SaverLoader;
use ui::UIManager;

use crate::GameConfig;

// The whole world that stores our managers and data
pub struct World {
    pub input: InputManager,
    pub ui: UIManager,
    pub ecs: ECSManager<Self>,
    pub io: SaverLoader,
    pub config: GameConfig,
    pub(crate) cmanager: ComponentManager,
}
// Some context that stores a reference to all of the world managers and data
pub struct Context<'a> {
    pub input: &'a InputManager,
    pub ui: &'a UIManager,
    pub ecs: &'a ECSManager<World>,
    pub io: &'a SaverLoader,
    pub config: &'a GameConfig,
}

impl<'a> Context<'a> {
    // Convert a world into a context, so we can share it around multiple threads
    pub fn convert(world: &World) -> Self {
        Self {
            input: &world.input,
            ui: &world.ui,
            ecs: &world.ecs,
            io: &world.io,
            config: &world.config,
        }
    }
}