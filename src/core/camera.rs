use crate::core::math::{clamp_scalar, rotate_vec3};
use crate::core::{math, math::matrix, math::Axes, math::Mat4, math::Quat, math::Vec3};
use crate::math::quaternion;
use nalgebra_glm::{normalize, quat_cast, quat_normalize};

pub struct Camera {
    position: Vec3,
    orientation: Quat,
    transform: Mat4,
    orbit_speed: f32,
    zoom_speed: f32,
    orbit_dampening: f32,
    zoom_dampening: f32,
    min_distance: f32,
    max_distance: f32,
    yaw: f32,
    pitch: f32,
    distance: f32,
    prev_distance: f32,
}

impl Camera {
    pub fn new(
        position: Vec3,
        target: Vec3,
        orbit_speed: f32,
        zoom_speed: f32,
        min_distance: f32,
        max_distance: f32,
        orbit_dampening: f32,
        zoom_dampening: f32,
    ) -> Self {
        let transform = math::look_at(&position, &target, &Axes::up());

        let distance = (&position - &target).norm();

        Camera {
            position,
            orientation: matrix::to_rotation_quat(&transform),
            transform,
            orbit_speed,
            zoom_speed,
            min_distance,
            max_distance,
            orbit_dampening,
            zoom_dampening,
            yaw: 0.0,
            pitch: 0.0,
            distance,
            prev_distance: distance,
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
        const EPSILON: f32 = 0.00001;

        if mouse_dx < -EPSILON || mouse_dx > EPSILON || mouse_dy < -EPSILON || mouse_dy > EPSILON {
            self.pitch += mouse_dy * self.orbit_speed * dt;

            self.yaw += mouse_dx * self.orbit_speed * dt;

            if self.yaw < 0.0 {
                self.yaw += 360.0;
            } else if self.yaw >= 360.0 {
                self.yaw -= 360.0;
            }

            self.pitch = clamp_scalar(self.pitch, -89.99, 89.99);
        }

        if mouse_scroll != 0.0 {
            let mut scroll_amount = mouse_scroll * self.zoom_speed;
            scroll_amount *= (self.distance * 0.3);
            self.distance -= scroll_amount * dt;
        }

        self.distance =
            math::lerp_scalar(self.prev_distance, self.distance, dt * self.zoom_dampening);
        self.distance = clamp_scalar(self.distance, self.min_distance, self.max_distance);
        self.prev_distance = self.distance;

        let dest = quat_normalize(&quaternion::from_euler(self.yaw, self.pitch, 0.0));
        self.orientation = quaternion::slerp(&self.orientation, &dest, dt * self.orbit_dampening);
        let direction = normalize(&rotate_vec3(&self.orientation, &Axes::forward()));
        self.position = Vec3::new(0.0, 0.0, 0.0) - direction * self.distance;

        self.look_at(self.position, Vec3::new(0.0, 0.0, 0.0), Axes::up());
    }
}
