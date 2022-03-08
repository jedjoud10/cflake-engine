#[cfg(test)]
mod tests {
    use crate::{
        bounds::aabb::AABB,
        octrees::{HeuristicSettings, Octree},
        shapes::Sphere,
    };

    // AABB sphere
    #[test]
    pub fn aabb_sphere() {
        let sphere = Sphere {
            center: veclib::Vector3::X * 2.0,
            radius: 1.0,
        };
        let aabb = AABB {
            min: -veclib::Vector3::ONE,
            max: veclib::Vector3::ONE,
        };
        assert!(!crate::intersection::aabb_sphere(&aabb, &sphere));
        let sphere = Sphere {
            center: veclib::Vector3::X * 1.9,
            radius: 1.0,
        };
        let aabb = AABB {
            min: -veclib::Vector3::ONE,
            max: veclib::Vector3::ONE,
        };
        assert!(crate::intersection::aabb_sphere(&aabb, &sphere));
        let sphere = Sphere {
            center: veclib::Vector3::ONE * 19.0,
            radius: 1.0,
        };
        let aabb = AABB {
            min: -veclib::Vector3::ONE * 20.0,
            max: veclib::Vector3::ONE * 20.0,
        };
        assert!(crate::intersection::aabb_sphere(&aabb, &sphere));
    }
    // Octree
    #[test]
    pub fn octree() {
        let mut octree = Octree::new(5, 32, HeuristicSettings::default());
        octree.update(veclib::Vector3::ZERO);
        assert_eq!(octree.nodes().len(), 33);
    }
}
