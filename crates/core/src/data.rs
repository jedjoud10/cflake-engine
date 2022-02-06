use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use ecs::ECSManager;
use globals::GlobalCollection;
use input::InputManager;
use io::SaverLoader;
use others::Time;
use rendering::pipeline::PipelineContext;
use ui::UIManager;

use crate::{GameConfig, TaskSenderContext};

// The whole world that stores our managers and data
pub struct World {
    pub input: InputManager,
    pub time: Time,
    pub ui: UIManager,
    pub ecs: ECSManager<Self>,
    pub globals: GlobalCollection,
    pub io: SaverLoader,
    pub config: GameConfig,
    pub pipeline: PipelineContext,
}
