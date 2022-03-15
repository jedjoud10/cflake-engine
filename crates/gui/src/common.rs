use egui::{Color32, ImageData, TextureId};

// Get the u64 ID from a TextureID
pub fn get_id(tid: TextureId) -> u64 {
    match tid {
        egui::TextureId::Managed(id) => id,
        egui::TextureId::User(_) => todo!(),
    }
}

// Get the dimensions of an image
pub fn get_dimensions(image: &ImageData) -> vek::Extent2<u16> {
    vek::Extent2::new(image.width(), image.height()).as_()
}

// Convert an image to raw bytes (rgba)
pub fn convert_image(image: ImageData) -> Vec<u8> {
    match image {
        ImageData::Color(image) => convert_color32(image.pixels),
        ImageData::Alpha(image) => convert_alpha(image.pixels),
    }
}

// Convert Color32 pixels to raw bytes (rgba)
pub fn convert_color32(pixels: Vec<Color32>) -> Vec<u8> {
    // I hate this
    let mut bytes = Vec::<u8>::with_capacity(pixels.len() * 4);
    for color in pixels.iter() {
        bytes.push(color.r());
        bytes.push(color.g());
        bytes.push(color.b());
        bytes.push(color.a());
    }
    bytes
}
// Convert Alpha pixels to raw bytes (rgba)
pub fn convert_alpha(pixels: Vec<u8>) -> Vec<u8> {
    // I hate this
    let mut bytes = Vec::<u8>::with_capacity(pixels.len() * 4);
    for alpha in pixels.into_iter() {
        let color = Color32::from_white_alpha(alpha);
        bytes.push(color.r());
        bytes.push(color.g());
        bytes.push(color.b());
        bytes.push(color.a());
    }
    bytes
}
