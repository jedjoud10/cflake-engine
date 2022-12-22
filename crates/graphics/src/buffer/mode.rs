
// Some settings that tell us how exactly we should create the buffer
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Dynamic buffers are like static buffers, but they allow the user to mutate each element
    #[default]
    Dynamic,

    // Partial buffer have a fixed capacity, but a dynamic length
    Parital,
}