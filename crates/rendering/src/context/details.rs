use std::ffi::CStr;

// Get a static string that calls the glGetString function with the input value "symbolic"
// This must be called on the main thread (I think)
pub(super) unsafe fn get_static_str(symbolic: u32) -> &'static str {
    let ptr = gl::GetString(symbolic);
    let str = CStr::from_ptr(ptr as *const i8);
    str.to_str().unwrap()
}