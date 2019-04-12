pub mod sampler;
pub mod buffer;
pub mod mesh;
pub mod shader;
pub mod texture;
pub mod program_pipeline;
pub mod format;

pub trait Draw {
    fn draw(&self);
}
