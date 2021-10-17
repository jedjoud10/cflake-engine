#[macro_export]
macro_rules! impl_custom_system_data {
    ($t: ty) => {
        impl InternalSystemData for $t {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}
