// A font character
pub struct FontChar {
    pub id: u8,
    pub min: veclib::Vector2<u16>,
    pub max: veclib::Vector2<u16>,
}

impl FontChar {
    // Get the width of the font char
    pub fn get_width(&self) -> u16 {
        return self.max.x - self.min.x;
    }
    // Get teh height of the font char
    pub fn get_height(&self) -> u16 {
        return self.max.y - self.min.y;
    }
}
