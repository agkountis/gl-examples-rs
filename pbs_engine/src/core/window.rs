use glfw;
use pbs_gl as gl;
use gl::types::*;

use std::sync::mpsc::Receiver;
use std::ptr;
use glfw::Context;
use super::{WindowMode, Msaa, Version};
use super::math::vector::UVec2;
use std::ffi::CStr;

pub struct Window {
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,
    size: UVec2
}

impl Window {

    pub fn new(title: &str,
               size: UVec2,
               api_version: &Version,
               window_mode: &WindowMode,
               msaa: &Msaa) -> Window {

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
                                                            Msaa::X16 => Some(16)
                                                        }));

        if cfg!(debug_assertions) {
            glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
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

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::MULTISAMPLE);

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
            size
        }
    }

    extern "system" fn debug_callback(source: GLenum,
                                      message_type: GLenum,
                                      id: GLuint,
                                      severity: GLenum,
                                      length: GLsizei,
                                      message: *const GLchar,
                                      user_param: *mut GLvoid) {

        let mut msg_severity = "";
        if message_type == gl::DEBUG_TYPE_ERROR {
            msg_severity = "** GL ERROR **"
        }

        let msg = unsafe { CStr::from_ptr(message) };

        eprintln!("GL CALLBACK: {} type = {:?}, severity = {:?}, message = {:#?}",
                  msg_severity, message_type, severity, msg )

    }

    pub fn handle_events(&mut self) {
        self.glfw.poll_events();

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                _ => {}
            }
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

}
