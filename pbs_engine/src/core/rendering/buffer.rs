use std::ptr;
use pbs_gl as gl;
use gl::types::*;
use crate::core::rendering::format::{BufferInternalFormat, DataFormat, DataType};


bitflags! {
    pub struct BufferStorageFlags : u32 {
        const DYNAMIC_STORAGE = gl::DYNAMIC_STORAGE_BIT;
        const MAP_READ = gl::MAP_READ_BIT;
        const MAP_WRITE = gl::MAP_WRITE_BIT;
        const MAP_PERSISTENT = gl::MAP_PERSISTENT_BIT;
        const MAP_COHERENT = gl::MAP_COHERENT_BIT;
        const CLIENT_STORAGE = gl::CLIENT_STORAGE_BIT;
        const MAP_READ_WRITE = Self::MAP_READ.bits | Self::MAP_WRITE.bits;
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum BufferAccess {
    ReadOnly = gl::READ_ONLY,
    WriteOnly = gl::WRITE_ONLY,
    ReadWrite = gl::READ_WRITE
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum BufferTarget {
    None = 0,
    Array = gl::ARRAY_BUFFER,
    AtomicCounter = gl::ATOMIC_COUNTER_BUFFER,
    CopyRead = gl::COPY_READ_BUFFER,
    CopyWrite = gl::COPY_WRITE_BUFFER,
    DispatchIndirect = gl::DISPATCH_INDIRECT_BUFFER,
    DrawIndirect = gl::DRAW_INDIRECT_BUFFER,
    ElementArray = gl::ELEMENT_ARRAY_BUFFER,
    PixelPack = gl::PIXEL_PACK_BUFFER,
    PixelUnpack = gl::PIXEL_UNPACK_BUFFER,
    QueryBuffer = gl::QUERY_BUFFER,
    ShaderStorage = gl::SHADER_STORAGE_BUFFER,
    Texture = gl::TEXTURE_BUFFER,
    TransformFeedback = gl::TRANSFORM_FEEDBACK_BUFFER,
    Uniform = gl::UNIFORM_BUFFER
}

pub struct BufferCopyInfo<'a> {
    pub source: &'a Buffer,
    pub destination: &'a Buffer,
    pub source_offset: isize,
    pub destination_offset: isize,
    pub size: isize
}

pub struct Buffer {
    id: GLuint,
    size: isize,
    mapped_ptr: *mut GLvoid,
    storage_flags: BufferStorageFlags,
    current_bound_target: BufferTarget
}

impl Buffer {
    pub fn new(size: isize, buffer_storage_flags: BufferStorageFlags) -> Buffer {
        let mut id: GLuint = 0;

        unsafe {
            gl::CreateBuffers(1, &mut id);
            gl::NamedBufferStorage(id, size, ptr::null(), buffer_storage_flags.bits());
        }

        Buffer {
            id,
            size,
            mapped_ptr: ptr::null_mut(),
            storage_flags: buffer_storage_flags,
            current_bound_target: BufferTarget::None
        }
    }

    pub fn new_with_data(data: &[u8],
                         size: isize,
                         buffer_storage_flags: BufferStorageFlags) -> Buffer {
        let mut id: GLuint = 0;

        unsafe {
            gl::CreateBuffers(1, &mut id);
            gl::NamedBufferStorage(
                id,
                size,
                data.as_ptr() as *const GLvoid,
                buffer_storage_flags.bits()
            );
        }

        Buffer {
            id,
            size,
            mapped_ptr: ptr::null_mut(),
            storage_flags: buffer_storage_flags,
            current_bound_target: BufferTarget::None
        }
    }

    pub fn bind(&self, buffer_target: BufferTarget) {
        unsafe {
            gl::BindBuffer(buffer_target as u32, self.id)
        }
    }

    pub fn unbind(&mut self) {
        unsafe {
            gl::BindBuffer(BufferTarget::None as u32, self.id);
            self.current_bound_target = BufferTarget::None;
        }
    }

    pub fn map(&mut self, buffer_access: BufferAccess) {
        self.map_range(0, self.size, buffer_access)
    }

    pub fn map_range(&mut self, offset: isize, length: isize, buffer_access: BufferAccess) {
        assert!(self.storage_flags.intersects(BufferStorageFlags::MAP_READ_WRITE),
                "Cannot map buffer.\n \
                Reason: Buffer was storage does not support memory mapping.\n\
                Hint: Create the buffer using BufferStorageFlags::MAP_READ, BufferStorageFlags::MAP_WRITE \
                or BUFFER_STORAGE_FLAGS::MAP_READ_WRITE");

        assert!(self.storage_flags.intersects(match buffer_access {
            BufferAccess::ReadOnly => BufferStorageFlags::MAP_READ,
            BufferAccess::WriteOnly => BufferStorageFlags::MAP_WRITE,
            BufferAccess::ReadWrite => BufferStorageFlags::MAP_READ_WRITE
        }),
                "Cannot map buffer. \n\
                Reason: buffer_access function parameter not contained \
                in the buffer's storage flags.\n\
                Hint: Create the buffer using BufferStorageFlags::MAP_<READ/WRITE/READ_WRITE> \
                to match the buffer_access function parameter.");

        assert!((offset + length) < self.size,
                "Cannot map buffer.\n\
                Reason: Requested range exceeds buffer capacity (out of bounds).\n\
                Hint: Offset + length must be smaller than the total buffer length(capacity)");

        if self.mapped_ptr == ptr::null_mut() {
            unsafe {
                self.mapped_ptr = gl::MapNamedBufferRange(self.id, offset, length, buffer_access as u32)
            }
        } else {
            println!("WARNING: Buffer already mapped. This call has no effect.")
        }
    }

    pub fn unmap(&mut self) {
        unsafe {
            gl::UnmapNamedBuffer(self.id);
        }
        self.mapped_ptr = ptr::null_mut();
    }

    pub fn fill(&self, offset: isize, size: isize, data: &[u8]) {
        assert!(self.storage_flags.intersects(BufferStorageFlags::DYNAMIC_STORAGE),
                "Cannot fill non-mapped buffer. \n \
                Reason: Not able to call glBufferSubData(...).\n\
                Hint: Create the buffer using BufferStorageFlags::DYNAMIC_STORAGE \
                for non-mapped data updates.");

        unsafe {
            gl::NamedBufferSubData(self.id, offset, size, data.as_ptr() as *const GLvoid)
        }
    }

    pub fn fill_mapped(&mut self, data: &[u8], size: usize) {
        assert_ne!(self.mapped_ptr, ptr::null_mut(),
                   "Attempting to fill unmapped buffer. Please map the buffer first by calling \
                   map(&mut self, buffer_access: BufferAccess)");

        assert!(self.storage_flags.intersects(BufferStorageFlags::MAP_WRITE),
                "Cannot fill mapped buffer.\n\
                Reason: Buffer not created using the flag BufferStorageFlags::MAP_WRITE.\n\
                Hint: Create the buffer using BufferStorageFlags::MAP_WRITE");

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.mapped_ptr as *mut u8, size)
        }
    }

    pub fn get_id(&self) -> GLuint {
        self.id
    }

    pub fn get_size(&self) -> isize {
        self.size
    }

    pub fn is_mapped(&self) -> bool {
        self.mapped_ptr != ptr::null_mut()
    }

    pub fn get_storage_flags(&self) -> BufferStorageFlags {
        self.storage_flags
    }

    pub fn clear(&self,
                 internal_format: BufferInternalFormat,
                 date_format: DataFormat,
                 data_type: DataType,
                 data: &[u8]) {
        self.clear_range(internal_format, 0, self.size, data_format, data_type, data)
    }

    pub fn clear_range(&self,
                       internal_format: BufferInternalFormat,
                       offset: isize,
                       size: isize,
                       data_format: DataFormat,
                       data_type: DataType,
                       data: &[u8]) {

        //TODO: Add asserts to ensure spec correctness

        unsafe {
            gl::ClearNamedBufferSubData(self.id,
                                        internal_format as u32,
                                        offset,
                                        size,
                                        data_format as u32,
                                        data_type as u32,
                                        data.as_ptr() as *const GLvoid);
        }
    }

    pub fn copy(buffer_copy_info: BufferCopyInfo) {
        assert!(buffer_copy_info.source_offset > 0, "Buffer copy source offset must be > 0.");
        assert!(buffer_copy_info.destination_offset > 0,
                "Buffer copy destination offset must be > 0");
        assert!(buffer_copy_info.size > 0, "Buffer copy size must be > 0.");
        assert!(buffer_copy_info.source_offset + buffer_copy_info.size
            <= buffer_copy_info.source.get_size(),
                "Source offset + size must be less or equal to source buffer size");
        assert!(buffer_copy_info.destination_offset + buffer_copy_info.size
                    <= buffer_copy_info.destination.get_size(),
                "Destination offset + size must be less or equal to destination buffer size");

        if buffer_copy_info.source.get_id() == buffer_copy_info.destination.get_id() {
            assert_ne!(buffer_copy_info.source_offset + buffer_copy_info.size - buffer_copy_info.source_offset,
                       buffer_copy_info.destination_offset + buffer_copy_info.size - buffer_copy_info.destination_offset,
                       "Source and destination memory ranges cannot overlap when copying on the same buffer")
        }

        if buffer_copy_info.source.is_mapped() {
            assert!(buffer_copy_info.source.get_storage_flags().intersects(BufferStorageFlags::MAP_PERSISTENT),
                    "Mapped buffer 'source' can only be used if mapped persistently. \
                    Map persistently or unmap the buffer before attempting to copy.")
        }

        if buffer_copy_info.destination.is_mapped() {
            assert!(buffer_copy_info.destination.get_storage_flags().intersects(BufferStorageFlags::MAP_PERSISTENT),
                    "Mapped buffer 'destination' can only be used if mapped persistently. \
                    Map persistently or unmap the buffer before attempting to copy.")
        }

        unsafe {
            gl::CopyNamedBufferSubData(buffer_copy_info.source.get_id(),
                                       buffer_copy_info.destination.get_id(),
                                       buffer_copy_info.source_offset,
                                       buffer_copy_info.destination_offset,
                                       buffer_copy_info.size)
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.unbind();
            gl::DeleteBuffers(1, &mut self.id)
        }
    }
}
