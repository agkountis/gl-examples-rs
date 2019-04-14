use image;
use image::{DynamicImage, GenericImage, GenericImageView, ColorType};

use pbs_gl as gl;
use gl::types::*;
use std::path::Path;
use core::borrow::Borrow;
use std::cmp::max;
use std::f32;
use std::fs::File;


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }
}

pub struct Texture2D {
    id: GLuint,
    image: DynamicImage
}

impl Texture2D {

//    pub fn new(width: u32, height: u32) -> Self {
//        let mut id: Gluint = 0;
//        unsafe {
//            gl::CreateTextures(gl::TEXTURE_2D, 1, &id);
//
//            gl::TextureStorage2D(id, 1, gl::RGBA8, width as i32, height as i32)
//        }
//    }

    pub fn new_from_file(path: &str, generate_mipmaps: bool) -> Result<Self, String> {

        if let Ok(img) = image::open(Path::new(path)) {
            let (width, height) = img.dimensions();

            let buffer= img.to_rgba();

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
                                     gl::SRGB8_ALPHA8,
                                     width as i32,
                                     height as i32);

                gl::TextureSubImage2D(id,
                                      0,
                                      0,
                                      0,
                                      width as i32,
                                      height as i32,
                                      gl::RGBA,
                                      gl::UNSIGNED_BYTE,
                                      buffer.to_vec().as_ptr() as *const GLvoid);

                if generate_mipmaps {
                    gl::GenerateTextureMipmap(id)
                }
            }

            Ok(Texture2D{
                id,
                image: img
            })
        }
        else {
            Err("foo".to_string())
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

