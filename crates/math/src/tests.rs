#[cfg(test)]
mod shapes {
    #[cfg(test)]
    mod cuboid {
        use crate::{Aabb, Boundable, Cuboid, Movable, SurfaceArea, Volume};
        use vek::{Extent3, Vec3};

        #[test]
        fn volume() {
            let cuboid = Cuboid::cube(Vec3::zero(), 0.5f32);
            assert_eq!(cuboid.volume(), 1.0f32);
            let cuboid = Cuboid::cube(Vec3::zero(), 1.0f32);
            assert_eq!(cuboid.volume(), 8.0f32);
        }

        #[test]
        fn surface_area() {
            let cuboid = Cuboid::cube(Vec3::zero(), 0.5f32);
            assert_eq!(cuboid.area(), 6.0f32);
            let cuboid = Cuboid::cube(Vec3::zero(), 1.0f32);
            assert_eq!(cuboid.area(), 24.0f32);
        }

        #[test]
        fn bounds() {
            let mut cuboid = Cuboid::cube(Vec3::zero(), 0.5);
            let aabb: Aabb<f32> = cuboid.bounds();
            assert_eq!(aabb.min, Vec3::broadcast(-0.5f32));
            assert_eq!(aabb.max, Vec3::broadcast(0.5f32));
            cuboid.expand_by(1.0);
            let aabb: Aabb<f32> = cuboid.bounds();
            assert_eq!(cuboid.half_extent, Extent3::broadcast(2.0f32));
            assert_eq!(aabb.min, Vec3::broadcast(-1f32));
            assert_eq!(aabb.max, Vec3::broadcast(1f32));

            let mut cuboid = Cuboid::cube(Vec3::zero(), 1.0);
            let aabb: Aabb<f32> = cuboid.bounds();
            assert_eq!(aabb.min, Vec3::broadcast(-1f32));
            assert_eq!(aabb.max, Vec3::broadcast(1f32));
            cuboid.expand_by(1.0);
            let aabb: Aabb<f32> = cuboid.bounds();
            assert_eq!(cuboid.half_extent, Extent3::broadcast(3.0f32));
            assert_eq!(aabb.min, Vec3::broadcast(-1.5f32));
            assert_eq!(aabb.max, Vec3::broadcast(1.5f32));
        }

        #[test]
        fn center() {
            let mut cuboid = Cuboid::cube(Vec3::zero(), 0.5);
            assert_eq!(cuboid.center(), Vec3::zero());

            cuboid.set_center(Vec3::one());

            assert_eq!(cuboid.center(), Vec3::one());
        }
    }

    #[cfg(test)]
    mod sphere {
        use crate::{Aabb, Boundable, Movable, Sphere, SurfaceArea, Volume};
        use vek::Vec3;

        #[test]
        fn volume() {
            let sphere = Sphere::new(Vec3::zero(), 1.0f32);
            assert_eq!(sphere.volume(), 4.188_790_3_f32);
            let sphere = Sphere::new(Vec3::zero(), 2.0f32);
            assert_eq!(sphere.volume(), 33.510_323_f32);
        }

        #[test]
        fn surface_area() {
            let sphere = Sphere::new(Vec3::zero(), 1.0f32);
            assert_eq!(sphere.area(), 12.566_371);
            let sphere = Sphere::new(Vec3::zero(), 2.0f32);
            assert_eq!(sphere.area(), 50.265_484_f32);
        }

        #[test]
        fn bounds() {
            let sphere = Sphere::new(Vec3::zero(), 1.0);
            let aabb: Aabb<f32> = sphere.bounds();
            assert_eq!(aabb.min, Vec3::broadcast(-1f32));
            assert_eq!(aabb.max, Vec3::broadcast(1f32));

            let sphere = Sphere::new(Vec3::zero(), 2.0);
            let aabb: Aabb<f32> = sphere.bounds();
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

    #[cfg(test)]
    mod intersect {
        use crate::Aabb;

        #[test]
        fn point_aabb() {
            let aabb = Aabb {
                min: -vek::Vec3::<f32>::one(),
                max: vek::Vec3::<f32>::one(),
            };
            assert!(crate::point_aabb(&vek::Vec3::<f32>::zero(), &aabb));
            assert!(crate::point_aabb(&vek::Vec3::<f32>::one(), &aabb));
            assert!(crate::point_aabb(&-vek::Vec3::<f32>::one(), &aabb));
        }
    }
}