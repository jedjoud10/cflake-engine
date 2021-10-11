use crate::console::Console;
use crate::renderer::DebugRenderer;

// Debug srtuct
#[derive(Default)]
pub struct MainDebug {
    pub console: Console,
    pub renderer: DebugRenderer,
}
