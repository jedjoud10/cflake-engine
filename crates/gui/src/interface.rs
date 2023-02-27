use std::ops::Deref;

// This interface encapsulates all the data that we need to use Egui and to draw
pub struct Interface {
    // This is the egui context that will handle everything related to egui
    pub(crate) egui: egui::Context,

    // This is the current egui state given from winit whenever we receive a new window event
    pub(crate) state: egui_winit::State,
    
    /*
    // This is the painter that will actually draw stuff onto the screen
    painter: Painter,
    */
}

impl Deref for Interface {
    type Target = egui::Context;

    fn deref(&self) -> &Self::Target {
        &self.egui
    }
}