use super::{ReadBytes, WriteBytes};

// A byte operation that we can execute on an OpenGL buffer
pub enum BufferOperation {
    Write(WriteBytes),
    Read(ReadBytes),
}