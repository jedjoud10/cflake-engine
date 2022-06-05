use std::{any::{TypeId, Any}, collections::HashMap};
use crate::{Resource, ResRef};

type OptVec<'a> = Vec<Option<&'a mut Box<dyn Resource>>>;
type ResHashMap = HashMap<TypeId, Box<dyn Resource>>;

// A resource reference bundle, like <&mut Assets, &Graphics> or <&AppData, &Config>
pub trait ResBundle<'a>: Sized + 'a {
    // Fetch a list of mutable references to the appropriate boxed resources
    fn fetch_boxed(map: &'a mut ResHashMap) -> Option<OptVec<'a>>;

    // Convert the boxed resources into the resource bundle
    fn cast(validated: OptVec<'a>) -> Self;
}


impl<'a, A: ResRef<'a>> ResBundle<'a> for A {
    fn fetch_boxed(map: &'a mut HashMap<TypeId, Box<dyn Resource>>) -> Option<OptVec<'a>> {
        let types = [&A::id()];
        map.get_many_mut(types).map(|slice| slice.map(Some)).map(Vec::from)
    }

    fn cast(mut validated: OptVec<'a>) -> Self {
        let boxed = validated[0].take().unwrap();
        A::from_mut(boxed)
    }
}

impl<'a, A: ResRef<'a>, B: ResRef<'a>> ResBundle<'a> for (A, B) {
    fn fetch_boxed(map: &'a mut HashMap<TypeId, Box<dyn Resource>>) -> Option<OptVec<'a>> {
        let types = [&A::id(), &B::id()];
        map.get_many_mut(types).map(|slice| slice.map(Some)).map(Vec::from)
    }

    fn cast(mut validated: OptVec<'a>) -> Self {
        let a = validated[0].take().unwrap();
        let b = validated[1].take().unwrap();
        (A::from_mut(a), B::from_mut(b))
    }
}

impl<'a, A: ResRef<'a>, B: ResRef<'a>, C: ResRef<'a>> ResBundle<'a> for (A, B, C) {
    fn fetch_boxed(map: &'a mut HashMap<TypeId, Box<dyn Resource>>) -> Option<OptVec<'a>> {
        let types = [&A::id(), &B::id(), &C::id()];
        map.get_many_mut(types).map(|slice| slice.map(Some)).map(Vec::from)
    }

    fn cast(mut validated: OptVec<'a>) -> Self {
        let a = validated[0].take().unwrap();
        let b = validated[1].take().unwrap();
        let c = validated[2].take().unwrap();
        (A::from_mut(a), B::from_mut(b), C::from_mut(c))
    }
}

impl<'a, A: ResRef<'a>, B: ResRef<'a>, C: ResRef<'a>, D: ResRef<'a>> ResBundle<'a> for (A, B, C, D) {
    fn fetch_boxed(map: &'a mut HashMap<TypeId, Box<dyn Resource>>) -> Option<OptVec<'a>> {
        let types = [&A::id(), &B::id(), &C::id(), &D::id()];
        map.get_many_mut(types).map(|slice| slice.map(Some)).map(Vec::from)
    }

    fn cast(mut validated: OptVec<'a>) -> Self {
        let a = validated[0].take().unwrap();
        let b = validated[1].take().unwrap();
        let c = validated[2].take().unwrap();
        let d = validated[3].take().unwrap();
        (A::from_mut(a), B::from_mut(b), C::from_mut(c), D::from_mut(d))
    }
}