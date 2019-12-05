use glfw;
use pbs_gl as gl;
use gl::types::*;
use crossbeam_channel::Sender;

use std::sync::mpsc::Receiver;
use std::ptr;
use std::ffi::CStr;

use glfw::Context;
use super::{WindowMode, Msaa, Version};
use super::math::UVec2;
use crate::engine::event::Event;
use crate::engine::input::Action;


pub struct Window {
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,
    size: UVec2,
    framebuffer_size: UVec2,
    event_producer: Sender<Event>
}

impl Window {

    pub fn new(title: &str,
               size: UVec2,
               api_version: &Version,
               window_mode: &WindowMode,
               msaa: Msaa,
               event_producer: Sender<Event>) -> Window {

        assert!(api_version.major > 3 && api_version.minor > 2,
                "Only OpenGL version greater than 3.2 are supported");

        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::OpenGl));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::ContextVersion(api_version.major,
                                                               api_version.minor));
        glfw.window_hint(glfw::WindowHint::SRgbCapable(true));
        glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(glfw::WindowHint::Samples(match msaa {
                                                            Msaa::None => None,
                                                            Msaa::X4 => Some(4),
                                                            Msaa::X8 => Some(8),
                                                            Msaa::X16 => Some(16)
                                                        }));

        glfw.window_hint(glfw::WindowHint::Resizable(false));

        if cfg!(debug_assertions) {
            glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true))
        }
        else {
            glfw.window_hint(glfw::WindowHint::ContextNoError(true))
        }


        let (mut window, events) =
            glfw.with_primary_monitor(|glfw, monitor| {
                glfw.create_window(size.x,
                                   size.y,
                                   title,
                                   monitor.map_or(glfw::WindowMode::Windowed, |m| {
                                       match window_mode {
                                           WindowMode::Windowed => {
                                               glfw::WindowMode::Windowed
                                            },
                                            WindowMode::Fullscreen => {
                                               glfw::WindowMode::FullScreen(m)
                                            }
                                        }
                                   }))
            }).expect("Failed to create GLFW window.");

        gl::load_with(|s| window.get_proc_address(s) as *const _);

        window.set_key_polling(true);
        window.make_current();

        window.set_framebuffer_size_polling(true);
        let (fb_width, fb_height) = window.get_framebuffer_size();

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::MULTISAMPLE);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);

            gl::Viewport(0, 0, size.x as i32, size.y as i32);

            if cfg!(debug_assertions) {
                gl::Enable(gl::DEBUG_OUTPUT);
                gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
                gl::DebugMessageCallback(Self::debug_callback, ptr::null());
            }
        }

        Window {
            glfw,
            window,
            events,
            size,
            framebuffer_size: UVec2::new(fb_width as u32, fb_height as u32),
            event_producer
        }
    }

    pub fn handle_events(&mut self) {
        self.glfw.poll_events();

        for (_, event) in glfw::flush_messages(&self.events) {
            self.event_producer.send(event.into()).unwrap();
        }
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    pub fn swap_buffers(&mut self) {
        self.window.swap_buffers()
    }

    pub fn get_width(&self) -> u32 {
        self.size.x
    }

    pub fn get_height(&self) -> u32 {
        self.size.y
    }

    pub fn get_framebuffer_width(&self) -> u32 {
        self.framebuffer_size.x
    }

    pub fn get_framebuffer_height(&self) -> u32 {
        self.framebuffer_size.y
    }

    extern "system" fn debug_callback(source: GLenum,
                                      message_type: GLenum,
                                      id: GLuint,
                                      severity: GLenum,
                                      length: GLsizei,
                                      message: *const GLchar,
                                      user_param: *mut GLvoid) {

        let msg = unsafe { CStr::from_ptr(message) };

        eprintln!("GL CALLBACK: type = {}, severity = {}, message = {:#?}",
                  Self::message_type_to_str(message_type),
                  Self::severity_to_str(severity),
                  msg )

    }

    fn message_type_to_str(message_type: GLenum) -> &'static str {
        match message_type {
            gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEPRECATED BEHAVIOR",
            gl::DEBUG_TYPE_ERROR => "ERROR",
            gl::DEBUG_TYPE_MARKER => "MARKER",
            gl::DEBUG_TYPE_OTHER => "OTHER",
            gl::DEBUG_TYPE_PERFORMANCE => "PERFORMANCE",
            gl::DEBUG_TYPE_POP_GROUP => "POP GROUP",
            gl::DEBUG_TYPE_PORTABILITY => "PORTABILITY",
            gl::DEBUG_TYPE_PUSH_GROUP => "PUSH GROUP",
            gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "UNDEFINED BEHAVIOR",
            _ => "!UNDEFINED ENUM!"
        }
    }

    fn severity_to_str(severity: GLenum) -> &'static str {
        match severity {
            gl::DEBUG_SEVERITY_HIGH => "HIGH",
            gl::DEBUG_SEVERITY_MEDIUM => "MEDIUM",
            gl::DEBUG_SEVERITY_LOW => "LOW",
            gl::DEBUG_SEVERITY_NOTIFICATION => "NOTIFICATION",
            _ => "!UNDEFINED ENUM!"
        }
    }
}
