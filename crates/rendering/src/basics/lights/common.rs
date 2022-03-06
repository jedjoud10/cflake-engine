use crate::{object::PipelineCollectionElement, pipeline::Pipeline};
use enum_as_inner::EnumAsInner;

// A light type
#[derive(EnumAsInner)]
pub enum LightSourceType {
    // TODO: Aeaea
    Directional,
    Point,
    Area,
}

// Light transform
pub struct StoredLightTransform {
    pub position: veclib::Vector3<f32>,
    pub rotation: veclib::Quaternion<f32>,
}

// Light source that we will render
pub struct StoredLight {
    pub _type: LightSourceType,
    pub transform: StoredLightTransform,
    pub strength: f32,
    pub color: veclib::Vector3<f32>,
}

impl PipelineCollectionElement for StoredLight {
    fn added(&mut self, handle: &crate::pipeline::Handle<Self>) {}

    fn disposed(self) {}
}
