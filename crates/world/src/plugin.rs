/// A plugin allows us to register systems and resources
/// Also allows us to insert multiple systems of the same event type
pub trait Plugin {
    /// Register the plugin's resources and systems
    fn register();
}
