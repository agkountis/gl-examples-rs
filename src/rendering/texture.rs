use image;
use image::{ColorType, DynamicImage, GenericImageView};

use gli::GliTexture;
use gli_rs as gli;

use crate::core::asset::Asset;
use gl::types::*;
use gl_bindings as gl;
use std::path::Path;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum SizedTextureFormat {
    R8 = gl::R8,
    R16 = gl::R16,
    Rg8 = gl::RG8,
    Rgb8 = gl::RGB8,
    Srgb8 = gl::SRGB8,
    Rgba8 = gl::RGBA8,
    Srgb8A8 = gl::SRGB8_ALPHA8,
    Rg16f = gl::RG16F,
    Rgb16f = gl::RGB16F,
    Rgba16f = gl::RGBA16F,
    Rgb32f = gl::RGB32F,
    Rgba32f = gl::RGBA32F,
    Depth16 = gl::DEPTH_COMPONENT16,
    Depth24 = gl::DEPTH_COMPONENT24,
    Depth32 = gl::DEPTH_COMPONENT32,
    Depth32f = gl::DEPTH_COMPONENT32F,
    Depth32fStencil8 = gl::DEPTH32F_STENCIL8,
    Depth24Stencil8 = gl::DEPTH24_STENCIL8,
    StencilIndex8 = gl::STENCIL_INDEX8,
}

