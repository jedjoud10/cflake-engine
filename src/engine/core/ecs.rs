use crate::engine::{
    core::world::World,
    input::InputManager,
    rendering::{shader::ShaderManager, texture::TextureManager},
};
use std::{any::Any, collections::HashMap, hash::Hash};

use super::world::{EntityManager, Time};

pub mod component;
pub mod entity;
pub mod system;
pub mod system_data;
