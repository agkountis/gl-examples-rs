use gl::types::*;
use gl_bindings as gl;
use std::fmt;

use crate::core::math;
use crate::core::math::{UVec2, Vec4};
use crate::rendering::state::StateManager;
use crate::rendering::texture::SizedTextureFormat;
use crate::Msaa;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TextureFilter {
    Nearest = gl::NEAREST,
    Linear = gl::LINEAR,
}

#[derive(Debug)]
pub enum FramebufferError {
    Unidentified,
    IncompleteAttachment,
    IncompleteMissingAttachment,
    IncompleteDrawBuffer,
    Unknown,
}

impl std::error::Error for FramebufferError {}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttachmentType {
    Texture,
    Renderbuffer,
    Undefined,
}

#[derive(Debug, Clone, Copy)]
enum AttachmentBindPoint {
    ColorAttachment(GLenum, i32),
    DepthAttachment(GLenum),
    DepthStencilAttachment(GLenum),
    StencilAttachment(GLenum),
}

impl AttachmentBindPoint {
    fn to_gl_enum(&self) -> GLenum {
        match *self {
            AttachmentBindPoint::ColorAttachment(n, _) => n,
            AttachmentBindPoint::DepthAttachment(n) => n,
            AttachmentBindPoint::DepthStencilAttachment(n) => n,
            AttachmentBindPoint::StencilAttachment(n) => n,
        }
    }
}

pub struct FramebufferAttachmentCreateInfo {
    format: SizedTextureFormat,
    attachment_type: AttachmentType,
}

impl FramebufferAttachmentCreateInfo {
    pub fn new(
        format: SizedTextureFormat,
        attachment_type: AttachmentType,
    ) -> FramebufferAttachmentCreateInfo {
        FramebufferAttachmentCreateInfo {
            format,
            attachment_type,
        }
    }

    pub fn format(&self) -> SizedTextureFormat {
        self.format
    }

