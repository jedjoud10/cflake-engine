/// A system is executed whenever something interesting happens
/// like an update event, tick event, or window event
/// Systems could be executed in parallel by making use of the "resources" type
/// to handle scheduling automatically
pub trait System<E: Event> {
    type Resources<'w>: 'w;

    /// Execute the system with the appropriate event context
    fn execute(&mut self, resources: &mut Self::Resources<'_>, ctx: E::Context);
    
    /// Handle the order of system execution compared to other systems 
    fn inject(&mut self) -> InjectionOrder;
}

pub struct InjectionOrder;
pub trait Event {
    type Context;
}

