use std::collections::HashMap;
use crate::rendering::texture::{Texture2D, TextureCube, Texture2DLoadConfig};
use crate::rendering::mesh::Mesh;
use std::path::Path;
use std::rc::Rc;
use std::fmt::Debug;

pub trait Asset {
    type Output;
    type Error;
    type LoadConfig;

    fn load<P: AsRef<Path> + Debug>(path: P, load_config: Option<Self::LoadConfig>) -> Result<Self::Output, Self::Error>;
}

pub struct AssetManager {
    textures: HashMap<String, Rc<Texture2D>>,
    cube_maps: HashMap<String, Rc<TextureCube>>,
    meshes: HashMap<String, Rc<Mesh>>
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            textures: Default::default(),
            cube_maps: Default::default(),
            meshes: Default::default()
        }
    }

    pub fn load_texture_2d<P: AsRef<Path>>(&mut self, path: P, is_srgb: bool, generate_mipmaps: bool) -> Result<Rc<Texture2D>, String> {
        match path.as_ref().file_name() {
            Some(fname) => {
                let texture = Rc::new(Texture2D::load(path.as_ref(), Some(Texture2DLoadConfig {
                    is_srgb,
                    generate_mipmaps
                }))?);

                self.textures.entry(String::from(fname.to_string_lossy())).or_insert(Rc::clone(&texture));

                Ok(texture)
            },
            None => {
                Err(String::from("Invalid file path."))
            }
        }
    }

    pub fn load_mesh<P: AsRef<Path>>(&mut self, path: P) -> Result<Rc<Mesh>, String> {
        match path.as_ref().file_name() {
            Some(fname) => {
                let mesh = Rc::new(Mesh::load(path.as_ref(), None)?);

                self.meshes.entry(String::from(fname.to_string_lossy())).or_insert(Rc::clone(&mesh));

                Ok(mesh)
            },
            None => {
                Err(String::from("Invalid file path."))
            }
        }
    }

    pub fn get_texture_2d(&self, name: &str) -> Option<Rc<Texture2D>> {
        if let Some(rc_tex) = self.textures.get(name) {
            return Some(Rc::clone(rc_tex))
        }

        None
    }

    pub fn get_mesh(&self, name: &str) -> Option<Rc<Mesh>> {
        if let Some(rc_mesh) = self.meshes.get(name) {
            return Some(Rc::clone(&rc_mesh))
        }

        None
    }
}
