use crate::core::rendering::texture::Texture2D;

pub struct Material {
    pub albedo: Texture2D,
    pub metallic: Texture2D,
    pub roughness: Texture2D,
    pub normals: Texture2D,
    pub ao: Texture2D,
    pub ibl_brdf_lut: Texture2D
}
