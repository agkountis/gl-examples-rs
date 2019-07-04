use crate::core::math::vector::Vec3;
use crate::core::math::matrix::{Mat4, look_at};

pub struct Camera {
    position: Vec3,
    target: Vec3,
    up: Vec3,
    transform: Mat4
}

impl Camera {
    pub fn new() -> Self {
        let position = Vec3::new(0.0, 0.0, 0.0);
        let target = Vec3::new(0.0, 0.0, 1.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let transform = look_at(&position, &target, &up);

        Camera {
            position,
            target,
            up,
            transform
        }
    }

    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn get_target(&self) -> &Vec3 {
        &self.target
    }

    pub fn get_transform(&self) -> &Mat4 {
        &self.transform
    }

    pub fn look_at(&mut self, position: Vec3, target: Vec3, up: Vec3) {
        self.position = position;
        self.target = target;
        self.up = up;
        self.transform = look_at(&position, &target, &up)
    }

    pub fn set_transform(&mut self, transform: Mat4) {
        self.transform = transform
    }
}
