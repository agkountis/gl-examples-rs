use pbs_engine;

pub mod pbs_application;

use pbs_application::Application;

use pbs_engine::core::{Settings, Version, WindowMode, Msaa};
use pbs_engine::core::math::vector::{Vec4, UVec2};
use pbs_engine::core::rendering::shader::{Shader, ShaderType};
use pbs_engine::core::rendering::program_pipeline::ProgramPipeline;
use pbs_engine::core::application::RenderingApplication;


fn main() {

    let mut app = Application::new(Settings{
        name: "PBS Demo",
        version: Version{major: 0, minor: 1, patch: 0},
        graphics_api_version: Version{major: 4, minor: 5, patch: 0},
        window_size: UVec2::new(1024, 764),
        window_mode: WindowMode::Windowed,
        msaa: Msaa::None,
        vsync: false,
        default_clear_color: Vec4::new(1.0, 0.0, 0.0, 0.0)
    });

//    let vertex_shader = Shader::new(ShaderType::Vertex,
//                                    "main",
//                                    "sdr/pbs.vert.spv").unwrap();
//
//    let fragment_shader = Shader::new(ShaderType::Fragment,
//                                      "main",
//                                      "sdr/pbs.frag.spv").unwrap();
//
//    let pipeline = ProgramPipeline::new().add_shader(&vertex_shader)
//                                         .add_shader(&fragment_shader)
//                                         .build().unwrap();

    app.run()
}
