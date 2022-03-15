use std::fs::OpenOptions;
use std::io::BufWriter;

use image::{DynamicImage, RgbImage, ImageFormat};
use time::{OffsetDateTime};
use world::ecs::component::ComponentQuerySet;
use world::input::Keys;
use world::World;

// The screeenshot system's update loop
fn run(world: &mut World, _data: ComponentQuerySet) {
    if world.input.pressed("take_screenshot") {
        // Take a screenshot
        let dimensions = world.pipeline.window().dimensions();
        let bytes = world.renderer.screenshot(dimensions);

        // Get the current time
        let time = OffsetDateTime::now_local().unwrap();
        let format = time::format_description::parse(
            "y[year]-m[month]-d[day]-h[hour]-m[minute]-s[second]",
        ).unwrap();
        let formatted = time.format(&format).unwrap();

        // And use it to format the name of the screenshot
        let name = format!("screenshots/{}.jpeg", formatted);

        // And then store it
        world.io.create_file(&name);
        let mut options = OpenOptions::new();
        options.write(true).truncate(true);
        
        // Write to a dynamic image
        let image = DynamicImage::ImageRgb8(RgbImage::from_vec(dimensions.w as u32, dimensions.h as u32, bytes).unwrap());
        let image = image.flipv();

        let file = world.io.open_file(&name, &options).unwrap();
        let mut writer = BufWriter::new(file);
        image.write_to(&mut writer, ImageFormat::Png).unwrap();
    }
}

// Create a system that'll allow us to screenshot the current frame
pub fn system(world: &mut World) {
    world.ecs.systems.builder().event(run).build();
    world.input.bind(Keys::F1, "take_screenshot");
}
