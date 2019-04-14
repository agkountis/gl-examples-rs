use pbs_gl as gl;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum BufferInternalFormat {
    R8 = gl::R8,
    R16 = gl::R16,
    R16f = gl::R16F,
    R32f = gl::R32F,
    R8i = gl::R8I,
    R16i = gl::R16I,
    R32i = gl::R32I,
    R8ui = gl::R8UI,
    R16ui = gl::R16UI,
    R32ui = gl::R32UI,
    Rg8 = gl::RG8,
    Rg16 = gl::RG16,
    Rg16f = gl::RG16F,
    Rg32f = gl::RG32F,
    Rg8i = gl::RG8I,
    Rg16i = gl::RG16I,
    Rg32i = gl::RG32I,
    Rg8ui = gl::RG8UI,
    Rg16ui = gl::RG16UI,
    Rg32ui = gl::RG32UI,
    Rgb32f = gl::RGB32F,
    Rgb32i = gl::RGB32I,
    Rgb32ui = gl::RGB32UI,
    Rgba8 = gl::RGBA8,
    Rgba16 = gl::RGBA16,
    Rgba16f = gl::RGBA16F,
    Rgba32f = gl::RGBA32F,
    Rgba8i = gl::RGBA8I,
    Rgba16i = gl::RGBA16I,
    Rgba32i = gl::RGBA32I,
    Rgba8ui = gl::RGBA8UI,
    Rgba16ui = gl::RGBA16UI,
    Rgba32ui = gl::RGBA32UI
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum DataType {
    Byte = gl::BYTE,
    UnsignedByte = gl::UNSIGNED_BYTE,
    Short = gl::SHORT,
    UnsignedShort = gl::UNSIGNED_SHORT,
    Int = gl::INT,
    UnsignedInt = gl::UNSIGNED_INT,
    HalfFloat = gl::HALF_FLOAT,
    Float = gl::FLOAT
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    StencilIndex = gl::STENCIL_INDEX,
    DepthComponent = gl::DEPTH_COMPONENT,
    DepthStencil = gl::DEPTH_STENCIL,
    Red = gl::RED,
    Green = gl::GREEN,
    Blue = gl::BLUE,
    Rg = gl::RG,
    Rgb = gl::RGB,
    Rgba = gl::RGBA,
    Bgr = gl::BGR,
    Bgra = gl::BGRA,
    RedInteger = gl::RED_INTEGER,
    BlueInteger = gl::BLUE_INTEGER,
    RgInteger = gl::RG_INTEGER,
    RgbInteger = gl::RGB_INTEGER,
    RgbaInteger = gl::RGBA_INTEGER,
    BgrInteger = gl::BGR_INTEGER,
    BgraInteger = gl::BGRA_INTEGER
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TextureStorageFormat {
    R8 = gl::R8,
    R16 = gl::R16,
    Rg8 = gl::RG8,
    Rgb8 = gl::RGB8,
    Srgb8 = gl::SRGB8,
    Rgba8 = gl::RGBA8,
    Srgb8A8 = gl::SRGB8_ALPHA8,
    Rgb16f = gl::RGB16F,
    Rgba32f = gl::RGBA32F
}

