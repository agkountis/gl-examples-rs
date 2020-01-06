use crossbeam_channel::{Receiver, Sender};
use pbs_gl as gl;

use crate::core::asset::AssetManager;
use crate::core::engine::Context;
use crate::core::math::Vec4;
use crate::core::scene::{Scene, SceneManager};
use crate::core::timer::Timer;
use crate::core::window::Window;
use crate::core::Settings;
use crate::engine::event::Event;
use std::error::Error;

pub struct Application<'a, T: 'a> {
    window: Window,
    scene_manager: SceneManager<'a, T>,
    asset_manager: AssetManager,
    timer: Timer,
    settings: Settings,
    event_consumer: Receiver<Event>,
    event_producer: Sender<Event>,
    user_data: T,
}

impl<'a, T> Application<'a, T> {
    pub fn new<Cons, S: Scene<T> + 'a>(
        mut settings: Settings,
        mut user_data: T,
        mut scene_constructor: Cons,
    ) -> Application<'a, T>
    where
        Cons: FnMut(Context<T>) -> S,
    {
        let (producer, consumer) = crossbeam_channel::unbounded();

        let mut window = Window::new(
            &settings.name,
            settings.window_size,
            &settings.graphics_api_version,
            &settings.window_mode,
            settings.msaa,
            producer.clone(),
        );

        let mut asset_manager = AssetManager::new();
        let mut timer = Timer::new();

        let initial_scene = scene_constructor(Context::new(
            &mut window,
            &mut asset_manager,
            &mut timer,
            &mut settings,
            &mut user_data,
        ));
        let scene_manager = SceneManager::new(initial_scene);

        Self {
            window,
            scene_manager,
            asset_manager,
            timer,
            settings,
            event_consumer: consumer,
            event_producer: producer,
            user_data,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.initialize();

        while !self.window.should_close() && self.scene_manager.is_running() {
            self.window.handle_events();

            for event in self.event_consumer.try_iter() {
                self.scene_manager.handle_event(
                    Context::new(
                        &mut self.window,
                        &mut self.asset_manager,
                        &mut self.timer,
                        &mut self.settings,
                        &mut self.user_data,
                    ),
                    event,
                )
            }

            self.scene_manager.update(Context::new(
                &mut self.window,
                &mut self.asset_manager,
                &mut self.timer,
                &mut self.settings,
                &mut self.user_data,
            ));

            self.scene_manager.pre_draw(Context::new(
                &mut self.window,
                &mut self.asset_manager,
                &mut self.timer,
                &mut self.settings,
                &mut self.user_data,
            ));

            self.scene_manager.draw(Context::new(
                &mut self.window,
                &mut self.asset_manager,
                &mut self.timer,
                &mut self.settings,
                &mut self.user_data,
            ));

            self.scene_manager.post_draw(Context::new(
                &mut self.window,
                &mut self.asset_manager,
                &mut self.timer,
                &mut self.settings,
                &mut self.user_data,
            ));

            self.window.swap_buffers()
        }

        Ok(())
    }

    fn initialize(&mut self) {
        self.scene_manager.initialize(Context::new(
            &mut self.window,
            &mut self.asset_manager,
            &mut self.timer,
            &mut self.settings,
            &mut self.user_data,
        ))
    }
}

pub fn clear_default_framebuffer(color: &Vec4) {
    unsafe {
        gl::ClearColor(color.x, color.y, color.z, color.w);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
