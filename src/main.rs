use pbs_engine;

pub mod pbs_application;

use pbs_application::Application;

use pbs_engine::core::application::RenderingApplication;
use pbs_engine::core::math::vector::{UVec2, Vec4};
use pbs_engine::core::{Msaa, Settings, Version, WindowMode};

fn main() {
    let mut app = Application::new(Settings {
        name: "PBS-rs: Physically Based Shading demo using Rust",
        version: Version {
            major: 0,
            minor: 1,
            patch: 0,
        },
        graphics_api_version: Version {
            major: 4,
            minor: 5,
            patch: 0,
        },
        window_size: UVec2::new(1920, 1080),
        window_mode: WindowMode::Windowed,
        msaa: Msaa::X4,
        vsync: true,
        default_clear_color: Vec4::new(0.02, 0.02, 0.02, 1.0),
    });

    app.run()
}
