use crate::core::engine::{LifetimeEvents, Context};
use std::borrow::BorrowMut;
use std::ops::DerefMut;

pub trait Scene: LifetimeEvents {
    fn name(&self) -> &str;
}

pub struct SceneManager {
    scenes: Vec<Box<dyn Scene>>,
    active_scene_index: usize
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            scenes: vec![],
            active_scene_index: 0
        }
    }

    pub fn new_with_scenes(scenes: Vec<Box<dyn Scene>>) -> Self {
        Self {
            scenes,
            active_scene_index: 0
        }
    }

    pub fn set_active_scene(&mut self, name: &str) -> Result<(), String> {
        if let Some(index) = self.scenes.iter().position(|scene| { scene.name() == name }) {
            self.active_scene_index = index;
            return Ok(())
        }

        Err(format!("Scene with name \"{}\" not present in the SceneManager.", name))
    }

    pub fn add_scene(&mut self, scene: Box<dyn Scene>) {
        self.scenes.push(scene)
    }

    pub fn clear_scenes(&mut self) {
        self.scenes.clear()
    }
}

impl LifetimeEvents for SceneManager {
    fn start(&mut self, context: &mut Context) {

    }

    fn update(&mut self, dt: f32) {
        self.scenes[self.active_scene_index].update(dt)
    }

    fn pre_draw(&mut self) {
        self.scenes[self.active_scene_index].pre_draw()
    }

    fn draw(&mut self) {
        self.scenes[self.active_scene_index].draw()
    }

    fn post_draw(&mut self) {
        self.scenes[self.active_scene_index].post_draw()
    }
}
