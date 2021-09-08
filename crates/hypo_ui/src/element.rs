// A simple element, could be a button or a panel or anything, it just has some variables
pub struct Element {
    pub size: veclib::Vector2<f32>,
    pub position: veclib::Vector2<f32>,
    pub parent: Option<Element>,    
}