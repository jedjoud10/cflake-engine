pub enum DisplayTarget<'a> {
    Texture2DLayer {
        name: u32,
        size: vek::Extent2<u32>
    },
}

pub trait ToDisplayTarget<'a> {
    fn into(&self) -> DisplayTarget<'a>; 
}

pub struct DisplayLayout<'a> {

}

pub struct Display<L: DisplayLayout> {

}