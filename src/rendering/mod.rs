pub mod buffer;
pub mod color;
pub mod device;
pub mod format;
pub mod framebuffer;
pub mod light;
pub mod material;
pub mod mesh;
pub mod postprocess;
pub mod program_pipeline;
pub mod sampler;
pub mod shader;
pub mod state;
pub mod texture;

pub trait Draw {
    fn draw(&self);
}
