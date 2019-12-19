use pbs_engine;

mod pbs_application;
mod pbs_scene;

use pbs_engine::application::Application;
use pbs_engine::math::vector::{UVec2, Vec4};
use pbs_engine::{Msaa, Settings, Version, WindowMode};
use crate::pbs_scene::PbsScene;
use std::error::Error;

pub struct ApplicationData {
    pub foo: i32
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = Application::new(Settings {
        name: String::from("PBS-rs: Physically Based Shading demo using Rust"),
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
        window_size: UVec2::new(2560, 1440),
        window_mode: WindowMode::Fullscreen,
        msaa: Msaa::X4,
        vsync: true,
        default_clear_color: Vec4::new(0.02, 0.02, 0.02, 1.0),
    }, ApplicationData{foo: 10}, |context| {
        PbsScene::new(context)
    });

    app.run()
}
