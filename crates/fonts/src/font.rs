use crate::FontChar;
use ::rendering::{basics::*, pipec, pipeline::*};
use ascii::AsciiStr;
use assets::{Asset, Object};
use byteorder::{LittleEndian, ReadBytesExt};

// A simple font containing the characters
#[derive(Default)]
pub struct Font {
    pub name: String,
    pub atlas_dimensions: veclib::Vector2<u16>,
    pub texture_pixels: Vec<u8>,
    pub texture: GPUObjectID,
    pub chars: Vec<FontChar>,
    pub font_options: FontOptions,
}

// Font options
pub struct FontOptions {
    pub thickness: f32,
    pub outline_thickness: f32,
    pub color: veclib::Vector4<f32>,
    pub outline_color: veclib::Vector4<f32>,
}

// Default font options
impl Default for FontOptions {
    fn default() -> Self {
        Self {
            thickness: 0.2,
            outline_thickness: 0.3,
            color: veclib::Vector4::ZERO,
            outline_color: veclib::Vector4::ONE,
        }
    }
}

impl Font {
    // Get a specific character from this font using it's ASCII code
    pub fn get_char(&self, ascii_code: u8) -> &FontChar {
        // The offset of the first ASCII character in the font
        const ASCII_FIRST_CHAR_OFFSET: u8 = 32;
        let char = self
            .chars
            .get((ascii_code - ASCII_FIRST_CHAR_OFFSET) as usize)
            .expect(format!("Couldn't get character {}", &ascii_code).as_str());
        return char;
    }
    // Create the actual texture from the raw pixel bitmap data we have
    pub fn create_texture(&mut self) {
        // Create the texture and set it's parameters
        self.texture = pipec::texture(
            Texture::default()
                .set_dimensions(TextureType::Texture2D(self.atlas_dimensions.x, self.atlas_dimensions.y))
                .set_filter(TextureFilter::Linear)
                .set_format(TextureFormat::R16R)
                .set_bytes(self.texture_pixels.clone()),
        );
    }
    // Turn some text into an array of font chars
    pub fn convert_text_to_font_chars(&self, text: &str) -> Vec<&FontChar> {
        let ascii_str = AsciiStr::from_ascii(text).unwrap();
        let chars = ascii_str.as_bytes();
        let font_chars = chars.iter().map(|&x| self.get_char(x)).collect::<Vec<&FontChar>>();
        return font_chars;
    }
    // Set the font parameters for this text
    pub fn set_font_parameter(&self) {}
}

// The font is loadable
impl Asset for Font {
    fn load_medadata(self, data: &assets::AssetMetadata) -> Option<Self>
    where
        Self: Sized,
    {
        // Load this font from the metadata bytes
        let mut reader = std::io::Cursor::new(data.bytes.clone());
        // Read the custom font
        let mut output_font = Font::default();
        output_font.name = data.name.clone();
        // Get the width and height of the bitmap
        let width = reader.read_u16::<LittleEndian>().unwrap();
        let height = reader.read_u16::<LittleEndian>().unwrap();
        output_font.atlas_dimensions = veclib::Vector2::new(width, height);
        let pixel_num: u32 = width as u32 * height as u32;
        // Read the pixels, one by one
        for i in 0..pixel_num {
            let pixel = reader.read_u8().unwrap();
            output_font.texture_pixels.push(pixel);
        }

        // Get the number of ASCII characters we have
        let font_char_num = reader.read_u8().unwrap();

        // Read the chars
        for i in 0..font_char_num {
            // Get the data back
            let loaded_char = FontChar {
                id: reader.read_u8().unwrap(),
                min: veclib::Vector2::new(reader.read_u16::<LittleEndian>().unwrap(), reader.read_u16::<LittleEndian>().unwrap()),
                max: veclib::Vector2::new(reader.read_u16::<LittleEndian>().unwrap(), reader.read_u16::<LittleEndian>().unwrap()),
            };
            output_font.chars.push(loaded_char);
        }
        output_font.create_texture();
        Some(output_font)
    }
}
impl Object for Font {}
