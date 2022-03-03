use slotmap::SlotMap;

// A simple builder trait
pub trait Builder {
    // Build
    type Element;
    type Key;
    fn build(self, slotmap: &mut SlotMap<Self::Key, Self::Element>) -> Self::Key;
}