use ecs::Entity;
use rapier3d::prelude::QueryFilter;

use crate::{Physics, util};

// Data containing what a ray hit
#[derive(Clone, Copy)]
pub struct RayHit {
    entity: Entity,
    point: vek::Vec3<f32>,
    normal: vek::Vec3<f32>,
    toi: f32,
}

impl RayHit {
    // Get the entity whose collider was hit 
    pub fn entity(&self) -> Entity {
        self.entity
    }

    // Get the point that was hit
    pub fn point(&self) -> vek::Vec3<f32> {
        self.point
    }

    // Get the normal that was hit
    pub fn normal(&self) -> vek::Vec3<f32> {
        self.normal
    }

    // Get the Time of Impact (distance) that was travelled
    pub fn toi(&self) -> f32 {
        self.toi
    }
}

// Query checks for the main resource
impl Physics {
    // Cast a ray into the scene and check if it hit anything or not
    // Returns None if the ray hit nothing
    pub fn cast_ray(
        &self,
        origin: vek::Vec3<f32>,
        direction: vek::Vec3<f32>,
        max_toi: f32,
        solid: bool,
    ) -> Option<RayHit> {
        let ray = rapier3d::geometry::Ray {
            origin: util::vek_vec_to_na_point(origin),
            dir: util::vek_vec_to_na_vec(direction.normalized()),
        };

        let hit = self.query.cast_ray_and_get_normal(
            &self.bodies,
            &self.colliders,
            &ray,
            max_toi,
            solid,
            QueryFilter::new()
        );

        hit.map(|(collider, intersection)| {
            let collider = self.colliders.get(collider).unwrap();
            let entity = Entity::from_raw((collider.user_data & u64::MAX as u128) as u64);

            RayHit {
                entity,
                normal: util::na_vec_to_vek_vec(intersection.normal),
                point: intersection.toi * direction + origin,
                toi: intersection.toi,
            }
        })
    }
}