use crate::prelude::TexelFormat;

// This describes what is the format / type of the attachment
#[derive(Clone, Copy)]
pub enum AttachmentDescription {
    Texture2D {
        name: u32,
    },
}

// A canvas attachment is something that will be boxed and stored within canvases
// Whenever we attach an attachment to a canvas, it will return a handle that we can use to fetch said attachment
pub trait Attachment {
    fn resize(&mut self, size: vek::Extent2<u16>);
    fn description(&self) -> AttachmentDescription;
    fn texel(&self) -> TexelFormat;
}

// This is some sort of handle we use to keep track of attached attachments
pub struct Attached<A: Attachment> {
    framebuffer: u32,
    index: u32,
    desc: AttachmentDescription,
}

impl<A: Attachment> Clone for Attached<A> {
    fn clone(&self) -> Self {
        Self { framebuffer: self.framebuffer.clone(), index: self.index.clone(), desc: self.desc.clone() }
    }
}

impl<A: Attachment> Copy for Attached<A> {}

impl<A: Attachment> Attached<A> {
    // Get the framebuffer of the attached target
    pub fn framebuffer_name(&self) -> usize {
        self.framebuffer
    }

    // Get the index of the attached target
    pub fn index(&self) -> usize {
        self.index
    }
    
    // Get the description of the attached target
    pub fn description(&self) -> AttachmentDescription {
        self.desc
    }
}

// This trait is implemented for every tuple that contains IntoAttachment values
pub trait IntoAttachmentLayout {
    fn into(self) -> Vec<Box<dyn Attachment>>;
}