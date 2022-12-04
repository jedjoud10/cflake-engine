#[cfg(test)]
mod shapes {
    #[cfg(test)]
    mod cuboid {
        use vek::Vec3;
        use crate::{Cuboid, Volume, SurfaceArea, AABB, Boundable};

        #[test]
        fn volume() {
            let cuboid = Cuboid::cube(Vec3::zero(), 1.0);
            assert_eq!(cuboid.volume(), 1.0f32);
            let cuboid = Cuboid::cube(Vec3::zero(), 2.0);
            assert_eq!(cuboid.volume(), 8.0f32);
        }
        
        #[test]
        fn surface_area() {
            let cuboid = Cuboid::cube(Vec3::zero(), 1.0);
            assert_eq!(cuboid.surface_area(), 6.0f32);
            let cuboid = Cuboid::cube(Vec3::zero(), 2.0);
            assert_eq!(cuboid.surface_area(), 24.0f32);
        }

        #[test]
        fn bounds() {
            let cuboid = Cuboid::cube(Vec3::zero(), 1.0);
            let aabb: AABB = cuboid.bounds();
            assert_eq!(aabb.min, Vec3::broadcast(-0.5f32));
            assert_eq!(aabb.max, Vec3::broadcast(0.5f32));

            let cuboid = Cuboid::cube(Vec3::zero(), 2.0);
            let aabb: AABB = cuboid.bounds();
            assert_eq!(aabb.min, Vec3::broadcast(-1f32));
            assert_eq!(aabb.max, Vec3::broadcast(1f32));
        }

        #[test]
        fn center() {}
    }

    #[cfg(test)]
    mod sphere {
        use vek::Vec3;
        use crate::{Sphere, Volume, SurfaceArea, AABB, Boundable};

        #[test]
        fn volume() {
            let sphere = Sphere::new(Vec3::zero(), 1.0);
            assert_eq!(sphere.volume(), 4.188790205f32);
            let sphere = Sphere::new(Vec3::zero(), 2.0);
            assert_eq!(sphere.volume(), 33.51032164f32);
        }
        
        #[test]
        fn surface_area() {
            let sphere = Sphere::new(Vec3::zero(), 1.0);
            assert_eq!(sphere.surface_area(), 12.56637061);
            let sphere = Sphere::new(Vec3::zero(), 2.0);
            assert_eq!(sphere.surface_area(), 50.26548246f32);
        }

        #[test]
        fn bounds() {
            let sphere = Sphere::new(Vec3::zero(), 1.0);
            let aabb: AABB = sphere.bounds();
            assert_eq!(aabb.min, Vec3::broadcast(-1f32));
            assert_eq!(aabb.max, Vec3::broadcast(1f32));

            let sphere = Sphere::new(Vec3::zero(), 2.0);
            let aabb: AABB = sphere.bounds();
            assert_eq!(aabb.min, Vec3::broadcast(-2f32));
            assert_eq!(aabb.max, Vec3::broadcast(2f32));
        }

        #[test]
        fn center() {}
    }
}