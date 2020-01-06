use crate::core::math::{clamp_scalar, rotate_vec3};
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
    distance: f32,
    prev_distance: f32,
}

impl Default for Camera {
    fn default() -> Self {
        let position = Vec3::new(0.0, 0.0, 0.0);
        let target = Vec3::new(0.0, 0.0, 1.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let transform = math::look_at(&position, &target, &up);
        let default_distance = 40.0f32;

        Camera {
            position,
            orientation: matrix::to_rotation_quat(&transform),
            transform,
            orbit_speed: 15.0,
            zoom_speed: 30.0,
            orbit_dampening: 12.0,
            zoom_dampening: 6.0,
            yaw: 0.0,
            pitch: 0.0,
            distance: default_distance,
            prev_distance: default_distance,
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
            distance: 0.0,
            prev_distance: 0.0,
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
        self.distance = d;
        self.prev_distance = self.distance;
    }

    pub fn update(&mut self, mouse_dx: f32, mouse_dy: f32, mouse_scroll: f32, dt: f32) {
        if mouse_dx != 0.0 || mouse_dy != 0.0 {
            self.pitch += mouse_dy * self.orbit_speed * dt;
            self.yaw += mouse_dx * self.orbit_speed * dt;

            self.pitch = clamp_scalar(self.pitch, -89.99, 89.99);
        }

        if mouse_scroll != 0.0 {
            let mut scroll_amount = mouse_scroll * self.zoom_speed;
            scroll_amount *= (self.distance * 0.3);
            self.distance -= scroll_amount * dt;
        }

        self.distance =
            math::lerp_scalar(self.prev_distance, self.distance, dt * self.zoom_dampening);
        println!("distance: {}", self.distance);
        self.distance = clamp_scalar(self.distance, 2.0, 200.0);
        self.prev_distance = self.distance;

        let dest = quaternion::from_euler(self.yaw, self.pitch, 0.0);
        self.orientation = quaternion::slerp(&self.orientation, &dest, dt * self.orbit_dampening);
        self.position = rotate_vec3(&self.orientation, &Vec3::new(0.0, 0.0, self.distance));

        self.look_at(self.position, Vec3::new(0.0, 0.0, 0.0), Axes::up());
    }
}
