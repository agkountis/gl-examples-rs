use pbs_gl as gl;
use crate::core::math::Vec4;
use crate::core::timer::Timer;
use crate::core::Settings;
use crate::core::asset::AssetManager;
use crate::core::scene::{SceneManager, Scene};
use crate::core::window::Window;
use crate::core::engine::{Context, LifetimeEvents};

pub struct Application {
    context: Context
}

impl Application {
    pub fn new(settings: Settings) -> Self {
        Self {
            context: Context::new(settings),
        }
    }

    pub fn add_scene<'a, T>(&mut self, f: T) where T: FnMut(&mut Context) -> Box<dyn Scene> + 'a {
        self.context.scene_manager_mut().add_scene({
            let mut scene = f(&mut self.context);
            scene.start(&mut self.context);
            scene
        })
    }

    pub fn run(&mut self) {
        self.setup();

        while !self.context.window().should_close() {
            self.context.update()
        }
    }

    fn setup(&mut self) {
        unimplemented!()
    }
}

pub fn clear_default_framebuffer(color: &Vec4) {
    unsafe {
        gl::ClearColor(color.x, color.y, color.z, color.w);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
