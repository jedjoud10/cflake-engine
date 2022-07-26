use crate::canvas::{ToCanvasAttachment, CanvasAttachment, Canvas};

pub enum PassInput<'a> {
    Main,
    Attachments(&'a [CanvasAttachment])
}

pub enum PassOutput<'a> {
    Main,
    Attachments(&'a [CanvasAttachment])
}

// A pass will simply apply a shader into a canvas -> canvas attachments
pub struct Pass {
    
}

impl Pass {
    // Creat a full-screen pass that will read from input attachments to end attachments
    fn from_attachments(input: PassInput, output: PassOutput) -> Option<Self> {
        let from_main = if let PassInput::Main = input { true } else { false };
        let into_main = if let PassOutput::Main = output { true } else { false };
        if from_main && into_main {
            return None;
        }

        None
    }
}