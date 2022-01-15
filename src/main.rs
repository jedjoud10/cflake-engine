use main::core::World;


fn main() {
    // Load up the engine
    main::start("DevJed", "DevGame", preload_assets, init);
}
pub fn preload_assets() {
    // -----Pre-load the game assets here-----
}
pub fn init(world: main::core::WriteContext) {
    // ----Start the world----
}
