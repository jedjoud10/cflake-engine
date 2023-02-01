use crate::vulkan::vk;

use crate::Texel;

// What we should do when loading in data from the attachment
pub enum LoadOp<T: Texel> {
    Ignore,
    Load,
    Clear(T::Storage),
}

// Untyped load operation that uses a funky union
// Only used internally when untyping the typed attachments
pub(crate) enum UntypedLoadOp {
    Ignore,
    Load,
    Clear(vk::ClearColorValue),
}

// What we should do when writing data to the attachment
pub enum StoreOp {
    Ignore,
    Store,
}
