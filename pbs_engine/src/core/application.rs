use super::{ Settings, window::Window };
use pbs_gl as gl;

pub struct Application<'a> {
    window: Window,
    settings: Settings<'a>
}

impl<'a> Application<'a> {

    pub fn new(settings: Settings) -> Application {
        Application {
            window: Window::new(&settings.name,
                                settings.window_size,
                                &settings.graphics_api_version,
                                &settings.window_mode,
                                &settings.msaa),
            settings
        }
    }

    pub fn run(&mut self) {
        while !self.window.should_close() {
            self.window.handle_events();
            self.draw();
            self.window.swap_buffers();
        }
    }

    fn draw(&self) {
        unsafe {
            gl::ClearColor(1.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

}