impl SizedTextureFormat {
    pub fn is_floating_point(&self) -> bool {
        match self {
            Self::Rg16f |
            Self::Rgb16f |
            Self::Rgba16f |
            Self::Rgb32f |
            Self::Rgba32f |
            Self::Depth32f |
            Self::Depth32fStencil8 => true,
            _ => false,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TextureFormat {
    Red = gl::RED,
    Rg = gl::RG,
    Rgb = gl::RGB,
    Rgba = gl::RGBA,
}

pub struct Utils;

impl Utils {
    fn open_image_file<P: AsRef<Path>>(path: P) -> Result<DynamicImage, String> {
        match image::open(path.as_ref()) {
            Ok(img) => Ok(img),
            Err(e) => Err(e.to_string()),
        }
    }

    fn color_type_to_texture_formats(
        color_type: ColorType,
        is_srgb: bool,
    ) -> Result<(SizedTextureFormat, TextureFormat), String> {
        match color_type {
            ColorType::Gray(_) => Ok((SizedTextureFormat::R8, TextureFormat::Red)),
            ColorType::GrayA(_) => Ok((SizedTextureFormat::Rg8, TextureFormat::Rg)),
            ColorType::RGB(_) => {
                if is_srgb {
                    Ok((SizedTextureFormat::Srgb8, TextureFormat::Rgb))
                } else {
                    Ok((SizedTextureFormat::Rgb8, TextureFormat::Rgb))
                }
            }
            ColorType::RGBA(_) => {
                if is_srgb {
                    Ok((SizedTextureFormat::Srgb8A8, TextureFormat::Rgba))
                } else {
                    Ok((SizedTextureFormat::Rgba8, TextureFormat::Rgba))
                }
            }
            _ => Err(String::from("Unsupported texture format.")),
        }
    }
}

pub struct Texture2D {
    id: GLuint,
    image: DynamicImage,
}

pub struct Texture2DLoadConfig {
    pub is_srgb: bool,
    pub generate_mipmap: bool,
}

impl Asset for Texture2D {
    type Output = Self;
    type Error = String;
    type LoadConfig = Texture2DLoadConfig;

    fn load<P: AsRef<Path>>(
        path: P,
        load_config: Option<Self::LoadConfig>,
    ) -> Result<Self::Output, Self::Error> {
        let mut is_srgb = false;
        let mut generate_mipmap = false;

        if let Some(config) = load_config {
            is_srgb = config.is_srgb;
            generate_mipmap = config.generate_mipmap;
        }

        match Utils::open_image_file(path.as_ref()) {
            Ok(img) => Ok(Self::new_from_image(img, generate_mipmap, is_srgb)?),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl Texture2D {
    pub fn new_from_image(
        image: DynamicImage,
        generate_mipmap: bool,
        is_srgb: bool,
    ) -> Result<Self, String> {
        let (width, height) = image.dimensions();

        let formats;
        match Utils::color_type_to_texture_formats(image.color(), is_srgb) {
            Ok(res) => formats = res,
            Err(e) => return Err(e),
        }

        let mut mip_levels = 1;
        if generate_mipmap {
            mip_levels =
                (f32::floor(f32::log2(f32::max(width as f32, height as f32))) + 1.0) as i32;
        }

        let mut id: GLuint = 0;
        unsafe {
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut id);

            gl::TextureStorage2D(
                id,
                mip_levels,
                formats.0 as u32,
                width as i32,
                height as i32,
            );

            gl::TextureSubImage2D(
                id,
                0,
                0,
                0,
                width as i32,
                height as i32,
                formats.1 as u32,
                gl::UNSIGNED_BYTE,
                image.raw_pixels().as_ptr() as *const GLvoid,
            );

            if generate_mipmap {
                gl::GenerateTextureMipmap(id)
            }
        }

        Ok(Self { id, image })
    }

    pub fn get_id(&self) -> GLuint {
        self.id
    }

    pub fn get_image(&self) -> &DynamicImage {
        &self.image
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id) }
    }
}

pub struct TextureCube {
    id: GLuint,
}

impl Asset for TextureCube {
    type Output = Self;
    type Error = String;
    type LoadConfig = ();

    fn load<P: AsRef<Path>>(
        path: P,
        _: Option<Self::LoadConfig>,
    ) -> Result<Self::Output, Self::Error> {
        let result: gli::Result<gli::TextureCube> = gli::load(path.as_ref());
        match result {
            Ok(tex) => {
                println!("Cube load ok!");
                println!("KTX Texture info:");
                println!(
                    "\tExtent: ({}, {})",
                    tex.extent(0).width,
                    tex.extent(0).height
                );
                println!("\tFaces  count: {}", tex.faces());
                println!("\tLayers count: {}", tex.layers());
                println!("\tLevels count: {}", tex.levels());
                println!("\tSize: {}", tex.size());
                println!("\tAddress: {:?}", tex.data());
                println!("\tFormat: {}", tex.format());
                println!("\tTarget: {}", tex.target());
                println!();

                let (internal_format, external_format, data_type) =
                    Self::translate_gli_format_info(tex.format());

                let mut id: GLuint = 0;
                unsafe {
                    gl::CreateTextures(gl::TEXTURE_CUBE_MAP, 1, &mut id);

                    gl::TextureStorage2D(
                        id,
                        tex.levels() as i32,
                        internal_format as u32,
                        tex.extent(0).width as i32,
                        tex.extent(0).height as i32,
                    );

                    for _ in 0..tex.layers() {
                        for face in 0..tex.faces() {
                            let gl_face = gl::TEXTURE_CUBE_MAP_POSITIVE_X + face as u32;
                            let face_tex = tex.get_face(gl_face as usize);

                            for level in 0..tex.levels() {
                                let image = face_tex.get_level(level);

                                // Cubemaps + DSA = TextureSubImage3D using zOffset as the face index
                                gl::TextureSubImage3D(
                                    id,
                                    level as i32,
                                    0,
                                    0,
                                    face as i32,
                                    image.extent().width as i32,
                                    image.extent().height as i32,
                                    1,
                                    external_format as u32,
                                    data_type,
                                    image.data(),
                                );
                            }
                        }
                    }
                }

                Ok(TextureCube { id })
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

impl TextureCube {
    //TODO: To be removed
    pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let result: gli::Result<gli::TextureCube> = gli::load(path.as_ref());
        match result {
            Ok(tex) => {
                println!("Cube load ok!");
                println!("KTX Texture info:");
                println!(
                    "\tExtent: ({}, {})",
                    tex.extent(0).width,
                    tex.extent(0).height
                );
                println!("\tFaces  count: {}", tex.faces());
                println!("\tLayers count: {}", tex.layers());
                println!("\tLevels count: {}", tex.levels());
                println!("\tSize: {}", tex.size());
                println!("\tAddress: {:?}", tex.data());
                println!("\tFormat: {}", tex.format());
                println!("\tTarget: {}", tex.target());
                println!();

                let (internal_format, external_format, data_type) =
                    Self::translate_gli_format_info(tex.format());

                let mut id: GLuint = 0;
                unsafe {
                    gl::CreateTextures(gl::TEXTURE_CUBE_MAP, 1, &mut id);
                    gl::TextureStorage2D(
                        id,
                        tex.levels() as i32,
                        internal_format as u32,
                        tex.extent(0).width as i32,
                        tex.extent(0).height as i32,
                    );

                    for _ in 0..tex.layers() {
                        for face in 0..tex.faces() {
                            let face_tex = tex.get_face(face);

                            for level in 0..tex.levels() {
                                let image = face_tex.get_level(level);

                                // Cubemaps + DSA = TextureSubImage3D using zOffset as the face index
                                gl::TextureSubImage3D(
                                    id,
                                    level as i32,
                                    0,
                                    0,
                                    face as i32,
                                    image.extent().width as i32,
                                    image.extent().height as i32,
                                    1,
                                    external_format as u32,
                                    data_type,
                                    image.data(),
                                );
                            }
                        }
                    }
                }

                Ok(TextureCube { id })
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn get_id(&self) -> GLuint {
        self.id
    }

    fn translate_gli_format_info(
        format: gli::Format,
    ) -> (SizedTextureFormat, TextureFormat, GLenum) {
        match format {
            gli::Format::RGB16_SFLOAT_PACK16 => (
                SizedTextureFormat::Rgb16f,
                TextureFormat::Rgb,
                gl::HALF_FLOAT,
            ),
            gli::Format::RGBA16_SFLOAT_PACK16 => (
                SizedTextureFormat::Rgba16f,
                TextureFormat::Rgba,
                gl::HALF_FLOAT,
            ),
            gli::Format::RGB8_UNORM_PACK8 => (
                SizedTextureFormat::Rgb8,
                TextureFormat::Rgb,
                gl::UNSIGNED_INT,
            ),
            gli::Format::RGBA32_SFLOAT_PACK32 => {
                (SizedTextureFormat::Rgba32f, TextureFormat::Rgba, gl::FLOAT)
            }
            gli::Format::RGB32_SFLOAT_PACK32 => {
                (SizedTextureFormat::Rgb32f, TextureFormat::Rgb, gl::FLOAT)
            }
            _ => (
                SizedTextureFormat::Rgba8,
                TextureFormat::Rgba,
                gl::UNSIGNED_INT,
            ),
        }
    }
}

impl Drop for TextureCube {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id) }
    }
}
