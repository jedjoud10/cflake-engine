use std::{ffi::c_void, ptr::null};

// Some utils
pub struct Utils {}

impl Utils {
    // Constantly check for errors
    pub fn start_error_check_loop() {
        unsafe {
            println!("START OPENGL ERROR CHECK LOOP");
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(opengl_error_callback), null());
        }
    }
}

extern "system" fn opengl_error_callback(_source: u32, _type: u32, _id: u32, severity: u32, _length: i32, message: *const i8, _userParam: *mut c_void) {
    // Check if it was really an error
    if _type == gl::DEBUG_TYPE_ERROR {
        println!("We caught an OpenGL error!");
        println!("Severity: 0x{:x?}", severity);
        let msg = unsafe { std::ffi::CStr::from_ptr(message) };
        println!("Message: '{}'", msg.to_str().unwrap());
        panic!();
    }
}
