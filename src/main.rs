use std::{iter::repeat, hint::black_box};

use app::prelude::*;

#[derive(Default, Component, Clone)]
struct Position(u32);

#[derive(Default, Component, Clone)]
struct Rotation(u32);

#[derive(Default, Component, Clone)]
struct Matrix(u32);

fn main() {
    let mut scene = Scene::default();
    scene.extend((0..100000).into_iter().map(|i| (Position(i), Rotation(i), Matrix(i))));
    
    for (a, b, c) in scene.query_mut::<(&mut Position, &Matrix, &Rotation)>().into_iter() {
        a.0 = b.0 + c.0; 
    }
}
