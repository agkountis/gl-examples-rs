use crate::rendering::mesh::Mesh;
use crate::rendering::shader::{Shader, ShaderStage};
use crate::rendering::texture::{Texture2D, Texture2DLoadConfig, TextureCube};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::rc::Rc;

pub trait Asset {
    type Output;
    type Error;
    type LoadConfig;

    fn load<P: AsRef<Path> + Debug>(
        path: P,
        load_config: Option<Self::LoadConfig>,
    ) -> Result<Self::Output, Self::Error>;
}

#[derive(Default)]
pub struct AssetManager {
    textures: HashMap<String, Rc<Texture2D>>,
    cube_maps: HashMap<String, Rc<TextureCube>>,
    meshes: HashMap<String, Rc<Mesh>>,
    shaders: HashMap<String, Rc<Shader>>,
}

impl AssetManager {
    pub fn load_texture_2d<P: AsRef<Path>>(
        &mut self,
        path: P,
        is_srgb: bool,
        generate_mipmaps: bool,
    ) -> Result<Rc<Texture2D>, String> {
        match path.as_ref().file_name() {
            Some(fname) => {
                let texture = Rc::new(Texture2D::load(
                    path.as_ref(),
                    Some(Texture2DLoadConfig {
                        is_srgb,
                        generate_mipmap: generate_mipmaps,
                    }),
                )?);

                self.textures
                    .entry(String::from(fname.to_string_lossy()))
                    .or_insert(Rc::clone(&texture));

                Ok(texture)
            }
            None => Err(String::from("Invalid file path.")),
        }
    }

    pub fn load_texture_cube<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<Rc<TextureCube>, String> {
        match path.as_ref().file_name() {
            Some(fname) => {
                let cubemap = Rc::new(TextureCube::new_from_file(path.as_ref())?);

                self.cube_maps
                    .entry(String::from(fname.to_string_lossy()))
                    .or_insert_with(|| Rc::clone(&cubemap));

                Ok(cubemap)
            }
            None => Err(String::from("Invalid file path.")),
        }
    }

    pub fn load_mesh<P: AsRef<Path>>(&mut self, path: P) -> Result<Rc<Mesh>, String> {
        match path.as_ref().file_name() {
            Some(fname) => {
                let mesh = Rc::new(Mesh::load(path.as_ref(), None)?);

                self.meshes
                    .entry(String::from(fname.to_string_lossy()))
                    .or_insert_with(|| Rc::clone(&mesh));

                Ok(mesh)
            }
            None => Err(String::from("Invalid file path.")),
        }
    }

    pub fn load_shader<P: AsRef<Path>>(
        &mut self,
        path: P,
        stage: ShaderStage,
    ) -> Result<Rc<Shader>, String> {
        match path.as_ref().file_name() {
            Some(fname) => {
                let shader = Rc::new(Shader::load(path.as_ref(), Some(stage))?);

                self.shaders
                    .entry(String::from(fname.to_string_lossy()))
                    .or_insert_with(|| Rc::clone(&shader));

                Ok(shader)
            }
            None => Err(String::from("Invalid file path.")),
        }
    }

    pub fn get_texture_2d(&self, name: &str) -> Option<Rc<Texture2D>> {
        if let Some(rc_tex) = self.textures.get(name) {
            return Some(Rc::clone(rc_tex));
        }

        None
    }

    pub fn get_texture_cube(&self, name: &str) -> Option<Rc<TextureCube>> {
        if let Some(rc_tex) = self.cube_maps.get(name) {
            return Some(Rc::clone(rc_tex));
        }

        None
    }

    pub fn get_mesh(&self, name: &str) -> Option<Rc<Mesh>> {
        if let Some(rc_mesh) = self.meshes.get(name) {
            return Some(Rc::clone(&rc_mesh));
        }

        None
    }
}
