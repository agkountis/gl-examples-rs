use pbs_gl as gl;

pub struct StateManager;

impl StateManager {
    pub fn set_viewport(x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            gl::Viewport(x, y, width, height)
        }
    }
}
