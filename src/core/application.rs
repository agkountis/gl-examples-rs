use crossbeam_channel::{Receiver, Sender};
use gl_bindings as gl;

use crate::core::asset::AssetManager;
use crate::core::event::Event;
use crate::core::math::Vec4;
use crate::core::scene::{Scene, SceneManager};
use crate::core::timer::Timer;
use crate::core::window::Window;
use crate::core::Context;
use crate::core::Settings;
use std::error::Error;

pub struct Application<'a> {
    window: Window,
    scene_manager: SceneManager<'a>,
    asset_manager: AssetManager,
    timer: Timer,
    settings: Settings,
    event_consumer: Receiver<Event>,
    event_producer: Sender<Event>,
}

impl<'a> Application<'a> {
    pub fn new<Cons, S: Scene + 'a>(
        mut settings: Settings,
        mut scene_constructor: Cons,
    ) -> Application<'a>
    where
        Cons: FnMut(Context) -> S,
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
                    ),
                    event,
                )
            }

            self.scene_manager.update(Context::new(
                &mut self.window,
                &mut self.asset_manager,
                &mut self.timer,
                &mut self.settings,
            ));

            self.scene_manager.pre_draw(Context::new(
                &mut self.window,
                &mut self.asset_manager,
                &mut self.timer,
                &mut self.settings,
            ));

            self.scene_manager.draw(Context::new(
                &mut self.window,
                &mut self.asset_manager,
                &mut self.timer,
                &mut self.settings,
            ));

            self.scene_manager.post_draw(Context::new(
                &mut self.window,
                &mut self.asset_manager,
                &mut self.timer,
                &mut self.settings,
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
        ))
    }
}

pub fn clear_default_framebuffer(color: &Vec4) {
    unsafe {
        gl::ClearColor(color.x, color.y, color.z, color.w);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
