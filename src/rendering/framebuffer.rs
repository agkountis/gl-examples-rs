use gl::types::*;
use gl_bindings as gl;
use std::fmt;

use crate::core::math::{UVec2, Vec4};
use crate::rendering::state::StateManager;
use crate::rendering::texture::SizedTextureFormat;
use crate::Msaa;
use std::collections::HashMap;
use std::rc::Rc;

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
    Color(GLenum, i32),
    Depth(GLenum),
    DepthStencil(GLenum),
    Stencil(GLenum),
}

impl AttachmentBindPoint {
    fn to_gl_enum(&self) -> GLenum {
        match *self {
            AttachmentBindPoint::Color(n, _) => n,
            AttachmentBindPoint::Depth(n) => n,
            AttachmentBindPoint::DepthStencil(n) => n,
            AttachmentBindPoint::Stencil(n) => n,
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

    pub fn is_depth_stencil(&self) -> bool {
        match self.attachment_bind_point {
            AttachmentBindPoint::Depth(_)
            | AttachmentBindPoint::DepthStencil(_)
            | AttachmentBindPoint::Stencil(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Framebuffer {
    id: GLuint,
    size: UVec2,
    texture_attachments: Vec<FramebufferAttachment>,
    renderbuffer_attachments: Vec<FramebufferAttachment>,
    output_locations: Vec<u32>,
    samples: u32,
    has_depth: bool,
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

            let tex_type = match msaa {
                Msaa::None => gl::TEXTURE_2D,
                _ => gl::TEXTURE_2D_MULTISAMPLE,
            };

            unsafe {
                gl::CreateTextures(
                    tex_type,
                    texture_attachment_ids.len() as i32,
                    texture_attachment_ids.as_ptr() as *mut GLuint,
                )
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
                            AttachmentBindPoint::Color(
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
            .filter(|&create_info| {
                matches!(create_info.attachment_type(), AttachmentType::Renderbuffer)
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
                            AttachmentBindPoint::Color(
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
                AttachmentBindPoint::Color(_, i) => unsafe {
                    gl::ClearNamedFramebufferfv(self.id, gl::COLOR, i, clear_color.as_ptr())
                },
                AttachmentBindPoint::Depth(_) => {
                    let depth_clear_val = 1.0f32;
                    unsafe { gl::ClearNamedFramebufferfv(self.id, gl::DEPTH, 0, &depth_clear_val) }
                }
                AttachmentBindPoint::DepthStencil(_) => unsafe {
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
                AttachmentBindPoint::Stencil(_) => unsafe {
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
        if invalidate {
            self.invalidate()
        }

        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) }
    }

    pub fn invalidate(&self) {
        if !self.renderbuffer_attachments.is_empty() {
            unsafe {
                let attachment_bind_points = self
                    .renderbuffer_attachments
                    .iter()
                    .map(|a| match a.attachment_bind_point {
                        AttachmentBindPoint::Color(n, _) => n,
                        AttachmentBindPoint::Depth(n) => n,
                        AttachmentBindPoint::DepthStencil(n) => n,
                        AttachmentBindPoint::Stencil(n) => n,
                    })
                    .collect::<Vec<_>>();

                gl::InvalidateNamedFramebufferData(
                    self.id,
                    attachment_bind_points.len() as i32,
                    attachment_bind_points.as_ptr() as *const GLenum,
                )
            }
        }
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

        let source_color_attachments = source.texture_attachments
            .iter()
            .chain(source.renderbuffer_attachments.iter())
            .filter(|&attachment| {
                !attachment.is_depth_stencil()
            }).collect::<Vec<_>>();

        let dest_color_attachments = destination.texture_attachments
            .iter()
            .chain(destination.renderbuffer_attachments.iter())
            .filter(|&attachment| !attachment.is_depth_stencil())
            .collect::<Vec<_>>();

        assert_eq!(
            source_color_attachments.len(),
            dest_color_attachments.len()
        );

        source_color_attachments
            .iter()
            .enumerate()
            .for_each(|(i, _)| unsafe {
                //TODO: Check if this is correct for depth attachments or if it works by chance.
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
            // TODO: This assumes all renderbuffers are depth textures, which is wrong.
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

            gl::NamedFramebufferReadBuffer(source.id(), gl::COLOR_ATTACHMENT0);
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
                Some(AttachmentBindPoint::Depth(gl::DEPTH_ATTACHMENT))
            }
            SizedTextureFormat::Depth24Stencil8 | SizedTextureFormat::Depth32fStencil8 => Some(
                AttachmentBindPoint::DepthStencil(gl::DEPTH_STENCIL_ATTACHMENT),
            ),
            SizedTextureFormat::StencilIndex8 => {
                Some(AttachmentBindPoint::Stencil(gl::STENCIL_ATTACHMENT))
            }
            _ => None,
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteFramebuffers(1, &self.id) }
    }
}

pub struct TemporaryFramebufferPool {
    free_framebuffers_map: HashMap<u32, Vec<(u64, bool, Rc<Framebuffer>)>>,
    keepalive_frames: u8,
    current_frame: u64,
}

impl TemporaryFramebufferPool {
    pub fn new(keepalive_frames: u8) -> Self {
        Self {
            free_framebuffers_map: Default::default(),
            keepalive_frames,
            current_frame: 0,
        }
    }

    pub fn get_temporary(
        &mut self,
        size: UVec2,
        format: SizedTextureFormat,
        depth_format: Option<SizedTextureFormat>,
    ) -> Rc<Framebuffer> {
        let key = format as u32 + depth_format.map_or(0, |value| value as u32);

        if let Some(framebuffers) = self.free_framebuffers_map.get_mut(&key) {
            if let Some((_, in_use, framebuffer)) = framebuffers
                .iter_mut()
                .find(|(_, in_use, framebuffer)| !*in_use && framebuffer.size == size)
            {
                *in_use = true;
                return Rc::clone(framebuffer);
            }

            let framebuffer = Self::create_temporary_framebuffer(size, format, depth_format);

            framebuffers.push((self.current_frame, true, Rc::clone(&framebuffer)));

            return framebuffer;
        }

        let framebuffer = Self::create_temporary_framebuffer(size, format, depth_format);

        self.free_framebuffers_map.insert(
            key,
            vec![(self.current_frame, true, Rc::clone(&framebuffer))],
        );

        framebuffer
    }

    pub(crate) fn collect(&mut self) {
        let current_frame = self.current_frame;
        let keepalive_frames = self.keepalive_frames;
        self.free_framebuffers_map
            .values_mut()
            .for_each(|framebuffers| {
                framebuffers
                    .iter_mut()
                    .filter(|(_, in_use, framebuffer)| {
                        *in_use
                            && Rc::strong_count(framebuffer) == 1
                            && Rc::weak_count(framebuffer) == 0
                    })
                    .for_each(|(last_used, in_use, _)| {
                        *in_use = false;
                        *last_used = current_frame
                    });

                // Keep only the values that satisfy the condition. Discard the rest.
                framebuffers.retain(|(last_used, in_use, _)| {
                    *in_use || (current_frame - *last_used) < keepalive_frames as u64
                });
            });

        // Retain only the entries that have allocated framebuffers. Clear the rest.
        self.free_framebuffers_map.retain(|_, v| !v.is_empty());

        self.current_frame += 1
    }

    fn create_temporary_framebuffer(
        size: UVec2,
        format: SizedTextureFormat,
        depth_format: Option<SizedTextureFormat>,
    ) -> Rc<Framebuffer> {
        let mut attachment_create_infos = vec![FramebufferAttachmentCreateInfo {
            format,
            attachment_type: AttachmentType::Texture,
        }];

        if let Some(depth_format) = depth_format {
            attachment_create_infos.push(FramebufferAttachmentCreateInfo {
                format: depth_format,
                attachment_type: AttachmentType::Texture,
            })
        }

        Rc::new(
            Framebuffer::new(size, Msaa::None, attachment_create_infos)
                .expect("Failed to create framebuffer!"),
        )
    }
}
