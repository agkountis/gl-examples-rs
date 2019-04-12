use pbs_gl as gl;
use crate::core::math::vector::Vec4;


pub trait Run {
    fn run(&mut self);
}

pub trait Update {
    fn update(&mut self, dt: f32);
}

pub trait RenderingApplication {
    fn run(&mut self);
    fn draw(&mut self);
    fn update(&mut self, dt: f32);
}

pub fn clear_default_framebuffer(color: &Vec4) {
    unsafe {
        gl::ClearColor(color.x, color.y, color.z, color.w);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

//TODO:: move this to some sort of state manager
pub fn set_vieport(x: i32, y:i32, width: i32, height: i32) {
    unsafe {
        gl::Viewport(x, y, width, height)
    }
}
