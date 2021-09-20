mod pbs_scene;

use crate::pbs_scene::PbsScene;
use engine::application::Application;
use engine::Settings;

fn main() {
    Application::run(
        Settings::from_file("examples/pbs/settings.ron"),
        PbsScene::new,
    )
}
