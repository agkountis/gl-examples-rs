use crate::core::{math, math::Mat4, math::Vec3, math::Quat};
use std::ptr;

pub struct Entity {
    name: String,
    parent: *mut Entity,
    local_position: Vec3,
    local_rotation: Quat,
    local_scale: Vec3,
    transform: Mat4,
    children: Vec<Entity>
}

impl Entity {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            parent: ptr::null_mut(),
            local_position: Vec3::new(0.0, 0.0, 0.0),
            local_rotation: Quat::identity(),
            local_scale: Vec3::new(1.0, 1.0, 1.0),
            transform: Mat4::identity(),
            children: vec![]
        }
    }

    pub fn transform(&self) -> Mat4 {
        self.transform
    }

    pub fn local_position(&self) -> Vec3 {
        self.local_position
    }

    pub fn local_rotation(&self) -> Quat {
        self.local_rotation.clone()
    }

    pub fn set_local_position(&mut self, local_position: Vec3) {
        self.local_position = local_position
    }

    pub fn set_local_rotation(&mut self, local_rotation: Quat) {
        self.local_rotation = local_rotation
    }

    pub fn set_transform(&mut self, transform: Mat4) {
        self.transform = transform;
    }

    pub fn add_child(&mut self, mut child: Entity) {
        child.parent = self;
        self.children.push(child)
    }

    pub fn update(&mut self) {
        self.transform = Mat4::identity();
        self.transform = math::translate(&self.transform, &self.local_position);
    }
}
