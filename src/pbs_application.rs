use pbs_engine::core::Settings;
use pbs_engine::core::application::{RenderingApplication, clear_default_framebuffer};
use pbs_engine::core::window::Window;
use pbs_engine::core::timer::Timer;
use pbs_engine::core::scene::Scene;
use crate::pbs_scene::PbsScene;


pub struct Application<'a> {
    window: Window,
    settings: Settings<'a>,
    timer: Timer,
    scene: Box<dyn Scene>
}

impl<'a> Application<'a> {

    pub fn new(settings: Settings) -> Application {

        let window = Window::new(&settings.name,
                                 settings.window_size,
                                 &settings.graphics_api_version,
                                 &settings.window_mode,
                                 &settings.msaa);

        let scene = Box::new(PbsScene::new(&window));

        Application {
            window,
            settings,
            timer: Timer::new(),
            scene
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

        self.setup();

        while !self.should_close() {
            let delta = self.timer.get_delta();
            self.update(delta);
            self.pre_draw();
            self.draw();
            self.post_draw();
        }
    }

    fn setup(&mut self) {
        self.scene.setup()
    }

    fn update(&mut self, dt: f32) {
        self.handle_events();
        self.scene.update(dt)
    }

    fn pre_draw(&mut self) {
        self.scene.pre_draw()
    }

    fn draw(&mut self) {
        clear_default_framebuffer(&self.get_settings().default_clear_color);
        self.scene.draw()
    }

    fn post_draw(&mut self) {
        self.scene.post_draw();
        self.swap_buffers()
    }
}
