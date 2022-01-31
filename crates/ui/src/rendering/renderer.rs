use crate::UIRenderingBuffer;

// The renderer that will render each element using instancing
pub struct UIRenderer {
    // We contain the OpenGL buffer data
    buffer: UIRenderingBuffer
}