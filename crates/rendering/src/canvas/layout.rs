pub trait AttachmentLayout {}

pub type SceneCanvasLayout = (i32);
pub type DefaultCanvasLayout = ();


impl AttachmentLayout for SceneCanvasLayout {

} 

impl AttachmentLayout for DefaultCanvasLayout {

} 