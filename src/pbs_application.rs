use pbs_engine::core::Settings;
use pbs_engine::core::application::{RenderingApplication, Draw, Update, Run, clear_default_framebuffer};
use pbs_engine::core::window::Window;

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

    pub fn handle_events(&mut self) {
        self.window.handle_events()
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    pub fn swap_buffers(&mut self) {
        self.window.swap_buffers()
    }
}

impl<'a> RenderingApplication for Application<'a> {
    fn run(&mut self) {
        while !self.should_close() {
            self.update(0.0); //TODO: fix timer
            self.draw();
        }
    }

    fn draw(&mut self) {
        clear_default_framebuffer(&self.get_settings().default_clear_color);

        self.swap_buffers()
    }

    fn update(&mut self, dt: f32) {
        self.handle_events()
    }
}
