
pub trait Scene {
    fn setup(&mut self);
    fn update(&mut self, dt: f32);
    fn pre_draw(&mut self);
    fn draw(&mut self);
    fn post_draw(&mut self);
}

pub struct SceneManager {
    scenes: Vec<Box<dyn Scene>>
}
