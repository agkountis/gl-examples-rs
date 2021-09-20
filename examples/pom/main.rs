mod pom_scene;

use crate::pom_scene::PomScene;
use engine::application::Application;
use engine::Settings;

fn main() {
    Application::run(
        Settings::from_file("examples/pbs/settings.ron"),
        PomScene::new,
    )
}
