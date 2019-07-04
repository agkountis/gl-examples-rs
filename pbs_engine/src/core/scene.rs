
pub trait Scene {
    fn setup(&mut self);
    fn update(&mut self, dt: f32);
    fn pre_draw(&mut self);
    fn draw(&mut self);
    fn post_draw(&mut self);
}

pub struct SceneManager {
    scenes: Vec<Box<dyn Scene>>,
    active_scene_index: usize
}

impl<'a> SceneManager {
    pub fn new(scenes: Vec<Box<dyn Scene>>) -> Self {
        SceneManager {
            scenes,
            active_scene_index: 0
        }
    }

    pub fn get_active_scene(&self) -> &dyn Scene {
        &*self.scenes[self.active_scene_index]
    }



}
