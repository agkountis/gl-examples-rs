use pbs_gl as gl;
use gl::types::GLuint;

use crate::core::math::vector::{Vec2, Vec3, Vec4};
use crate::core::rendering::buffer::{ Buffer, BufferStorageFlags };
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
    indices: Vec<u16>,
    vbo: Buffer,
    ibo: Buffer
}

impl Mesh {

    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Mesh {

        let v: &[u8] = unsafe{ mem::transmute::<&[Vertex], &[u8]>(vertices.as_slice()) };
        let i: &[u8] = unsafe{ mem::transmute::<&[u16], &[u8]>(indices.as_slice()) };

        let vbo = Buffer::new_with_data(v,
                                        BufferStorageFlags::MAP_READ_WRITE |
                                            BufferStorageFlags::MAP_PERSISTENT |
                                            BufferStorageFlags::MAP_COHERENT);

        let ibo = Buffer::new_with_data(i,
                                        BufferStorageFlags::MAP_READ_WRITE |
                                            BufferStorageFlags::MAP_PERSISTENT |
                                            BufferStorageFlags::MAP_COHERENT);

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
                             gl::UNSIGNED_SHORT,
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
