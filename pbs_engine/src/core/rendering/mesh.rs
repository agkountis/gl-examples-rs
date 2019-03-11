use crate::core::math::vector::Vec3;

#[derive(Debug)]
#[repr(C)]
pub struct Vertex {
    position: Vec3,
    normal: Vec3,
    tangent: Vec3
}

pub struct Mesh {
    vertices : Vec<Vertex>,
    indices : Vec<u16>
}

impl Mesh {

    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Mesh {
        Mesh {
            vertices,
            indices
        }
    }

    pub fn set_vertices(&mut self, vertices: Vec<Vertex>) {
        self.vertices = vertices
    }

    pub fn set_indices(&mut self, indices: Vec<u16>) {
        self.indices = indices
    }

    pub fn recalculate_normals(&mut self) {
        //TODO
    }
}
