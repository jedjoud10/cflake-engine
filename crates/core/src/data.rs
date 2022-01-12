use ecs::ECSManager;
use input::InputManager;
use io::SaverLoader;
use ui::UIManager;

use crate::GameConfig;

// The whole world that stores our managers and data
pub struct World {
    pub input: InputManager,
    pub ui: UIManager,
    pub ecs: ECSManager,
    pub(crate) components
    pub io: SaverLoader,
    pub config: GameConfig,
}
// Some context that stores a reference to all of the world managers and data
pub struct Context {

}