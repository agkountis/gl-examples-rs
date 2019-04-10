use pbs_gl as gl;
use gl::types::*;

use crate::core::math::vector::{Vec2, Vec3, Vec4};
use crate::core::rendering::buffer::{Buffer, BufferStorageFlags};
use std::mem;
use std::ptr;


#[derive(Debug)]
#[repr(C)]
pub struct Vertex {
    position: Vec3,
    normal: Vec3,
    tangent: Vec3,
    tex_coord: Vec2,
    color: Vec4
}

pub struct Mesh {
    vao: GLuint,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vbo: Buffer,
    ibo: Buffer
}

impl Mesh {

    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Mesh {

        let vbo = Buffer::new_with_data(&vertices,
                                            BufferStorageFlags::DYNAMIC);

        let ibo = Buffer::new_with_data(&indices,
                                            BufferStorageFlags::DYNAMIC);

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
            gl::VertexArrayAttribFormat(vao, 0, 3, gl::FLOAT, gl::FALSE,
                                        offset_of!(Vertex, position) as u32);

            // Specify format for the normal attribute (1)
            gl::VertexArrayAttribFormat(vao, 1, 3, gl::FLOAT, gl::FALSE,
                                        offset_of!(Vertex, normal) as u32);

            // Specify format for the tangent attribute (2)
            gl::VertexArrayAttribFormat(vao, 2, 3, gl::FLOAT, gl::FALSE,
                                        offset_of!(Vertex, tangent) as u32);

            // Specify format for the texture coordinate attribute (3)
            gl::VertexArrayAttribFormat(vao, 3, 2, gl::FLOAT, gl::FALSE,
                                        offset_of!(Vertex, tex_coord) as u32);

            // Specify format for the color attribute (4)
            gl::VertexArrayAttribFormat(vao, 4, 4, gl::FLOAT, gl::FALSE,
                                        offset_of!(Vertex, color) as u32);

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
            vbo,
            ibo
        }
    }


    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);

            gl::DrawElements(gl::TRIANGLES,
                             self.indices.len() as i32,
                             gl::UNSIGNED_INT,
                             ptr::null());

            gl::BindVertexArray(0);
        }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao)
        }
    }
}

pub struct MeshUtilities;

impl MeshUtilities {

    pub fn generate_quadrilateral(dimensions: Vec3) -> Mesh {

        let half_dimensions = dimensions / 2.0;

        let vertices = vec![
            // front
            Vertex {
                position: Vec3::new(-half_dimensions.x, -half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tangent: Vec3::new(1.0, 0.0, 0.0),
                tex_coord: Vec2::new(0.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tangent: Vec3::new(1.0, 0.0, 0.0),
                tex_coord: Vec2::new(0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, -half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tangent: Vec3::new(1.0, 0.0, 0.0),
                tex_coord: Vec2::new(1.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tangent: Vec3::new(1.0, 0.0, 0.0),
                tex_coord: Vec2::new(1.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            // right
            Vertex {
                position: Vec3::new(half_dimensions.x, -half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(1.0, 0.0, 0.0),
                tangent: Vec3::new(0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(1.0, 0.0, 0.0),
                tangent: Vec3::new(0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, -half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(1.0, 0.0, 0.0),
                tangent: Vec3::new(0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(1.0, 0.0, 0.0),
                tangent: Vec3::new(0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            // left
            Vertex {
                position: Vec3::new(-half_dimensions.x, -half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(-1.0, 0.0, 0.0),
                tangent: Vec3::new(0.0, 0.0, -1.0),
                tex_coord: Vec2::new(0.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(-1.0, 0.0, 0.0),
                tangent: Vec3::new(0.0, 0.0, -1.0),
                tex_coord: Vec2::new(0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, -half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(-1.0, 0.0, 0.0),
                tangent: Vec3::new(0.0, 0.0, -1.0),
                tex_coord: Vec2::new(1.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(-1.0, 0.0, 0.0),
                tangent: Vec3::new(0.0, 0.0, -1.0),
                tex_coord: Vec2::new(1.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            // back
            Vertex {
                position: Vec3::new(half_dimensions.x, -half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, -1.0),
                tangent: Vec3::new(-1.0, 0.0, 0.0),
                tex_coord: Vec2::new(0.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, -1.0),
                tangent: Vec3::new(-1.0, 0.0, 0.0),
                tex_coord: Vec2::new(0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, -half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, -1.0),
                tangent: Vec3::new(-1.0, 0.0, 0.0),
                tex_coord: Vec2::new(1.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, 0.0, -1.0),
                tangent: Vec3::new(-1.0, 0.0, 0.0),
                tex_coord: Vec2::new(1.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            // top
            Vertex {
                position: Vec3::new(-half_dimensions.x, half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, 1.0, 0.0),
                tangent: Vec3::new(1.0, 0.0, 0.0),
                tex_coord: Vec2::new(0.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, 1.0, 0.0),
                tangent: Vec3::new(1.0, 0.0, 0.0),
                tex_coord: Vec2::new(0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, 1.0, 0.0),
                tangent: Vec3::new(1.0, 0.0, 0.0),
                tex_coord: Vec2::new(1.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, 1.0, 0.0),
                tangent: Vec3::new(1.0, 0.0, 0.0),
                tex_coord: Vec2::new(1.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            // bottom
            Vertex {
                position: Vec3::new(half_dimensions.x, -half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, -1.0, 0.0),
                tangent: Vec3::new(-1.0, 0.0, 0.0),
                tex_coord: Vec2::new(0.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(half_dimensions.x, -half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, -1.0, 0.0),
                tangent: Vec3::new(-1.0, 0.0, 0.0),
                tex_coord: Vec2::new(0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, -half_dimensions.y, half_dimensions.z),
                normal: Vec3::new(0.0, -1.0, 0.0),
                tangent: Vec3::new(-1.0, 0.0, 0.0),
                tex_coord: Vec2::new(1.0, 1.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            },
            Vertex {
                position: Vec3::new(-half_dimensions.x, -half_dimensions.y, -half_dimensions.z),
                normal: Vec3::new(0.0, -1.0, 0.0),
                tangent: Vec3::new(-1.0, 0.0, 0.0),
                tex_coord: Vec2::new(1.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0)
            }
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

            i+=6;
            j+=4;
        }

        Mesh::new(vertices, indices)
    }

    pub fn generate_cube(size: f32) -> Mesh {
        Self::generate_quadrilateral(Vec3::new(size, size, size))
    }

}
