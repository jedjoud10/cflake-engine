use crate::FontChar;
use ascii::AsciiStr;
use assets::Asset;
use byteorder::{LittleEndian, ReadBytesExt};
use rendering::{
    basics::texture::{Texture, TextureFilter, TextureFormat, TextureType},
    object::ObjectID,
    pipeline::{pipec, Pipeline},
};

// A simple font containing the characters
#[derive(Default)]
pub struct Font {
    pub name: String,
    pub atlas_dimensions: veclib::Vector2<u16>,
    pub texture_pixels: Vec<u8>,
    pub texture: ObjectID<Texture>,
    pub chars: Vec<FontChar>,
    pub font_options: FontOptions,
}

// Font options that we use when rendering a specific font
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
            thickness: 1.0,
            outline_thickness: 3.0,
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
            .unwrap_or_else(|| panic!("Couldn't get character {}", &ascii_code));
        char
    }
    // Create the actual texture from the raw pixel bitmap data we have
    pub fn create_texture(&mut self, pipeline: &Pipeline) {
        // Create the texture and set it's parameters
        self.texture = pipec::construct(
            Texture::default()
                .set_dimensions(TextureType::Texture2D(self.atlas_dimensions.x, self.atlas_dimensions.y))
                .set_filter(TextureFilter::Linear)
                .set_format(TextureFormat::R16R)
                .set_bytes(self.texture_pixels.clone()),
            pipeline,
        );
    }
    // Turn some text into an array of font chars
    pub fn convert_text_to_font_chars(&self, text: &str) -> Vec<&FontChar> {
        let ascii_str = AsciiStr::from_ascii(text).unwrap();
        let chars = ascii_str.as_bytes();
        let font_chars = chars.iter().map(|&x| self.get_char(x)).collect::<Vec<&FontChar>>();
        font_chars
    }
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
        for _i in 0..pixel_num {
            let pixel = reader.read_u8().unwrap();
            output_font.texture_pixels.push(pixel);
        }

        // Get the number of ASCII characters we have
        let font_char_num = reader.read_u8().unwrap();

        // Read the chars
        for _i in 0..font_char_num {
            // Get the data back
            let loaded_char = FontChar {
                id: reader.read_u8().unwrap(),
                min: veclib::Vector2::new(reader.read_u16::<LittleEndian>().unwrap(), reader.read_u16::<LittleEndian>().unwrap()),
                max: veclib::Vector2::new(reader.read_u16::<LittleEndian>().unwrap(), reader.read_u16::<LittleEndian>().unwrap()),
            };
            output_font.chars.push(loaded_char);
        }
        Some(output_font)
    }
}
