mod pbs_scene;

use crate::pbs_scene::PbsScene;
use engine::application::Application;
use engine::math::vector::{UVec2, Vec4};
use engine::{Msaa, Settings, Version};

fn main() {
    Application::run(
        Settings::from_file("examples/pbs/settings.ron"),
        PbsScene::new,
    )
}
