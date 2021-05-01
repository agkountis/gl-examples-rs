use crate::rendering::format::{BufferInternalFormat, DataFormat, DataType};
use gl::types::*;
use gl_bindings as gl;
use std::ffi::CString;
use std::{mem, ptr};

bitflags! {
    pub struct BufferStorageFlags : u32 {
        const DYNAMIC = gl::DYNAMIC_STORAGE_BIT;
        const MAP_READ = gl::MAP_READ_BIT;
        const MAP_WRITE = gl::MAP_WRITE_BIT;
        const MAP_PERSISTENT = gl::MAP_PERSISTENT_BIT;
        const MAP_COHERENT = gl::MAP_COHERENT_BIT;
        const CLIENT_STORAGE = gl::CLIENT_STORAGE_BIT;
        const MAP_READ_WRITE = Self::MAP_READ.bits | Self::MAP_WRITE.bits;
        const MAP_READ_WRITE_PERSISTENT_COHERENT = Self::MAP_READ.bits | Self::MAP_WRITE.bits | Self::MAP_PERSISTENT.bits | Self::MAP_COHERENT.bits;
        const MAP_WRITE_COHERENT = Self::MAP_WRITE.bits | Self::MAP_COHERENT.bits;
        const MAP_WRITE_PERSISTENT_COHERENT = Self::MAP_WRITE.bits | Self::MAP_PERSISTENT.bits | Self::MAP_COHERENT.bits;
    }
}

bitflags! {
    pub struct MapModeFlags : u32 {
        const MAP_READ = gl::MAP_READ_BIT;
        const MAP_WRITE = gl::MAP_WRITE_BIT;
        const MAP_PERSISTENT = gl::MAP_PERSISTENT_BIT;
        const MAP_COHERENT = gl::MAP_COHERENT_BIT;
        const MAP_READ_WRITE = Self::MAP_READ.bits | Self::MAP_WRITE.bits;
        const MAP_READ_WRITE_PERSISTENT_COHERENT = Self::MAP_READ.bits | Self::MAP_WRITE.bits | Self::MAP_PERSISTENT.bits | Self::MAP_COHERENT.bits;
        const MAP_WRITE_COHERENT = Self::MAP_WRITE.bits | Self::MAP_COHERENT.bits;
        const MAP_WRITE_PERSISTENT_COHERENT = Self::MAP_WRITE.bits | Self::MAP_PERSISTENT.bits | Self::MAP_COHERENT.bits;
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
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
    Uniform = gl::UNIFORM_BUFFER,
}

pub struct BufferCopyInfo<'a> {
    pub source: &'a Buffer,
    pub destination: &'a Buffer,
    pub source_offset: isize,
    pub destination_offset: isize,
    pub size: isize,
}

pub struct Buffer {
    _name: String,
    id: GLuint,
    size: isize,
    mapped_ptr: *mut GLvoid,
    storage_flags: BufferStorageFlags,
    current_bound_target: BufferTarget,
}

impl Buffer {
    pub fn new(
        name: &str,
        size: isize,
        buffer_target: BufferTarget,
        buffer_storage_flags: BufferStorageFlags,
    ) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::CreateBuffers(1, &mut id);
            gl::NamedBufferStorage(id, size, ptr::null(), buffer_storage_flags.bits());

