use crate::core::scene::SceneManager;
use crate::core::asset::AssetManager;
use crate::core::window::Window;
use crate::core::Settings;
use crate::core::timer::Timer;

pub trait LifetimeEvents {
    fn start(&mut self, context: &mut Context) {}
    fn update(&mut self, dt: f32) {}
    fn pre_draw(&mut self) {}
    fn draw(&mut self) {}
    fn post_draw(&mut self) {}
    fn shutdown(&mut self) {}
}

pub struct Context {
    window: Window,
    scene_manager: SceneManager,
    asset_manager: AssetManager,
    timer: Timer,
    settings: Settings
}

impl Context {
    pub fn new(settings: Settings) -> Self {
        Self {
            window: Window::new(&settings.name,
                                settings.window_size,
                                &settings.graphics_api_version,
                                &settings.window_mode,
                                &settings.msaa),
            scene_manager: SceneManager::new(),
            asset_manager: AssetManager::new(),
            timer: Timer::new(),
            settings
        }
    }

    pub(crate) fn update(&mut self) {
        self.scene_manager.update(self.timer.get_delta())
    }

    pub fn scene_manager_mut(&mut self) -> &mut SceneManager{
        &mut self.scene_manager
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}
