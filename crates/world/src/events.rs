use winit::event::{WindowEvent, DeviceEvent};
/// An event is something that a system can "subscribe" to to execute specific code when something interesting happens
pub trait Event: 'static + Sized {}
impl Event for WindowEvent {}
impl Event for DeviceEvent {}

pub struct Init(());
impl Event for Init {}
pub struct Update(());
impl Event for Update {}
pub struct Shutdown(());
impl Event for Shutdown {}
pub struct Tick(());
impl Event for Tick {}
