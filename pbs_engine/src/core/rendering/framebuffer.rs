use pbs_gl as gl;
use gl::types::*;
use std::fmt;

use crate::core::math::vector::UVec2;
use crate::core::rendering::texture::SizedTextureFormat;
use crate::core::Rectangle;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TextureFilter {
    Nearest = gl::NEAREST,
    Linear = gl::LINEAR
}

#[derive(Debug)]
pub enum FramebufferError {
    Unidentified,
    IncompleteAttachment,
    IncompleteMissingAttachment,
    IncompleteDrawBuffer,
    Unknown
}

impl fmt::Display for FramebufferError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FramebufferError::Unidentified => write!(f, "Undefined framebuffer."),
            FramebufferError::IncompleteAttachment => write!(f, "Incomplete framebuffer attachment"),
            FramebufferError::IncompleteMissingAttachment => write!(f, "Incomplete framebuffer. Add at least one attachment to the framebuffer."),
            FramebufferError::IncompleteDrawBuffer => write!(f, "Incomplete draw buffer. Check that all attachments enabled exist in the framebuffer."),
            FramebufferError::Unknown => write!(f, "Unknown framebuffer error.")
        }
    }
}

//TODO: Do I have to implement the Error trait for the error enum?!

pub struct FramebufferAttachmentCreateInfo {
    format: SizedTextureFormat,
    can_sample: bool
}

impl FramebufferAttachmentCreateInfo {
    pub fn new(format: SizedTextureFormat, can_sample: bool) -> FramebufferAttachmentCreateInfo {
        FramebufferAttachmentCreateInfo {
            format,
            can_sample
        }
    }

    pub fn get_format(&self) -> SizedTextureFormat {
        self.format
    }

    pub fn can_sample(&self) -> bool {
        self.can_sample
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FramebufferAttachment {
    id: GLuint,
    format: SizedTextureFormat
}

impl FramebufferAttachment {
    pub fn new(id: GLuint, size: UVec2, format: SizedTextureFormat) -> Self {
        FramebufferAttachment {
            id,
            format
        }
    }

    pub fn get_id(&self) -> GLuint {
        self.id
    }

    pub fn get_format(&self) -> SizedTextureFormat {
        self.format
    }
}

pub struct Framebuffer {
    id: GLuint,
    size: UVec2,
    attachments: Vec<FramebufferAttachment>,
    has_depth: bool
}

impl Framebuffer {

    pub fn new(size: UVec2, attachment_create_infos: Vec<FramebufferAttachmentCreateInfo>) -> Result<Self, FramebufferError> {
        let mut framebuffer_id: GLuint = 0;
        let mut attachment_ids: Vec<GLuint> = vec![0; attachment_create_infos.len()];
        let mut attachments: Vec<FramebufferAttachment> = vec![];
        let mut has_depth_attachment = false;

        //TODO: Check if attachments can be sampled and create Renderbuffers instead of textures if the cannot.
        unsafe {
            gl::CreateFramebuffers(1, &mut framebuffer_id);


            if !attachment_create_infos.is_empty() {

                gl::CreateTextures(gl::TEXTURE_2D,
                                   attachment_ids.len() as i32,
                                   attachment_ids.as_ptr() as *mut GLuint);

                let mut color_attachment_count = 0;
                let mut output_locations: Vec<GLuint> = vec![];

                attachment_create_infos.iter()
                                  .zip(attachment_ids.iter())
                                  .for_each(| (create_info, tex_id) |{

                                      gl::TextureStorage2D(*tex_id,
                                                           1,
                                                           create_info.get_format() as u32,
                                                           size.x as i32,
                                                           size.y as i32);

                                      let mut is_depth_attachment = Self::is_depth_stencil(create_info.get_format());

                                      if is_depth_attachment && !has_depth_attachment {
                                          gl::NamedFramebufferTexture(framebuffer_id,
                                                                      gl::DEPTH_ATTACHMENT,
                                                                      *tex_id,
                                                                      0);
                                          has_depth_attachment = true
                                      }
                                      else {
                                          output_locations.push(gl::COLOR_ATTACHMENT0 + color_attachment_count);
                                          color_attachment_count += 1;
                                          gl::NamedFramebufferTexture(framebuffer_id,
                                                                      *output_locations.last().unwrap(),
                                                                      *tex_id,
                                                                      0)
                                      }

                                      attachments.push(FramebufferAttachment::new(*tex_id,
                                                                                        size,
                                                                                       create_info.get_format()))
                                  });

                gl::NamedFramebufferDrawBuffers(framebuffer_id,
                                                output_locations.len() as i32,
                                                output_locations.as_ptr())
            }
        }

        if let Err(e) = Self::check_status(framebuffer_id) {
            Err(e)
        }
        else {
            Ok(Framebuffer {
                id: framebuffer_id,
                size,
                attachments,
                has_depth: has_depth_attachment
            })
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id)
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0)
        }
    }

    pub fn get_attachment(&self, index: usize) -> FramebufferAttachment {
        assert!(index < self.attachments.len(), "Index out of bounds.");
        self.attachments[index]
    }

    pub fn get_id(&self) -> GLuint {
        self.id
    }

    pub fn get_size(&self) -> UVec2 {
        self.size
    }

    pub fn blit(source: &Framebuffer,
                destination: &Framebuffer) {
        unsafe {
            gl::BlitNamedFramebuffer(source.get_id(),
                                     destination.get_id(),
                                     0,
                                     0,
                                     source.get_size().x as i32,
                                     source.get_size().y as i32,
                                     0,
                                     0,
                                     destination.get_size().x as i32,
                                     destination.get_size().y as i32,
                                     gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT,
                                     gl::NEAREST);
        }
    }

    fn check_status(id: GLuint) -> Result<(), FramebufferError> {
        unsafe {
            let status = gl::CheckNamedFramebufferStatus(id, gl::DRAW_FRAMEBUFFER);

            match status {
                gl::FRAMEBUFFER_UNDEFINED => Err(FramebufferError::Unidentified),
                gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => Err(FramebufferError::IncompleteAttachment),
                gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => Err(FramebufferError::IncompleteMissingAttachment),
                gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => Err(FramebufferError::IncompleteDrawBuffer),
                _ => Ok(())
            }
        }
    }

    pub fn is_depth_stencil(format: SizedTextureFormat) -> bool {
        match format {
            SizedTextureFormat::Depth16 |
            SizedTextureFormat::Depth24 |
            SizedTextureFormat::Depth32 |
            SizedTextureFormat::Depth32f |
            SizedTextureFormat::Depth24Stencil8 |
            SizedTextureFormat::Depth32fStencil8 |
            SizedTextureFormat::StencilIndex8 => true,
            _ => false
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id)
        }
    }
}

