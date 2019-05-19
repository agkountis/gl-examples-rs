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
    fn setup(&mut self);
    fn update(&mut self, dt: f32);
    fn pre_draw(&mut self);
    fn draw(&mut self);
    fn post_draw(&mut self);
}

pub fn clear_default_framebuffer(color: &Vec4) {
    unsafe {
        gl::ClearColor(color.x, color.y, color.z, color.w);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}
