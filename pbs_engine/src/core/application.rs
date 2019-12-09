use pbs_gl as gl;
use crossbeam_channel::{Sender, Receiver};

use crate::core::math::Vec4;
use crate::core::timer::Timer;
use crate::core::Settings;
use crate::core::asset::AssetManager;
use crate::core::scene::{SceneManager, Scene};
use crate::core::window::Window;
use crate::core::engine::Context;
use std::error::Error;
use crate::engine::event::Event;

pub struct Application<T> {
    window: Window,
    scene_manager: SceneManager<T>,
    asset_manager: AssetManager,
    timer: Timer,
    settings: Settings,
    event_consumer: Receiver<Event>,
    event_producer: Sender<Event>,
    user_data: T
}

impl<T> Application<T> {
    pub fn new<Cons>(mut settings: Settings, mut user_data: T, mut scene_constructor: Cons) -> Self
        where Cons: FnMut(Context<T>) -> Box<dyn Scene<T>> {

        let (producer, consumer) = crossbeam_channel::unbounded();

        let mut window = Window::new(&settings.name,
                                 settings.window_size,
                                 &settings.graphics_api_version,
                                 &settings.window_mode,
                                 settings.msaa,
                                 producer.clone());

        let mut asset_manager = AssetManager::new();
        let mut timer = Timer::new();

        let initial_scene = scene_constructor(Context::new(&mut window, &mut asset_manager, &mut timer, &mut settings, &mut user_data));
        let mut scene_manager = SceneManager::new(initial_scene);

        Self {
            window,
            scene_manager,
            asset_manager,
            timer,
            settings,
            event_consumer: consumer,
            event_producer: producer,
            user_data
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.initialize();

        while !self.window.should_close() && self.scene_manager.is_running() {
            self.window.handle_events();

            for event in self.event_consumer.try_iter() {
                self.scene_manager.handle_event(Context::new(&mut self.window,
                                                             &mut self.asset_manager,
                                                             &mut self.timer,
                                                             &mut self.settings,
                                                             &mut self.user_data),
                                                event)
            }

            self.scene_manager.update(Context::new(&mut self.window,
                                                   &mut self.asset_manager,
                                                   &mut self.timer,
                                                   &mut self.settings,
                                                   &mut self.user_data));

            self.window.swap_buffers()
        }

        Ok(())
    }

    fn initialize(&mut self) {
        self.scene_manager.initialize(Context::new(&mut self.window,
                                                   &mut self.asset_manager,
                                                   &mut self.timer,
                                                   &mut self.settings,
                                                   &mut self.user_data))
    }
}

pub fn clear_default_framebuffer(color: &Vec4) {
    unsafe {
        gl::ClearColor(color.x, color.y, color.z, color.w);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
