#[cfg(test)]
mod tests {
    use std::num::{NonZeroU64, NonZeroU8};

    use crate::{
        bounds::aabb::AABB,
        octrees::{HeuristicSettings, Octree},
        shapes::Sphere,
    };

    // AABB sphere
    #[test]
    pub fn aabb_sphere() {
        let sphere = Sphere {
            center: vek::Vec3::unit_x() * 2.0,
            radius: 1.0,
        };
        let aabb = AABB {
            min: -vek::Vec3::one(),
            max: vek::Vec3::one(),
        };
        assert!(!crate::intersection::aabb_sphere(&aabb, &sphere));
        let sphere = Sphere {
            center: vek::Vec3::unit_x() * 1.9,
            radius: 1.0,
        };
        let aabb = AABB {
            min: -vek::Vec3::one(),
            max: vek::Vec3::one(),
        };
        assert!(crate::intersection::aabb_sphere(&aabb, &sphere));
        let sphere = Sphere {
            center: vek::Vec3::one() * 19.0,
            radius: 1.0,
        };
        let aabb = AABB {
            min: -vek::Vec3::one() * 20.0,
            max: vek::Vec3::one() * 20.0,
        };
        assert!(crate::intersection::aabb_sphere(&aabb, &sphere));
    }
    // Octree
    #[test]
    pub fn octree() {
        let mut octree = Octree::new(NonZeroU8::new(5).unwrap(), NonZeroU64::new(32).unwrap(), HeuristicSettings::default());
        octree.update(vek::Vec3::zero());
        assert_eq!(octree.nodes().len(), 33);
    }
}