            let label = CString::new(name).unwrap();
            gl::ObjectLabel(gl::BUFFER, id, name.len() as i32 + 1, label.as_ptr())
        }

        Self {
            _name: name.to_string(),
            id,
            size,
            mapped_ptr: ptr::null_mut(),
            storage_flags: buffer_storage_flags,
            current_bound_target: buffer_target,
        }
    }

    pub fn new_with_data<T: Sized>(
        name: &str,
        data: &T,
        buffer_target: BufferTarget,
        buffer_storage_flags: BufferStorageFlags,
    ) -> Self {
        let mut id: GLuint = 0;
        let size = mem::size_of::<T>() as isize;
        unsafe {
            gl::CreateBuffers(1, &mut id);
            gl::NamedBufferStorage(
                id,
                size,
                data as *const T as *const GLvoid,
                buffer_storage_flags.bits(),
            );

            let label = CString::new(name).unwrap();
            gl::ObjectLabel(gl::BUFFER, id, name.len() as i32 + 1, label.as_ptr())
        }

        Self {
            _name: name.to_string(),
            id,
            size,
            mapped_ptr: ptr::null_mut(),
            storage_flags: buffer_storage_flags,
            current_bound_target: buffer_target,
        }
    }

    pub fn new_from_slice<T>(
        name: &str,
        data: &[T],
        buffer_target: BufferTarget,
        buffer_storage_flags: BufferStorageFlags,
    ) -> Self {
        let mut id: GLuint = 0;
        let size = (data.len() * mem::size_of::<T>()) as isize;
        unsafe {
            gl::CreateBuffers(1, &mut id);
            gl::NamedBufferStorage(
                id,
                size,
                data.as_ptr() as *const GLvoid,
                buffer_storage_flags.bits(),
            );

            let label = CString::new(name).unwrap();
            gl::ObjectLabel(gl::BUFFER, id, name.len() as i32 + 1, label.as_ptr())
        }

        Self {
            _name: name.to_string(),
            id,
            size,
            mapped_ptr: ptr::null_mut(),
            storage_flags: buffer_storage_flags,
            current_bound_target: buffer_target,
        }
    }

    pub fn bind(&self, binding_index: u32) {
        self.bind_range(binding_index, 0, self.size);
    }

    pub fn bind_range(&self, binding_index: u32, offset: isize, size: isize) {
        assert!(
            self.current_bound_target == BufferTarget::Uniform
                || self.current_bound_target == BufferTarget::ShaderStorage
                || self.current_bound_target == BufferTarget::AtomicCounter
                || self.current_bound_target == BufferTarget::TransformFeedback,
            "Cannot bind buffer range. Buffer target is not one of [ShaderStorage|AtomicCounter|Uniform|TransformFeedback]."
        );
        assert!(
            offset + size <= self.size,
            "Buffer bind operation out of buffer range. Buffer size: {}, Requested bind offset: {}, Requested bind size: {}",
            self.size,
            offset,
            size
        );
        unsafe {
            gl::BindBufferRange(
                self.current_bound_target as u32,
                binding_index,
                self.id,
                offset,
                size,
            )
        }
    }

    pub fn map(&mut self, map_mode: MapModeFlags) {
        self.map_range(0, self.size, map_mode)
    }

    pub fn map_range(&mut self, offset: isize, length: isize, map_mode: MapModeFlags) {
        assert!(self.storage_flags.intersects(BufferStorageFlags::MAP_READ_WRITE),
                "Cannot map buffer.\n \
                Reason: Buffer was storage does not support memory mapping.\n\
                Hint: Create the buffer using BufferStorageFlags::MAP_READ, BufferStorageFlags::MAP_WRITE \
                or BUFFER_STORAGE_FLAGS::MAP_READ_WRITE");

        // assert!({
        //     let flags = 0u32;
        // }
        //     self.storage_flags.intersects({
        //
        //         match map_mode {
        //
        //         }
        //         MapModeFlags::MAP_READ
        //     })
        //         || self.storage_flags.intersects(MapModeFlags::MAP_WRITE)
        //         || self.storage_flags.intersects(MapModeFlags::MAP_READ_WRITE),
        //     "Cannot map buffer. \n\
        //         Reason: buffer_access function parameter not contained \
        //         in the buffer's storage flags.\n\
        //         Hint: Create the buffer using BufferStorageFlags::MAP_<READ/WRITE/READ_WRITE> \
        //         to match the buffer_access function parameter."
        // );

        assert!(
            (offset + length) <= self.size,
            "Cannot map buffer.\n\
                Reason: Requested range exceeds buffer capacity (out of bounds).\n\
                Hint: Offset + length must be smaller than the total buffer length(capacity)"
        );

        if self.mapped_ptr == ptr::null_mut() {
            unsafe {
                self.mapped_ptr = gl::MapNamedBufferRange(self.id, offset, length, map_mode.bits())
            }
        } else {
            println!("WARNING: Buffer already mapped. This call has no effect.")
        }
    }

    pub fn unmap(&mut self) {
        // TODO: Validate what happens when you call unmap on a persistently mapped buffer?
        unsafe {
            gl::UnmapNamedBuffer(self.id);
        }
        self.mapped_ptr = ptr::null_mut();
    }

    pub fn fill<T>(&self, offset: isize, data: &T) {
        assert!(
            self.storage_flags.intersects(BufferStorageFlags::DYNAMIC),
            "Cannot fill non-mapped buffer. \n \
                Reason: Not able to call glBufferSubData(...).\n\
                Hint: Create the buffer using BufferStorageFlags::DYNAMIC \
                for non-mapped data updates."
        );

        unsafe {
            gl::NamedBufferSubData(
                self.id,
                offset,
                std::mem::size_of::<T>() as isize,
                data as *const T as *const GLvoid,
            )
        }
    }

    pub fn fill_mapped<T: Sized>(&self, offset: isize, data: &T) {
        assert_ne!(
            self.mapped_ptr,
            ptr::null_mut(),
            "Attempting to fill unmapped buffer. Please map the buffer first by calling \
                   map(&mut self, buffer_access: BufferAccess)"
        );

        assert!(
            self.storage_flags.intersects(BufferStorageFlags::MAP_WRITE),
            "Cannot fill mapped buffer.\n\
                Reason: Buffer not created using the flag BufferStorageFlags::MAP_WRITE.\n\
                Hint: Create the buffer using BufferStorageFlags::MAP_WRITE"
        );

        assert_eq!(self.size, mem::size_of::<T>() as isize);
        assert!(offset + std::mem::size_of::<T>() as isize <= self.size);

        let source = unsafe { (data as *const T).offset(offset) };

        unsafe { ptr::copy_nonoverlapping(source, self.mapped_ptr as *mut T, 1) }
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

    pub fn clear(
        &self,
        internal_format: BufferInternalFormat,
        data_format: DataFormat,
        data_type: DataType,
        data: &[u8],
    ) {
        self.clear_range(internal_format, 0, self.size, data_format, data_type, data)
    }

    pub fn clear_range(
        &self,
        internal_format: BufferInternalFormat,
        offset: isize,
        size: isize,
        data_format: DataFormat,
        data_type: DataType,
        data: &[u8],
    ) {
        //TODO: Add asserts to ensure spec correctness

        unsafe {
            gl::ClearNamedBufferSubData(
                self.id,
                internal_format as u32,
                offset,
                size,
                data_format as u32,
                data_type as u32,
                data.as_ptr() as *const GLvoid,
            );
        }
    }

    pub fn copy(buffer_copy_info: BufferCopyInfo) {
        assert!(
            buffer_copy_info.source_offset > 0,
            "Buffer copy source offset must be > 0."
        );
        assert!(
            buffer_copy_info.destination_offset > 0,
            "Buffer copy destination offset must be > 0"
        );
        assert!(buffer_copy_info.size > 0, "Buffer copy size must be > 0.");
        assert!(
            buffer_copy_info.source_offset + buffer_copy_info.size
                <= buffer_copy_info.source.get_size(),
            "Source offset + size must be less or equal to source buffer size"
        );
        assert!(
            buffer_copy_info.destination_offset + buffer_copy_info.size
                <= buffer_copy_info.destination.get_size(),
            "Destination offset + size must be less or equal to destination buffer size"
        );

        if buffer_copy_info.source.get_id() == buffer_copy_info.destination.get_id() {
            assert_ne!(buffer_copy_info.source_offset + buffer_copy_info.size - buffer_copy_info.source_offset,
                       buffer_copy_info.destination_offset + buffer_copy_info.size - buffer_copy_info.destination_offset,
                       "Source and destination memory ranges cannot overlap when copying on the same buffer")
        }

        if buffer_copy_info.source.is_mapped() {
            assert!(
                buffer_copy_info
                    .source
                    .get_storage_flags()
                    .intersects(BufferStorageFlags::MAP_PERSISTENT),
                "Mapped buffer 'source' can only be used if mapped persistently. \
                    Map persistently or unmap the buffer before attempting to copy."
            )
        }

        if buffer_copy_info.destination.is_mapped() {
            assert!(
                buffer_copy_info
                    .destination
                    .get_storage_flags()
                    .intersects(BufferStorageFlags::MAP_PERSISTENT),
                "Mapped buffer 'destination' can only be used if mapped persistently. \
                    Map persistently or unmap the buffer before attempting to copy."
            )
        }

        unsafe {
            gl::CopyNamedBufferSubData(
                buffer_copy_info.source.get_id(),
                buffer_copy_info.destination.get_id(),
                buffer_copy_info.source_offset,
                buffer_copy_info.destination_offset,
                buffer_copy_info.size,
            )
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.is_mapped() {
            self.unmap()
        }
        unsafe { gl::DeleteBuffers(1, &mut self.id) }
    }
}
