use crate::prelude::*;

/*
// Simple empty system that WILL execute in parallel
// This will also execute BEFORE SampleSystemB
fn init(world: &mut World) {

}

impl Plugin {
    fn register(systems: &mut Systems) {
        systems.insert(init);
    }
}

// Simple empty system that will NOT execute in parallel
struct SampleSystemB;

impl System<Init> for SampleSystemB {
    fn execute(&mut self, view: &mut WorldView) {
        let resource = view.get_mut::<u32>();
    }
}
*/