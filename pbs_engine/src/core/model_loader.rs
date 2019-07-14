use assimp;

use assimp::import::Importer;

struct ModelLoader;

impl ModelLoader {
    pub fn load(path: &str) -> () {
        let importer = import::Importer::new();

        let scene = importer.read_file(path);

        ()
    }
}
