// Used to catch errors from OpenGL and other unsafe sources
pub struct ErrorCatcher {}

impl ErrorCatcher {
    // Catch opengl error
    pub fn catch_opengl_errors() -> Option<()> {
        // Check for OpenGL errors
        unsafe {
            let error: gl::types::GLenum = gl::GetError();
            if error != gl::NO_ERROR {
                // We caught an error!
                println!("\x1b[31mWe caught an OpenGL error! Error code: '{:x?}'\x1b[0m", error);
                //panic!();
                return Some(());
            } else {
                return Some(());
            }
        }
    }
}
