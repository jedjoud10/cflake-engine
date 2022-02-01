// A polygon 2D point
pub struct Polygon2DPoint {
    pub center: veclib::Vector2<f32>,
}

// A polygon 2D shape
pub struct Polygon2D {
    pub points: Vec<Polygon2DPoint>,
}
