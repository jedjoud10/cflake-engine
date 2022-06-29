use std::ops::{Bound, RangeBounds, RangeFull};

// Name key type
type Key = &'static str;

// A restriction is some sort of rule that depicts the arrangement and the location of the stages
enum Restriction {
    BeforeThan(Key),
    AfterThan(Key)
}

pub struct Stage {
    // The user defined name of the current stage
    name: Key,

    // Restrictions that apply to this stage
    restrictions: Vec<Restriction>,
}

impl Stage {
    // Create a stage that will execute anytime, since it has no restrictions
    pub fn new(name: impl Into<Key>) -> Self {
        Self { name: name.into(), restrictions: Default::default() }
    }

    // Create a stage that must execute before another one
    pub fn before_than(name: impl Into<Key>, other: &Stage) -> Self {
        Self { name: name.into(), restrictions: vec![Restriction::BeforeThan(other.name)] }
    }

    // Create a stage that must execute after another one
    pub fn after_than(name: impl Into<Key>, other: &Stage) -> Self {
        Self { name: name.into(), restrictions: vec![Restriction::AfterThan(other.name)] }
    }
}

fn eval(vec: Vec<Stage>) {

}

#[test]
fn test() {
    let main = Stage::new("main");
    let after = Stage::after_than("after", &main);
    let before = Stage::before_than("before", &main);
    eval()
}
