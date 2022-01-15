// Macro to automatically implement the traits
#[macro_export(super)]
macro_rules! impl_component {
    ($t: ty) => {
        // Main traits implemented
        impl $crate::component::Component for $t {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            fn get_component_name() -> String {
                String::from(stringify!($t).split(" ").last().unwrap().to_string())
            }
        }
    };
}