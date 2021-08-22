use crate::engine::core::defaults::components::transforms;
use crate::engine::core::ecs::component::{Component, ComponentID, ComponentInternal, ComponentManager};
use crate::engine::core::ecs::entity::Entity;
use crate::engine::math::{self, bounds};

use crate::engine::rendering::renderer::Renderer;
use crate::engine::rendering::window::Window;
use glam::Vec4Swizzles;