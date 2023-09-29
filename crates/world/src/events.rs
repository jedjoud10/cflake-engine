use winit::event::{DeviceEvent, WindowEvent};
/// An event is something that a system can "subscribe" to to execute specific code when something interesting happens
pub trait Event: 'static + Sized + Sync + Send {}
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
