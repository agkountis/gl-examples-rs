use std::ptr;
use pbs_gl as gl;
use gl::types::*;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum BufferUsage {
    StreamDraw = gl::STREAM_DRAW,
    StreamRead = gl::STREAM_READ,
    StreamCopy = gl::STREAM_COPY,
    StaticDraw = gl::STATIC_DRAW,
    StaticRead = gl::STATIC_READ,
    StaticCopy = gl::STATIC_COPY,
    DynamicDraw = gl::DYNAMIC_DRAW,
    DynamicRead = gl::DYNAMIC_READ,
    DynamicCopy = gl::DYNAMIC_COPY
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum BufferAccess {
    ReadOnly = gl::READ_ONLY,
    WriteOnly = gl::WRITE_ONLY,
    ReadWrite = gl::READ_WRITE
}

pub struct Buffer {
    id: GLuint,
    mapped_ptr: *mut GLvoid,
    usage: BufferUsage
}

impl Buffer {
    pub fn new(buffer_usage: BufferUsage, size: isize) -> Buffer {
        let mut id: GLuint = 0;

        unsafe {
            gl::CreateBuffers(1, &mut id);
            gl::NamedBufferData(id, size, ptr::null(), buffer_usage as u32)
        }


        Buffer {
            id,
            mapped_ptr: ptr::null_mut(),
            usage: buffer_usage
        }
    }

    pub fn map(&mut self, buffer_access: BufferAccess) {
        unsafe {
            self.mapped_ptr = gl::MapNamedBuffer(self.id, buffer_access as u32)
        }
    }

    pub fn fill(&mut self, data: &[u8], size: usize) {
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.mapped_ptr as *mut u8, size)
        }
    }

    pub fn unmap(&mut self) {
        unsafe {
            gl::UnmapNamedBuffer(self.id);
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.id)
        }
    }
}
