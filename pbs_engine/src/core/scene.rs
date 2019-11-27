
pub trait Scene {
    fn name(&self) -> &str;
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
