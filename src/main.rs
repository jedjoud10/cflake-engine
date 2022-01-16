use window::start;
fn main() {
    // Load up the engine
    start("DevJed", "DevGame", preload_assets, init);
}
pub fn preload_assets() {
    // -----Pre-load the game assets here-----
}
pub fn init(world: window::core::WriteContext) {
    // ----Start the world----
}
