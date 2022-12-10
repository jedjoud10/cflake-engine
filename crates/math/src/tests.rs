#[cfg(test)]
mod shapes {
    #[cfg(test)]
    mod cuboid {
        use crate::{
            Boundable, Cuboid, Movable, SurfaceArea, Volume, AABB,
        };
        use vek::{Extent3, Vec3};

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
            let mut cuboid = Cuboid::cube(Vec3::zero(), 1.0);
            let aabb: AABB = cuboid.bounds();
            assert_eq!(aabb.min, Vec3::broadcast(-0.5f32));
            assert_eq!(aabb.max, Vec3::broadcast(0.5f32));
            cuboid.expand_by(1.0);
            let aabb: AABB = cuboid.bounds();
            assert_eq!(cuboid.extent, Extent3::broadcast(2.0f32));
            assert_eq!(aabb.min, Vec3::broadcast(-1f32));
            assert_eq!(aabb.max, Vec3::broadcast(1f32));

            let mut cuboid = Cuboid::cube(Vec3::zero(), 2.0);
            let aabb: AABB = cuboid.bounds();
            assert_eq!(aabb.min, Vec3::broadcast(-1f32));
            assert_eq!(aabb.max, Vec3::broadcast(1f32));
            cuboid.expand_by(1.0);
            let aabb: AABB = cuboid.bounds();
            assert_eq!(cuboid.extent, Extent3::broadcast(3.0f32));
            assert_eq!(aabb.min, Vec3::broadcast(-1.5f32));
            assert_eq!(aabb.max, Vec3::broadcast(1.5f32));
        }

        #[test]
        fn center() {
            let mut cuboid = Cuboid::cube(Vec3::zero(), 1.0);
            assert_eq!(cuboid.center(), Vec3::zero());

            cuboid.set_center(Vec3::one());

            assert_eq!(cuboid.center(), Vec3::one());
        }
    }

    #[cfg(test)]
    mod sphere {
        use crate::{
            Boundable, Movable, Sphere, SurfaceArea, Volume, AABB,
        };
        use vek::Vec3;

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
        fn center() {
            let mut sphere = Sphere::new(Vec3::zero(), 1.0);
            assert_eq!(sphere.center(), Vec3::zero());

            sphere.set_center(Vec3::one());

            assert_eq!(sphere.center(), Vec3::one());
        }
    }
}
