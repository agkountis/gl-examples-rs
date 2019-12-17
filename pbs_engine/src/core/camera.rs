use crate::core::math::{clamp_scalar, inverse, quat_normalize, rotate_vec3, to_mat4, translate};
use crate::core::{math, math::matrix, math::Axes, math::Mat4, math::Quat, math::Vec3};
use crate::math::quaternion;

pub struct Camera {
    position: Vec3,
    orientation: Quat,
    transform: Mat4,
    orbit_speed: f32,
    zoom_speed: f32,
    orbit_dampening: f32,
    zoom_dampening: f32,
    yaw: f32,
    pitch: f32,
}

impl Default for Camera {
    fn default() -> Self {
        let position = Vec3::new(0.0, 0.0, 0.0);
        let target = Vec3::new(0.0, 0.0, 1.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let transform = math::look_at(&position, &target, &up);

        Camera {
            position,
            orientation: matrix::to_rotation_quat(&transform),
            transform,
            orbit_speed: 6.0,
            zoom_speed: 0.5,
            orbit_dampening: 2.0,
            zoom_dampening: 2.0,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl Camera {
    pub fn new(
        position: Vec3,
        target: Vec3,
        orbit_speed: f32,
        zoom_speed: f32,
        orbit_dampening: f32,
        zoom_dampening: f32,
    ) -> Self {
        let transform = math::look_at(&position, &target, &Axes::up());

        Camera {
            position,
            orientation: matrix::to_rotation_quat(&transform),
            transform,
            orbit_speed: 6.0,
            zoom_speed: 2.0,
            orbit_dampening: 2.0,
            zoom_dampening: 2.0,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn get_transform(&self) -> &Mat4 {
        &self.transform
    }

    pub fn look_at(&mut self, position: Vec3, target: Vec3, up: Vec3) {
        self.transform = math::look_at(&position, &target, &up)
    }

    pub fn set_transform(&mut self, transform: Mat4) {
        self.transform = transform
    }

    pub fn set_distance(&mut self, d: f32) {
        self.position.z = d;
    }

    pub fn update(&mut self, mouse_dx: f32, mouse_dy: f32, mouse_scroll: f32, dt: f32) {
        println!("{} {}", self.yaw, self.pitch);
        if mouse_dx != 0.0 || mouse_dy != 0.0 {
            self.pitch += mouse_dy * self.orbit_speed * dt;
            self.yaw += mouse_dx * self.orbit_speed * dt;

            self.pitch = clamp_scalar(self.pitch, -90.0, 90.0);
        }

        if mouse_scroll != 0.0 {
            let mut scroll_amount = mouse_scroll * self.zoom_speed;
            scroll_amount *= self.position.z * 0.3;
            self.position.z += scroll_amount * -1.0;
            self.position.z = clamp_scalar(self.position.z, 1.5, 50.0)
        }

        let dest = quaternion::from_euler(self.yaw, self.pitch, 0.0);
        self.orientation = quaternion::slerp(&self.orientation, &dest, dt * self.orbit_dampening);
        self.position = rotate_vec3(&dest, &self.position);

        self.look_at(self.position, Vec3::new(0.0, 0.0, 0.5), Axes::up());

        self.pitch = 0.0;
        self.yaw = 0.0;
    }
}
