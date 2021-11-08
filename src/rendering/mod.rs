#[macro_use]
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    };
}

pub mod buffer;
pub mod color;
pub mod device;
pub mod format;
pub mod framebuffer;
pub mod light;
pub mod material;
pub mod mesh;
pub mod postprocess;
pub mod sampler;
pub mod shader;
pub mod state;
pub mod texture;

pub trait Draw {
    fn draw(&self);
}
