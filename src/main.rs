use pbs_engine;

use pbs_engine::core::application::Application;
use pbs_engine::core::{Settings, Version, WindowMode, Msaa};
use pbs_engine::core::math::vector::UVec2;

fn main() {

    let mut app = Application::new(Settings{
        name: "PBS Demo".to_string(),
        version: Version{major: 0, minor: 1, patch: 0},
        graphics_api_version: Version{major: 4, minor:5, patch: 0},
        window_size: UVec2::new(1024, 764),
        window_mode: WindowMode::Windowed,
        msaa: Msaa::None,
        vsync: false
    });

    app.run()
}
