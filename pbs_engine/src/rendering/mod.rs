#[macro_use]
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    }
}

pub mod sampler;
pub mod buffer;
pub mod mesh;
pub mod shader;
pub mod texture;
pub mod program_pipeline;
pub mod format;
pub mod state;
pub mod framebuffer;
pub mod material;
pub mod light;

pub trait Draw {
    fn draw(&self);
}
