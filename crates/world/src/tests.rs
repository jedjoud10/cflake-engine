use crate::prelude::*;

struct BasicSystem;
impl System<Init> for BasicSystem {
    type Resources<'w> = ();

    fn execute(&mut self, resources: &mut Self::Resources<'_>) {
        todo!()
    }

    fn inject(&mut self) -> InjectionOrder {
        todo!()
    }
}

impl System<Update> for BasicSystem {
    type Resources<'w> = ();

    fn execute(&mut self, resources: &mut Self::Resources<'_>) {
        todo!()
    }

    fn inject(&mut self) -> InjectionOrder {
        todo!()
    }
}

#[test]
fn test() {
}