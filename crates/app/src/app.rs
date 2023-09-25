use std::sync::mpsc;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// An app that can be built that will use cFlake engine.
/// It uses the builder pattern to set settings and to register custom events
pub struct App {
}

impl Default for App {
    fn default() -> Self {
        Self {}
    }
}

impl App {
    /// Consume the App builder, and start the engine.
    pub fn execute(mut self) {
        let (sender, receiver) = mpsc::channel::<String>();
        crate::logger::init_logger(log::LevelFilter::Debug, sender);
    }
}
