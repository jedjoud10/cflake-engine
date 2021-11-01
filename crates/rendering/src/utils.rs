use std::{ffi::c_void, ptr::null};

// Some utils
pub struct Utils {
}

impl Utils {
    // Convert an OpenGL enum into a readable string
    pub fn convert_e(id: u32) -> String {
        /*
        let str = unsafe { 
            let ptr = gl::GetString(id);
            std::ffi::CStr::from_ptr(ptr as *const i8)
        }.to_str().unwrap();
        return str.to_string();
        */
        "".to_string()
    }
    // Constantly check for errors
    pub fn start_error_check_loop() {
        unsafe {
            println!("START OPENGL ERROR CHECK LOOP");
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(opengl_error_callback), null());
        }
    }
}

extern "system" fn opengl_error_callback(source: u32, _type: u32, id: u32, severity: u32, length: i32, message: *const i8, userParam: *mut c_void) {
    // Check if it was really an error
    if _type == gl::DEBUG_TYPE_ERROR {
        println!("We caught an OpenGL error!");
        println!("Severity: 0x{:x?}", severity);        
        let msg = unsafe { std::ffi::CStr::from_ptr(message) };
        println!("Message: '{}'", msg.to_str().unwrap());
        panic!();
    }
}