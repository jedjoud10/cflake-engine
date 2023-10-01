use crate::prelude::*;

// Simple empty system that WILL execute in parallel
// This will also execute BEFORE SampleSystemB
struct SampleSystemA;

impl System<Init> for SampleSystemA {
    fn execute(&mut self, view: &mut WorldView) {
        let resource = view.get_mut::<u32>();
    }

    fn inject(&mut self) -> InjectionOrder<Init> {
        InjectionOrder::default()
            .before::<SampleSystemB>()
    }
}

impl ParSystem<Init> for SampleSystemA {
    fn scheduling(&mut self) -> SystemScheduling {
        SystemScheduling::default()
    }
}

// Simple empty system that will NOT execute in parallel
struct SampleSystemB;

impl System<Init> for SampleSystemB {
    fn execute(&mut self, view: &mut WorldView) {
        let resource = view.get_mut::<u32>();
    }
}
