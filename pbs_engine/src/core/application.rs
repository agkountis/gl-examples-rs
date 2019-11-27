use pbs_gl as gl;
use crate::core::math::Vec4;
use crate::core::timer::Timer;
use crate::core::Settings;
use crate::core::asset::AssetManager;
use crate::core::scene::SceneManager;
use crate::core::window::Window;

pub struct Application<'a> {
    window: Window,
    asset_manager: AssetManager,
    scene_manager: SceneManager,
    timer: Timer,
    settings: Settings<'a>
}

impl<'a> Application<'a> {
    pub fn new(settings: Settings) -> Self {
        Self {
            window: Window::new(settings.name,
                                settings.window_size,
                                &settings.graphics_api_version,
                                &settings.window_mode,
                                &settings.msaa),
            asset_manager: AssetManager::new(),
            scene_manager: SceneManager::new(),
            timer: Timer::new(),
            settings
        }
    }
}

pub trait Run {
    fn run(&mut self);
}

pub trait Update {
    fn update(&mut self, dt: f32);
}

pub trait RenderingApplication {
    fn run(&mut self);
    fn setup(&mut self);
    fn update(&mut self, dt: f32);
    fn pre_draw(&mut self);
    fn draw(&mut self);
    fn post_draw(&mut self);
}

pub fn clear_default_framebuffer(color: &Vec4) {
    unsafe {
        gl::ClearColor(color.x, color.y, color.z, color.w);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
