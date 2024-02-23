/*
use winit::{event::{DeviceEvent, WindowEvent}, event_loop::{EventLoop, ControlFlow, EventLoopWindowTarget}};
/// An event is something that a system can "subscribe" to to execute specific code when something interesting happens
/// Events are passed directly from winit to required systems
pub trait Event<'a>: Sized {}
impl Event<'_> for WindowEvent {}
impl Event<'_> for DeviceEvent {}

/// Initialization event that occurs during initialization
pub struct Init<'a> {
    pub control_flow: &'a mut ControlFlow, 
    pub event_loop_window_target: &'a mut EventLoopWindowTarget<()>,
};
impl<'a> Event<'a> for Init<'a> {}

/// Update event that occurs every frame
pub struct Update<'a> {
    pub control_flow: &'a mut ControlFlow, 
    pub event_loop_window_target: &'a mut EventLoopWindowTarget<()>,
}
impl Event for Update {}

/// Shutdown event that only occurs when the engine is shutting down
pub struct Shutdown<'a> {
    pub control_flow: &'a mut ControlFlow, 
    pub event_loop_window_target: &'a mut EventLoopWindowTarget<()>,
}
impl Event for Shutdown {}

/// Tick event that occurs a specific amount per second
pub struct Tick {
    pub control_flow: &'a mut ControlFlow, 
    pub event_loop_window_target: &'a mut EventLoopWindowTarget<()>,
}
impl Event for Tick {}
*/

use winit::{event_loop::{EventLoop, ControlFlow, EventLoopWindowTarget}};
pub use winit::event::{DeviceEvent, WindowEvent};
/// An event is something that a system can "subscribe" to to execute specific code when something interesting happens
/// Events are passed directly from winit to required systems
pub trait Event: 'static + Sized {}
impl Event for WindowEvent {}
impl Event for DeviceEvent {}

/// Initialization event that occurs during initialization
pub struct Init;
impl Event for Init {}

/// Update event that occurs every frame
pub struct Update;
impl Event for Update {}

/// Shutdown event that only occurs when the engine is shutting down
pub struct Shutdown;
impl Event for Shutdown {}

/// Tick event that occurs a specific amount per second
pub struct Tick;
impl Event for Tick {}