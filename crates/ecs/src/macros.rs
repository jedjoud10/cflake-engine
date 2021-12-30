// Macro to automatically implement the traits
#[macro_export]
macro_rules! impl_component {
    ($t: ty) => {
        use $crate::Component;
        // Main traits implemented
        impl Component for $t {
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
#[macro_export]
macro_rules! impl_systemdata {
    ($t:ty) => {
        use $crate::CustomSystemData;
        impl CustomSystemData for $t {}
    };
}

impl_systemdata!(());
