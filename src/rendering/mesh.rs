use gl::types::*;
use gl_bindings as gl;

use crate::{
    core::{
        asset::Asset,
        math::{Vec2, Vec3, Vec4},
    },
    rendering::{
        buffer::{Buffer, BufferStorageFlags, BufferTarget},
        Draw,
    },
};
use std::{mem, path::Path, ptr};

lazy_static! {
    pub static ref FULLSCREEN_MESH: FullscreenMesh = FullscreenMesh::new();
}

#[derive(Debug)]
#[repr(C)]
pub struct Vertex {
    position: Vec3,
    normal: Vec3,
    tangent: Vec4,
    tex_coord: Vec2,
    color: Vec4,
}

pub struct Mesh {
    vao: GLuint,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    _vbo: Buffer,
    _ibo: Buffer,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Mesh {
        //TODO: Check if dynamic buffer storage is needed here.
        println!(
            "Creating mesh with {} vertices and {} indices.",
            vertices.len(),
            indices.len()
        );
        let vbo = Buffer::new_from_slice(
            "Vertex Buffer",
            &vertices,
            BufferTarget::Array,
            BufferStorageFlags::NONE,
        );

        let ibo = Buffer::new_from_slice(
            "Index Buffer",
            &indices,
            BufferTarget::ElementArray,
            BufferStorageFlags::NONE,
        );

        let mut vao: GLuint = 0;
        unsafe {
            gl::CreateVertexArrays(1, &mut vao);

            gl::VertexArrayVertexBuffer(vao, 0, vbo.get_id(), 0, mem::size_of::<Vertex>() as i32);
            gl::VertexArrayElementBuffer(vao, ibo.get_id());

            gl::EnableVertexArrayAttrib(vao, 0); //positions
            gl::EnableVertexArrayAttrib(vao, 1); //normals
            gl::EnableVertexArrayAttrib(vao, 2); //tangents
            gl::EnableVertexArrayAttrib(vao, 3); //texture coordinates
            gl::EnableVertexArrayAttrib(vao, 4); //colors

            // Specify format for the position attribute (0)
            gl::VertexArrayAttribFormat(
                vao,
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                offset_of!(Vertex, position) as u32,
            );

            // Specify format for the normal attribute (1)
            gl::VertexArrayAttribFormat(
                vao,
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                offset_of!(Vertex, normal) as u32,
            );

            // Specify format for the tangent attribute (2)
            gl::VertexArrayAttribFormat(
                vao,
                2,
                4,
                gl::FLOAT,
                gl::FALSE,
                offset_of!(Vertex, tangent) as u32,
            );

            // Specify format for the texture coordinate attribute (3)
            gl::VertexArrayAttribFormat(
                vao,
                3,
                2,
                gl::FLOAT,
                gl::FALSE,
                offset_of!(Vertex, tex_coord) as u32,
            );

            // Specify format for the color attribute (4)
            gl::VertexArrayAttribFormat(
                vao,
                4,
                4,
                gl::FLOAT,
                gl::FALSE,
                offset_of!(Vertex, color) as u32,
            );

            // Associate attribute bindings with the VBO binding in the VAO.
            // This VAO has only 1 VBO so it is located in binding 0.
            gl::VertexArrayAttribBinding(vao, 0, 0);
            gl::VertexArrayAttribBinding(vao, 1, 0);
            gl::VertexArrayAttribBinding(vao, 2, 0);
            gl::VertexArrayAttribBinding(vao, 3, 0);
            gl::VertexArrayAttribBinding(vao, 4, 0);
        }

        Mesh {
            vao,
            vertices,
            indices,
            _vbo: vbo,
            _ibo: ibo,
        }
    }
}

impl Draw for Mesh {
    fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);

            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );

            gl::BindVertexArray(0);
        }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, &self.vao) }
    }
}

impl Asset for Mesh {
    type Output = Self;
    type Error = String;
    type LoadConfig = ();

    fn load<P: AsRef<Path>>(
        path: P,
        _: Option<Self::LoadConfig>,
    ) -> Result<Self::Output, Self::Error> {
        use gltf::buffer;

        if let Ok((document, buffers, _)) = gltf::import(path) {
            let scene = document
                .scenes()
                .next()
                .expect("Gltf document has no scenes");
            let node = scene.nodes().next().expect("Gltf scene has no nodes");
            let mesh = node.mesh().expect("Gltf node has no mesh");
            let primitive = mesh.primitives().next().unwrap();
            let reader = primitive.reader(|buffer| {
                buffers
                    .get(buffer.index())
                    .and_then(|buffer::Data(data)| data.as_slice().into())
            });

            let positions = reader.read_positions().expect("Mesh has no positions");
            let normals = reader.read_normals().expect("Mesh has no normals");
            let tangents = reader.read_tangents().expect("Mesh has no tangents");
            let tex_coords = reader
                .read_tex_coords(0)
                .expect("Mesh has no UV channel 0.");

            let vertices = positions
                .zip(normals)
                .zip(tangents)
                .zip(tex_coords.into_f32())
                .map(|(((v, n), t), tc)| Vertex {
                    position: Vec3::new(v[0], v[1], v[2]),
                    normal: Vec3::new(n[0], n[1], n[2]),
                    tangent: t.into(),
                    tex_coord: Vec2::new(tc[0], tc[1]),
                    color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                })
                .collect::<Vec<_>>();

            let indices = reader
                .read_indices()
                .expect("Mesh has no indices.")
                .into_u32()
                .map(|index| index)
                .collect::<Vec<_>>();

            Ok(Mesh::new(vertices, indices))
        } else {
            Err("Failed to load Gltf file".to_string())
        }
    }
}

