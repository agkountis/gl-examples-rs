use glfw;
use pbs_gl as gl;

use std::sync::mpsc::Receiver;
use glfw::Context;
use super::{WindowMode, Msaa, Version};
use super::math::vector::UVec2;

pub struct Window {
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>
}

impl Window {

    pub fn new(title: &String, size: UVec2,
               api_version: &Version, window_mode: &WindowMode, msaa: &Msaa) -> Window {

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


        let (mut window, events) = glfw.with_primary_monitor(|glfw, monitor| {
            glfw.create_window(size.x,
                               size.y,
                               title.as_str(),
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

        Window {
            glfw,
            window,
            events
        }
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

}
