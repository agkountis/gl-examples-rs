use engine;

mod pbs_scene;

use crate::pbs_scene::PbsScene;
use engine::application::Application;
use engine::math::vector::{UVec2, Vec4};
use engine::{Msaa, Settings, Version};

fn main() {
    Application::run(
        Settings {
            name: String::from("PBS-rs: Physically Based Shading demo using Rust"),
            asset_path: "examples/assets".into(),
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
            window_size: UVec2::new(1600, 900),
            fullscreen: false,
            msaa: Msaa::None,
            vsync: true,
            default_clear_color: Vec4::new(0.02, 0.02, 0.02, 1.0),
        },
        |context| PbsScene::new(context),
    )
}