pub struct FullscreenMesh {
    vao: GLuint,
}

impl FullscreenMesh {
    pub fn new() -> Self {
        let mut vao: GLuint = 0;

        unsafe { gl::CreateVertexArrays(1, &mut vao) }

        FullscreenMesh { vao }
    }
}

impl Draw for FullscreenMesh {
    fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
        }
    }
}

pub struct MeshUtilities;

impl MeshUtilities {
    pub fn generate_quadrilateral(dimensions: Vec3) -> Mesh {
        let half_dimensions = dimensions * 0.5;

        let vertices = vec![
            // front
            Vertex {
                position: Vec3::new(-half_dimensions.x, -half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tangent: Vec4::new(1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tangent: Vec4::new(1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, -half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tangent: Vec4::new(1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tangent: Vec4::new(1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            // right
            Vertex {
                position: Vec3::new(half_dimensions.x, -half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(1.0, 0.0, 0.0),
                tangent: Vec4::new(0.0, 0.0, -1.0, 1.0),
                tex_coord: Vec2::new(0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(1.0, 0.0, 0.0),
                tangent: Vec4::new(0.0, 0.0, -1.0, 1.0),
                tex_coord: Vec2::new(0.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, -half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(1.0, 0.0, 0.0),
                tangent: Vec4::new(0.0, 0.0, -1.0, 1.0),
                tex_coord: Vec2::new(1.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(1.0, 0.0, 0.0),
                tangent: Vec4::new(0.0, 0.0, -1.0, 1.0),
                tex_coord: Vec2::new(1.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            // left
            Vertex {
                position: Vec3::new(-half_dimensions.x, -half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(-1.0, 0.0, 0.0),
                tangent: Vec4::new(0.0, 0.0, 1.0, 1.0),
                tex_coord: Vec2::new(0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(-1.0, 0.0, 0.0),
                tangent: Vec4::new(0.0, 0.0, 1.0, 1.0),
                tex_coord: Vec2::new(0.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, -half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(-1.0, 0.0, 0.0),
                tangent: Vec4::new(0.0, 0.0, 1.0, 1.0),
                tex_coord: Vec2::new(1.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(-1.0, 0.0, 0.0),
                tangent: Vec4::new(0.0, 0.0, 1.0, 1.0),
                tex_coord: Vec2::new(1.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            // back
            Vertex {
                position: Vec3::new(half_dimensions.x, -half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, -1.0),
                tangent: Vec4::new(-1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, -1.0),
                tangent: Vec4::new(-1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, -half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, -1.0),
                tangent: Vec4::new(-1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, -1.0),
                tangent: Vec4::new(-1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            // top
            Vertex {
                position: Vec3::new(-half_dimensions.x, half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, 1.0, 0.0),
                tangent: Vec4::new(1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, 1.0, 0.0),
                tangent: Vec4::new(1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, 1.0, 0.0),
                tangent: Vec4::new(1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, 1.0, 0.0),
                tangent: Vec4::new(1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            // bottom
            Vertex {
                position: Vec3::new(half_dimensions.x, -half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, -1.0, 0.0),
                tangent: Vec4::new(-1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, -half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, -1.0, 0.0),
                tangent: Vec4::new(-1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, -half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, -1.0, 0.0),
                tangent: Vec4::new(-1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, -half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, -1.0, 0.0),
                tangent: Vec4::new(-1.0, 0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
        ];

        let index_count = vertices.len() / 2 * 3;

        let mut indices = vec![];
        indices.resize(index_count, 0);

        let mut i = 0;
        let mut j = 0;
        while i < indices.len() {
            indices[i] = j;

            indices[i + 1] = j + 2;
            indices[i + 4] = j + 2;

            indices[i + 2] = j + 1;
            indices[i + 3] = j + 1;

            indices[i + 5] = j + 3;

            i += 6;
            j += 4;
        }

        Mesh::new(vertices, indices)
    }

    pub fn generate_cube(size: f32) -> Mesh {
        Self::generate_quadrilateral(Vec3::new(size, size, size))
    }
}
