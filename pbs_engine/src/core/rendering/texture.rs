use image;
use image::{DynamicImage, GenericImageView, ColorType};

use pbs_gl as gl;
use gl::types::*;
use std::path::Path;
use std::f32;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum SizedTextureFormat {
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

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TextureFormat {
    Red = gl::RED,
    Rgb = gl::RGB,
    Rgba = gl::RGBA
}

pub struct Utils;

impl Utils {
    fn open_image_file(path: &str) -> Result<DynamicImage, String> {
        match image::open(Path::new(path)) {
            Ok(img) => Ok(img),
            Err(e) => Err(e.to_string()),
        }
    }

    fn color_type_to_texture_formats(color_type: ColorType, is_srgb: bool) -> Result<(SizedTextureFormat, TextureFormat), String> {
        match color_type {
            ColorType::Gray(_) => {
                Ok((SizedTextureFormat::R8, TextureFormat::Red))
            },
            ColorType::RGB(_) => {
                if is_srgb {
                    Ok((SizedTextureFormat::Srgb8, TextureFormat::Rgb))
                }
                else {
                    Ok((SizedTextureFormat::Rgb8, TextureFormat::Rgb))
                }
            },
            ColorType::RGBA(_) => {
                if is_srgb {
                    Ok((SizedTextureFormat::Srgb8A8, TextureFormat::Rgba))
                }
                else {
                    Ok((SizedTextureFormat::Rgba8, TextureFormat::Rgba))
                }
            },
            _ => Err(String::from("Unsupported texture format."))
        }
    }
}

pub struct Texture2D {
    id: GLuint,
    image: DynamicImage
}

impl Texture2D {

//    pub fn new(width: u32, height: u32, format: SizedTextureFormat) -> Self {
//        let mut id: GLuint = 0;
//        unsafe {
//            gl::CreateTextures(gl::TEXTURE_2D, 1, &id);
//
//            gl::TextureStorage2D(id, 1, format as u32, width as i32, height as i32)
//        }
//
//        let mut img;
//        match format {
//            SizedTextureFormat::R8 => img = DynamicImage::new_luma8(width, height),
//            SizedTextureFormat::R16 => img = DynamicImage::new_,
//            SizedTextureFormat::Rg8 => {},
//            SizedTextureFormat::Rgb8 => {},
//            SizedTextureFormat::Srgb8 => {},
//            SizedTextureFormat::Rgba8 => {},
//            SizedTextureFormat::Srgb8A8 => {},
//            SizedTextureFormat::Rgb16f => {},
//            SizedTextureFormat::Rgba32f => {},
//        }
//
//        Texture2D {
//            id: id,
//            image: DynamicImage::
//        }
//    }

    pub fn new_from_file(path: &str,
                         is_srgb: bool,
                         generate_mipmaps: bool) -> Result<Self, String> {

        let img = Utils::open_image_file(path);
        match image::open(Path::new(path)) {
            Ok(img) => {
                let (width, height) = img.dimensions();

                let formats;
                match Utils::color_type_to_texture_formats(img.color(), is_srgb) {
                    Ok(res) => formats = res,
                    Err(e) => return Err(e),
                }

                let mut mip_levels = 1;
                if generate_mipmaps {
                    mip_levels = (f32::floor(f32::log2(f32::max(width as f32,
                                                                height as f32))) + 1.0)
                        as i32;
                }

                let mut id: GLuint = 0;
                unsafe {
                    gl::CreateTextures(gl::TEXTURE_2D, 1, &mut id);

                    gl::TextureStorage2D(id,
                                         mip_levels,
                                         formats.0 as u32,
                                         width as i32,
                                         height as i32);

                    gl::TextureSubImage2D(id,
                                          0,
                                          0,
                                          0,
                                          width as i32,
                                          height as i32,
                                          formats.1 as u32,
                                          gl::UNSIGNED_BYTE,
                                          img.raw_pixels().as_ptr() as *const GLvoid);

                    if generate_mipmaps {
                        gl::GenerateTextureMipmap(id)
                    }
                }

                Ok(Texture2D{
                    id,
                    image: img
                })
            },
            Err(e) => {
                Err(e.to_string())
            }
        }
    }

    pub fn new_from_bytes(data: &[u8]) -> Result<Self, String> {
        Err("Bla".to_string())
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
        unsafe {
            gl::DeleteTextures(1, &self.id)
        }
    }
}

pub struct TextureCube {
    id: GLuint
}

impl TextureCube {
//    let a = image::hdr::HDRDecoder::new(io::BuffReader::new(File::open(&path).unwrap())).unwrap();
//            let data = a.read_image_hdr();
}

