// Archetype layout
#[derive(Clone, Copy, Debug)]
pub struct ComponentLayout<'a> {
    // Combined bitmask
    pub(crate) mask: u64,

    // Separate bits
    pub(crate) bits: &'a [u64],
}
impl<'a> ComponentLayout<'a> {
    // Create a new archetype layout using some component bitfields
    pub fn new(bits: &'a [u64]) -> Self {
        Self {
            mask: bits.iter().fold(0, |a, b| a | *b),
            bits: bits,
        }
    }
}
