use std::{any::{TypeId, Any}, collections::HashMap};

use crate::{Resource, ResRef};

// A resource reference bundle, like <&mut Assets, &Graphics> or <&AppData, &Config>
pub trait ResBundle<'a>: Sized {
    // The raw type tuple that contains the types of the resources. Ex: (Assets, Config)
    type Raw: 'static;

    // Get the type Ids that we must fetch from the map
    fn types() -> &'static [TypeId];

    // Get the resource bundle from the map directly
    fn fetch(map: &mut HashMap<TypeId, Box<dyn Resource>>) -> Option<Self>;
}


impl<'a, A: ResRef<'a>> ResBundle<'a> for A {
    type Raw = A::Inner;
    
    fn types() -> &'static [TypeId] {
        &[]
    }

    fn fetch(map: &mut HashMap<TypeId, Box<dyn Resource>>) -> Option<Self> {
        let output: Option<(A)> = map.get_many_mut([&A::id()]).map(|slice| {
            // Convert the slice of boxed resources into the raw resource references
            //let a = slice[0];
            todo!()
        });
        None
    }

}