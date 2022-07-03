use gl::types;
use std::ffi::{c_void, CStr};

// Get a static string that calls the glGetString function with the input value "symbolic"
// This must be called on the main thread (I think)
pub(crate) unsafe fn get_static_str(symbolic: u32) -> &'static str {
    let ptr = gl::GetString(symbolic);
    let str = CStr::from_ptr(ptr as *const i8);
    str.to_str().unwrap()
}

// Callback function for OpenGl debugging output
// This might log the newly fed message to a log or the console
pub(crate) extern "system" fn callback(
    source: types::GLenum,
    _type: types::GLenum,
    _id: types::GLuint,
    severity: types::GLenum,
    length: types::GLsizei,
    ptr: *const types::GLchar,
    _user: *mut c_void,
) {
    // Convert the source type to a user safe name
    let source = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
        gl::DEBUG_SOURCE_APPLICATION => "App",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
        gl::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
        _ => "Don't care",
    };

    // Convert the unsafe message pointer to a safe string
    let message = unsafe {
        let bytes = std::slice::from_raw_parts(ptr as *const u8, length as usize + 1);
        let msg = CStr::from_bytes_with_nul_unchecked(bytes);
        msg.to_str().unwrap()
    };

    // Convert the severity type to a user safe name
    let severity = match severity {
        gl::DEBUG_SEVERITY_LOW => "Low",
        gl::DEBUG_SEVERITY_MEDIUM => "Mid",
        gl::DEBUG_SEVERITY_HIGH => "High",
        _ => "Don't care",
    };

    // Print el messsage
    println!("{source}, {severity} severity, {message}");
}