    pub fn attachment_type(&self) -> AttachmentType {
        self.attachment_type
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FramebufferAttachment {
    id: GLuint,
    format: SizedTextureFormat,
    attachment_type: AttachmentType,
    attachment_bind_point: AttachmentBindPoint,
}

impl FramebufferAttachment {
    fn new(
        id: GLuint,
        format: SizedTextureFormat,
        attachment_type: AttachmentType,
        attachment_bind_point: AttachmentBindPoint,
    ) -> Self {
        FramebufferAttachment {
            id,
            format,
            attachment_type,
            attachment_bind_point,
        }
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn format(&self) -> SizedTextureFormat {
        self.format
    }

    pub fn attachment_type(&self) -> AttachmentType {
        self.attachment_type
    }
}

pub struct Framebuffer {
    id: GLuint,
    size: UVec2,
    texture_attachments: Vec<FramebufferAttachment>,
    renderbuffer_attachments: Vec<FramebufferAttachment>,
    output_locations: Vec<u32>,
    samples: u32,
    has_depth: bool,
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self {
            id: 0,
            size: UVec2::new(0, 0),
            texture_attachments: vec![],
            renderbuffer_attachments: vec![],
            output_locations: vec![],
            samples: Msaa::None as u32,
            has_depth: false,
        }
    }
}

impl Framebuffer {
    pub fn new(
        size: UVec2,
        msaa: Msaa,
        attachment_create_infos: Vec<FramebufferAttachmentCreateInfo>,
    ) -> Result<Self, FramebufferError> {
        let mut framebuffer_id: GLuint = 0;

        unsafe {
            gl::CreateFramebuffers(1, &mut framebuffer_id);
        }

        let mut color_attachment_count = 0;
        let mut output_locations: Vec<GLuint> = vec![];

        let mut texture_attachments: Vec<FramebufferAttachment> = vec![];
        let mut renderbuffer_attachments: Vec<FramebufferAttachment> = vec![];

        let mut has_depth_attachment = false;

        let texture_attachment_create_infos = attachment_create_infos
            .iter()
            .filter(|&create_info| create_info.attachment_type() == AttachmentType::Texture)
            .collect::<Vec<_>>();

        if !texture_attachment_create_infos.is_empty() {
            let texture_attachment_ids: Vec<GLuint> =
                vec![0; texture_attachment_create_infos.len()];

            match msaa {
                Msaa::None => unsafe {
                    gl::CreateTextures(
                        gl::TEXTURE_2D,
                        texture_attachment_ids.len() as i32,
                        texture_attachment_ids.as_ptr() as *mut GLuint,
                    )
                },
                _ => unsafe {
                    gl::CreateTextures(
                        gl::TEXTURE_2D_MULTISAMPLE,
                        texture_attachment_ids.len() as i32,
                        texture_attachment_ids.as_ptr() as *mut GLuint,
                    )
                },
            }

            texture_attachment_create_infos
                .iter()
                .zip(texture_attachment_ids.iter())
                .for_each(|(&create_info, id)| {
                    unsafe {
                        //TODO: Assert that num samples is 0 if internal format is singed or unsigned int
                        match msaa {
                            Msaa::None => gl::TextureStorage2D(
                                *id,
                                1,
                                create_info.format() as u32,
                                size.x as i32,
                                size.y as i32,
                            ),
                            _ => gl::TextureStorage2DMultisample(
                                *id,
                                msaa as i32,
                                create_info.format() as u32,
                                size.x as i32,
                                size.y as i32,
                                gl::TRUE,
                            ),
                        }
                    }

                    if let Some(attachment_bind_point) =
                        Self::is_depth_stencil_attachment(create_info.format())
                    {
                        unsafe {
                            gl::NamedFramebufferTexture(
                                framebuffer_id,
                                attachment_bind_point.to_gl_enum(),
                                *id,
                                0,
                            )
                        }

                        has_depth_attachment = true;

                        texture_attachments.push(FramebufferAttachment::new(
                            *id,
                            create_info.format(),
                            create_info.attachment_type(),
                            attachment_bind_point,
                        ))
                    } else {
                        let output_location = gl::COLOR_ATTACHMENT0 + color_attachment_count;
                        output_locations.push(output_location);

                        unsafe {
                            gl::NamedFramebufferTexture(framebuffer_id, output_location, *id, 0);
                        }

                        texture_attachments.push(FramebufferAttachment::new(
                            *id,
                            create_info.format(),
                            create_info.attachment_type(),
                            AttachmentBindPoint::ColorAttachment(
                                output_location,
                                color_attachment_count as i32,
                            ),
                        ));

                        color_attachment_count += 1
                    }
                });
        }

        let renderbuffer_attachment_create_infos = attachment_create_infos
            .iter()
            .filter(|&create_info| match create_info.attachment_type() {
                AttachmentType::Renderbuffer => true,
                _ => false,
            })
            .collect::<Vec<_>>();

        if !renderbuffer_attachment_create_infos.is_empty() {
            let renderbuffer_attachment_ids: Vec<GLuint> =
                vec![0; renderbuffer_attachment_create_infos.len()];

            unsafe {
                gl::CreateRenderbuffers(
                    renderbuffer_attachment_ids.len() as i32,
                    renderbuffer_attachment_ids.as_ptr() as *mut GLuint,
                )
            }

            renderbuffer_attachment_create_infos
                .iter()
                .zip(renderbuffer_attachment_ids.iter())
                .for_each(|(create_info, id)| {
                    unsafe {
                        match msaa {
                            Msaa::None => gl::NamedRenderbufferStorage(
                                *id,
                                create_info.format() as u32,
                                size.x as i32,
                                size.y as i32,
                            ),
                            _ => gl::NamedRenderbufferStorageMultisample(
                                *id,
                                msaa as i32,
                                create_info.format() as u32,
                                size.x as i32,
                                size.y as i32,
                            ),
                        };
                    }

                    if let Some(attachment_bind_point) =
                        Self::is_depth_stencil_attachment(create_info.format())
                    {
                        unsafe {
                            gl::NamedFramebufferRenderbuffer(
                                framebuffer_id,
                                attachment_bind_point.to_gl_enum(),
                                gl::RENDERBUFFER,
                                *id,
                            )
                        }

                        has_depth_attachment = true;

                        renderbuffer_attachments.push(FramebufferAttachment::new(
                            *id,
                            create_info.format(),
                            create_info.attachment_type(),
                            attachment_bind_point,
                        ))
                    } else {
                        let output_location = gl::COLOR_ATTACHMENT0 + color_attachment_count;
                        output_locations.push(output_location);

                        unsafe {
                            gl::NamedFramebufferRenderbuffer(
                                framebuffer_id,
                                output_location,
                                gl::RENDERBUFFER,
                                *id,
                            )
                        }

                        renderbuffer_attachments.push(FramebufferAttachment::new(
                            *id,
                            create_info.format(),
                            create_info.attachment_type(),
                            AttachmentBindPoint::ColorAttachment(
                                output_location,
                                color_attachment_count as i32,
                            ),
                        ));

                        color_attachment_count += 1
                    }
                })
        }

        unsafe {
            gl::NamedFramebufferDrawBuffers(
                framebuffer_id,
                output_locations.len() as i32,
                output_locations.as_ptr(),
            )
        }

        if let Err(e) = Self::check_status(framebuffer_id) {
            Err(e)
        } else {
            Ok(Framebuffer {
                id: framebuffer_id,
                size,
                texture_attachments,
                renderbuffer_attachments,
                output_locations,
                has_depth: has_depth_attachment,
                samples: msaa as u32,
            })
        }
    }

    pub fn clear(&self, clear_color: &Vec4) {
        self.texture_attachments
            .iter()
            .chain(self.renderbuffer_attachments.iter())
            .for_each(|attachment| match attachment.attachment_bind_point {
                AttachmentBindPoint::ColorAttachment(_, i) => unsafe {
                    gl::ClearNamedFramebufferfv(
                        self.id,
                        gl::COLOR,
                        i,
                        math::utilities::value_ptr(clear_color),
                    )
                },
                AttachmentBindPoint::DepthAttachment(_) => unsafe {
                    let depth_clear_val: f32 = 1.0;
                    gl::ClearNamedFramebufferfv(self.id, gl::DEPTH, 0, &depth_clear_val)
                },
                AttachmentBindPoint::DepthStencilAttachment(_) => unsafe {
                    let depth_clear_val: f32 = 1.0;
                    let stencil_clear_val: i32 = 0;
                    gl::ClearNamedFramebufferfi(
                        self.id,
                        gl::DEPTH_STENCIL,
                        0,
                        depth_clear_val,
                        stencil_clear_val,
                    )
                },
                AttachmentBindPoint::StencilAttachment(_) => unsafe {
                    let stencil_clear_val = 1;
                    gl::ClearNamedFramebufferiv(self.id, gl::STENCIL, 0, &stencil_clear_val)
                },
            });
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
            StateManager::set_viewport(0, 0, self.size.x as i32, self.size.y as i32);
        }
    }

    pub fn unbind(&self, invalidate: bool) {
        if !self.renderbuffer_attachments.is_empty() && invalidate {
            unsafe {
                let attachment_bind_points = self
                    .renderbuffer_attachments
                    .iter()
                    .map(|a| match a.attachment_bind_point {
                        AttachmentBindPoint::ColorAttachment(n, _) => n,
                        AttachmentBindPoint::DepthAttachment(n) => n,
                        AttachmentBindPoint::DepthStencilAttachment(n) => n,
                        AttachmentBindPoint::StencilAttachment(n) => n,
                    })
                    .collect::<Vec<_>>();

                gl::InvalidateNamedFramebufferData(
                    self.id,
                    attachment_bind_points.len() as i32,
                    attachment_bind_points.as_ptr() as *const GLenum,
                )
            }
        }

        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) }
    }

    pub fn texture_attachment(&self, index: usize) -> FramebufferAttachment {
        assert!(
            index < self.texture_attachments.len(),
            "Index out of bounds."
        );
        self.texture_attachments[index]
    }

    pub fn texture_attachments(&self) -> &Vec<FramebufferAttachment> {
        &self.texture_attachments
    }

    pub fn renderbuffer_attachments(&self) -> &Vec<FramebufferAttachment> {
        &self.renderbuffer_attachments
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn samples(&self) -> u32 {
        self.samples
    }

    pub fn blit(source: &Framebuffer, destination: &Framebuffer) {
        let source_color_attachments = &source.texture_attachments;
        let dest_color_attachments = &destination.texture_attachments;

        assert_eq!(source_color_attachments.len(), dest_color_attachments.len());

        source_color_attachments
            .iter()
            .enumerate()
            .for_each(|(i, attachment)| unsafe {
                gl::NamedFramebufferReadBuffer(source.id(), gl::COLOR_ATTACHMENT0 + i as u32);
                gl::NamedFramebufferDrawBuffer(destination.id(), gl::COLOR_ATTACHMENT0 + i as u32);

                gl::BlitNamedFramebuffer(
                    source.id(),
                    destination.id(),
                    0,
                    0,
                    source.size().x as i32,
                    source.size().y as i32,
                    0,
                    0,
                    destination.size().x as i32,
                    destination.size().y as i32,
                    gl::COLOR_BUFFER_BIT,
                    gl::NEAREST,
                );
            });

        unsafe {
            gl::BlitNamedFramebuffer(
                source.id(),
                destination.id(),
                0,
                0,
                source.size().x as i32,
                source.size().y as i32,
                0,
                0,
                destination.size().x as i32,
                destination.size().y as i32,
                gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT,
                gl::NEAREST,
            );

            gl::NamedFramebufferReadBuffer(source.id(), gl::BACK);
            gl::NamedFramebufferDrawBuffers(
                destination.id(),
                destination.output_locations.len() as i32,
                destination.output_locations.as_ptr(),
            )
        }
    }

    pub fn blit_to_default(source: &Framebuffer, default_framebuffer_size: UVec2) {
        unsafe {
            gl::BlitNamedFramebuffer(
                source.id(),
                0,
                0,
                0,
                source.size().x as i32,
                source.size().y as i32,
                0,
                0,
                default_framebuffer_size.x as i32,
                default_framebuffer_size.y as i32,
                gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT,
                gl::NEAREST,
            );
        }
    }

    fn check_status(id: GLuint) -> Result<(), FramebufferError> {
        unsafe {
            let status = gl::CheckNamedFramebufferStatus(id, gl::DRAW_FRAMEBUFFER);

            match status {
                gl::FRAMEBUFFER_UNDEFINED => Err(FramebufferError::Unidentified),
                gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => {
                    Err(FramebufferError::IncompleteAttachment)
                }
                gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
                    Err(FramebufferError::IncompleteMissingAttachment)
                }
                gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => {
                    Err(FramebufferError::IncompleteDrawBuffer)
                }
                _ => Ok(()),
            }
        }
    }

    fn is_depth_stencil_attachment(format: SizedTextureFormat) -> Option<AttachmentBindPoint> {
        match format {
            SizedTextureFormat::Depth16
            | SizedTextureFormat::Depth24
            | SizedTextureFormat::Depth32
            | SizedTextureFormat::Depth32f => {
                Some(AttachmentBindPoint::DepthAttachment(gl::DEPTH_ATTACHMENT))
            }
            SizedTextureFormat::Depth24Stencil8 | SizedTextureFormat::Depth32fStencil8 => Some(
                AttachmentBindPoint::DepthStencilAttachment(gl::DEPTH_STENCIL_ATTACHMENT),
            ),
            SizedTextureFormat::StencilIndex8 => Some(AttachmentBindPoint::StencilAttachment(
                gl::STENCIL_ATTACHMENT,
            )),
            _ => None,
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteFramebuffers(1, &self.id) }
    }
}
