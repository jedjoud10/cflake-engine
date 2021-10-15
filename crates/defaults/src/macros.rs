// Some tasty macros
#[macro_export]
macro_rules! impl_component {
    ($t: ty) => {
        impl t {
            // Wrappers around system data
            fn get_system_data(&self) -> &SystemData {
                &self.system_data
            }
        
            fn get_system_data_mut(&mut self) -> &mut SystemData {
                &mut self.system_data
            }

            // Turn this into "Any" so we can cast into child systems
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

        }
    };
}