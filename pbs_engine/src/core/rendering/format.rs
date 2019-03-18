use pbs_gl as gl;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum BufferClearInternalFormat {
    R8 = gl::R8,
    R16 = gl::R16,
    R16F = gl::R16F,
    R32F = gl::R32F,
    R8I = gl::R8I,
    R16I = gl::R16I,
    R32I = gl::R32I,
    R8UI = gl::R8UI,
    R16UI = gl::R16UI,
    R32UI = gl::R32UI,
    RG8 = gl::RG8,
    RG16 = gl::RG16,
    RG16F = gl::RG16F,
    RG32F = gl::RG32F,
    RG8I = gl::RG8I,
    RG16I = gl::RG16I,
    RG32I = gl::RG32I,
    RG8UI = gl::RG8UI,
    RG16UI = gl::RG16UI,
    RG32UI = gl::RG32UI,
    RGB32F = gl::RGB32F,
    RGB32I = gl::RGB32I,
    RGB32UI = gl::RGB32UI,
    RGBA8 = gl::RGBA8,
    RGBA16 = gl::RGBA16,
    RGBA16F = gl::RGBA16F,
    RGBA32F = gl::RGBA32F,
    RGBA8I = gl::RGBA8I,
    RGBA16I = gl::RGBA16I,
    RGBA32I = gl::RGBA32I,
    RGBA8UI = gl::RGBA8UI,
    RGBA16UI = gl::RGBA16UI,
    RGBA32UI = gl::RGBA32UI
}