#[cfg(test)]
mod tests {
    use crate::{bounds::AABB, Intersection};

    // Intersection tests
    #[test]
    pub fn intersection() {
        // Create some AABBs
        let aabb1 = AABB {
            min: -veclib::Vector3::ONE,
            max: veclib::Vector3::ONE,
            ..AABB::default()
        };
        let aabb2 = AABB {
            min: veclib::Vector3::new(-20.0, 0.0, 0.0),
            max: veclib::Vector3::new(-10.0, 10.0, 10.0),
            ..AABB::default()
        };
        assert_eq!(Intersection::aabb_aabb(&aabb1, &AABB::ndc_forward()), true);
        assert_eq!(Intersection::aabb_aabb(&aabb2, &AABB::ndc_forward()), false);
    }
}
