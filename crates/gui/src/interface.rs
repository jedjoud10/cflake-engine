use std::ops::Deref;

use crate::Rasterizer;

// This interface encapsulates all the data that we need to use Egui and to draw
pub struct Interface {
    // This is the egui context that will handle everything related to egui
    pub(crate) egui: egui::Context,

    // This is the current egui state given from winit whenever we receive a new window event
    pub(crate) state: egui_winit::State,
    
    // This is the rasterizer that will actually draw stuff onto the screen
    pub(crate) rasterizer: Rasterizer,

    // Tells egui if it's currently taking window events or not
    pub enabled: bool,
}

impl Deref for Interface {
    type Target = egui::Context;

    fn deref(&self) -> &Self::Target {
        &self.egui
    }
}