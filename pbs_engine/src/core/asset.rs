use std::collections::HashMap;
use crate::rendering::texture::{Texture2D, TextureCube, Texture2DLoadConfig};
use crate::rendering::mesh::Mesh;
use std::path::Path;

pub trait Asset {
    type Output;
    type Error;
    type LoadConfig;

    fn load(path: &str, load_config: Option<Self::LoadConfig>) -> Result<Self::Output, Self::Error>;
}

pub struct AssetManager {
    textures: HashMap<String, Texture2D>,
    cube_maps: HashMap<String, TextureCube>,
    meshes: HashMap<String, Mesh>
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            textures: Default::default(),
            cube_maps: Default::default(),
            meshes: Default::default()
        }
    }

    pub fn load_texture_2d(&mut self, path: &str, is_srgb: bool, generate_mipmaps: bool) -> Result<&Texture2D, String> {
        match Path::new(path).file_name() {
            Some(fname) => {
                Ok(self.textures.entry(String::from(fname.to_string_lossy())).or_insert(
                    Texture2D::load(path, Some(Texture2DLoadConfig {
                        is_srgb,
                        generate_mipmaps
                    }))?))
            },
            None => {
                Err(String::from("Invalid file path."))
            }
        }
    }

    pub fn get_texture_2d(&self, name: &str) -> Option<&Texture2D> {
        self.textures.get(name)
    }

    pub fn get_texture_2d_mut(&mut self, name: &str) -> Option<&mut Texture2D> {
        self.textures.get_mut(name)
    }


}
