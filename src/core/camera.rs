use std::ops::RangeInclusive;

use glutin::dpi::PhysicalSize;
use nalgebra_glm::{normalize, quat_normalize};

use crate::core::math::{clamp_scalar, rotate_vec3, Vec4};
use crate::core::{math, math::matrix, math::Axes, math::Mat4, math::Quat, math::Vec3};
use crate::imgui::{im_str, Gui, Ui};
use crate::math::{perspective, quaternion};
use crate::rendering::buffer::{Buffer, BufferStorageFlags, BufferTarget, MapModeFlags};

#[repr(C)]
struct CameraUniformBlock {
    view: Mat4,
    projection: Mat4,
    view_projection_matrix: Mat4,
    eye_position: Vec4,
}

pub struct Camera {
    position: Vec3,
    orientation: Quat,
    transform: Mat4,
    fov_deg: u32,
    near_plane: f32,
    far_plane: f32,
    aperture: f32,
    shutter_speed: f32,
    sensitivity: f32,
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
    uniform_buffer: Buffer,
}

impl Camera {
    pub fn position(&self) -> &Vec3 {
        &self.position
    }

    pub fn transform(&self) -> &Mat4 {
        &self.transform
    }

    pub fn orbit_speed(&self) -> f32 {
        self.orbit_speed
    }

    pub fn zoom_speed(&self) -> f32 {
        self.zoom_speed
    }

    pub fn orbit_dampening(&self) -> f32 {
        self.orbit_dampening
    }

    pub fn zoom_dampening(&self) -> f32 {
        self.zoom_dampening
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

    pub fn set_orbit_speed(&mut self, orbit_speed: f32) {
        self.orbit_speed = orbit_speed
    }

    pub fn set_zoom_speed(&mut self, zoom_speed: f32) {
        self.zoom_speed = zoom_speed
    }

    pub fn set_orbit_dampening(&mut self, orbit_dampening: f32) {
        self.orbit_dampening = orbit_dampening
    }

    pub fn set_zoom_dampening(&mut self, zoom_dampening: f32) {
        self.zoom_dampening = zoom_dampening
    }

    pub fn update(
        &mut self,
        window_size: PhysicalSize<u32>,
        mouse_dx: f32,
        mouse_dy: f32,
        mouse_scroll: f32,
        dt: f32,
    ) {
        self.pitch += mouse_dy * self.orbit_speed * dt;

        self.yaw += mouse_dx * self.orbit_speed * dt;

        if self.yaw < 0.0 {
            self.yaw += 360.0;
        } else if self.yaw >= 360.0 {
            self.yaw -= 360.0;
        }

        self.pitch = clamp_scalar(self.pitch, -89.99, 89.99);

        let mut scroll_amount = mouse_scroll * self.zoom_speed;
        scroll_amount *= self.distance * 0.3;
        self.distance -= scroll_amount * dt;

        self.distance =
            math::lerp_scalar(self.prev_distance, self.distance, dt * self.zoom_dampening);
        self.distance = clamp_scalar(self.distance, self.min_distance, self.max_distance);
        self.prev_distance = self.distance;

        let dest = quat_normalize(&quaternion::from_euler(self.yaw, self.pitch, 0.0));
        self.orientation = quaternion::slerp(&self.orientation, &dest, dt * self.orbit_dampening);
        let direction = normalize(&rotate_vec3(&self.orientation, &Axes::forward()));
        self.position = Vec3::new(0.0, 0.0, 0.0) - direction * self.distance;

        self.look_at(self.position, Vec3::new(0.0, 0.0, 0.0), Axes::up());

        let projection = perspective(
            window_size.width,
            window_size.height,
            self.fov_deg,
            self.near_plane,
            self.far_plane,
        );

        let block = CameraUniformBlock {
            view: self.transform,
            projection,
            view_projection_matrix: projection * self.transform,
            eye_position: Vec4::new(self.position.x, self.position.y, self.position.z, 1.0),
        };

        self.uniform_buffer.fill_mapped(0, &block)
    }

    pub fn exposure(&self) -> f32 {
        let ev100 = f32::log2(
            self.aperture * self.aperture / self.shutter_speed * 100.0 / self.sensitivity,
        );

        1.0 / 2.0f32.powf(ev100) * 1.2
    }
}

pub struct CameraBuilder {
    position: Vec3,
    target: Vec3,
    fov_deg: u32,
    near_plane: f32,
    far_plane: f32,
    aperture: f32,
    shutter_speed: f32,
    sensitivity: f32,
    orbit_speed: f32,
    zoom_speed: f32,
    orbit_dampening: f32,
    zoom_dampening: f32,
    min_distance: f32,
    max_distance: f32,
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 1.0),
            target: Default::default(),
            fov_deg: 60,
            near_plane: 0.1,
            far_plane: 500.0,
            aperture: 1.4,
            shutter_speed: 0.55,
            sensitivity: 500.0,
            orbit_speed: 1.0,
            zoom_speed: 1.0,
            orbit_dampening: 0.0,
            zoom_dampening: 0.0,
            min_distance: 0.0,
            max_distance: 1.0,
        }
    }
}

impl CameraBuilder {
    pub fn new() -> Self {
        CameraBuilder::default()
    }

    pub fn position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn target(mut self, target: Vec3) -> Self {
        self.target = target;
        self
    }

    pub fn fov(mut self, degrees: u32) -> Self {
        self.fov_deg = degrees;
        self
    }

    pub fn near_plane(mut self, near_plane: f32) -> Self {
        self.near_plane = near_plane;
        self
    }

    pub fn far_plane(mut self, far_plane: f32) -> Self {
        self.far_plane = far_plane;
        self
    }

    pub fn aperture(mut self, aperture: f32) -> Self {
        self.aperture = aperture;
        self
    }

    pub fn shutter_speed(mut self, shutter_speed: f32) -> Self {
        self.shutter_speed = shutter_speed;
        self
    }

    pub fn sensitivity(mut self, sensitivity: f32) -> Self {
        self.sensitivity = sensitivity;
        self
    }

    pub fn orbit_speed(mut self, orbit_speed: f32) -> Self {
        self.orbit_speed = orbit_speed;
        self
    }

    pub fn zoom_speed(mut self, zoom_speed: f32) -> Self {
        self.zoom_speed = zoom_speed;
        self
    }

    pub fn orbit_dampening(mut self, orbit_dampening: f32) -> Self {
        self.orbit_dampening = orbit_dampening;
        self
    }

    pub fn zoom_dampening(mut self, zoom_dampening: f32) -> Self {
        self.zoom_dampening = zoom_dampening;
        self
    }

    pub fn min_distance(mut self, min_distance: f32) -> Self {
        self.min_distance = min_distance;
        self
    }

    pub fn max_distance(mut self, max_distance: f32) -> Self {
        self.max_distance = max_distance;
        self
    }

    pub fn build(self) -> Camera {
        let transform = math::look_at(&self.position, &self.target, &Axes::up());

        let distance = (self.position - self.target).norm();

        let mut uniform_buffer = Buffer::new(
            "Camera UBO",
            std::mem::size_of::<CameraUniformBlock>() as isize,
            BufferTarget::Uniform,
            BufferStorageFlags::MAP_WRITE_PERSISTENT_COHERENT,
        );
        uniform_buffer.bind(0);
        uniform_buffer.map(MapModeFlags::MAP_WRITE_PERSISTENT_COHERENT);

        Camera {
            position: self.position,
            orientation: matrix::to_rotation_quat(&transform),
            transform,
            fov_deg: self.fov_deg,
            near_plane: self.near_plane,
            far_plane: self.far_plane,
            aperture: self.aperture,
            shutter_speed: self.shutter_speed,
            sensitivity: self.sensitivity,
            orbit_speed: self.orbit_speed,
            zoom_speed: self.zoom_speed,
            min_distance: self.min_distance,
            max_distance: self.max_distance,
            orbit_dampening: self.orbit_dampening,
            zoom_dampening: self.zoom_dampening,
            yaw: 0.0,
            pitch: 0.0,
            distance,
            prev_distance: distance,
            uniform_buffer,
        }
    }
}

impl Gui for Camera {
    fn gui(&mut self, ui: &Ui) {
        if imgui::CollapsingHeader::new(im_str!("Camera"))
            .default_open(true)
            .open_on_arrow(true)
            .open_on_double_click(true)
            .build(ui)
        {
            ui.spacing();
            ui.group(|| {
                imgui::TreeNode::new(im_str!("Controls"))
                    .default_open(true)
                    .open_on_arrow(true)
                    .open_on_double_click(true)
                    .framed(false)
                    .build(ui, || {
                        imgui::TreeNode::new(im_str!("Lens"))
                            .default_open(true)
                            .open_on_arrow(true)
                            .open_on_double_click(true)
                            .framed(false)
                            .build(ui, || {
                                let mut aperture = self.aperture;
                                if imgui::Slider::new(im_str!("Aperture (f-stop)"))
                                    .range(RangeInclusive::new(32.0, 1.4))
                                    .display_format(im_str!("%.2f"))
                                    .build(ui, &mut aperture)
                                {
                                    self.aperture = aperture;
                                }

                                let mut shutter_speed = self.shutter_speed;
                                if imgui::Slider::new(im_str!("Shutter Speed (s)"))
                                    .range(RangeInclusive::new(0.00025, 30.0))
                                    .display_format(im_str!("%.2f"))
                                    .build(ui, &mut shutter_speed)
                                {
                                    self.shutter_speed = shutter_speed;
                                }

                                let mut sensitivity = self.sensitivity;
                                if imgui::Slider::new(im_str!("Sensitivity (ISO)"))
                                    .range(RangeInclusive::new(200.0, 1600.0))
                                    .display_format(im_str!("%.2f"))
                                    .build(ui, &mut sensitivity)
                                {
                                    self.sensitivity = sensitivity;
                                }
                            });

                        imgui::TreeNode::new(im_str!("Projection"))
                            .default_open(true)
                            .open_on_arrow(true)
                            .open_on_double_click(true)
                            .framed(false)
                            .build(ui, || {
                                let mut fov = self.fov_deg;
                                if imgui::Drag::new(im_str!("Field of View"))
                                    .range(RangeInclusive::new(1u32, 180u32))
                                    .speed(1.0)
                                    .build(ui, &mut fov)
                                {
                                    self.fov_deg = fov;
                                }

                                let mut near_plane = self.near_plane;
                                if imgui::Drag::new(im_str!("Near Plane"))
                                    .range(RangeInclusive::new(0.5, 5000.0))
                                    .display_format(im_str!("%.2f"))
                                    .speed(0.5)
                                    .build(&ui, &mut near_plane)
                                {
                                    self.near_plane = near_plane;
                                }

                                let mut far_plane = self.far_plane;
                                if imgui::Drag::new(im_str!("Far Plane"))
                                    .range(RangeInclusive::new(0.5, 5000.0))
                                    .display_format(im_str!("%.2f"))
                                    .speed(0.5)
                                    .build(&ui, &mut far_plane)
                                {
                                    self.far_plane = far_plane;
                                }
                            });

                        imgui::TreeNode::new(im_str!("Movement"))
                            .default_open(true)
                            .open_on_arrow(true)
                            .open_on_double_click(true)
                            .framed(false)
                            .build(ui, || {
                                let mut orbit_speed = self.orbit_speed;
                                if imgui::Slider::new(im_str!("Orbit Speed"))
                                    .range(RangeInclusive::new(1.0, 10.0))
                                    .display_format(im_str!("%.2f"))
                                    .build(&ui, &mut orbit_speed)
                                {
                                    self.set_orbit_speed(orbit_speed)
                                }

                                let mut orbit_dampening = self.orbit_dampening();
                                if imgui::Slider::new(im_str!("Orbit Dampening"))
                                    .range(RangeInclusive::new(1.0, 10.0))
                                    .display_format(im_str!("%.2f"))
                                    .build(&ui, &mut orbit_dampening)
                                {
                                    self.set_orbit_dampening(orbit_dampening)
                                }

                                let mut zoom_speed = self.zoom_speed();
                                if imgui::Slider::new(im_str!("Zoom Speed"))
                                    .range(RangeInclusive::new(1.0, 40.0))
                                    .display_format(im_str!("%.2f"))
                                    .build(&ui, &mut zoom_speed)
                                {
                                    self.set_zoom_speed(zoom_speed)
                                }

                                let mut zoom_dampening = self.zoom_dampening();
                                if imgui::Slider::new(im_str!("Zoom Dampening"))
                                    .range(RangeInclusive::new(0.1, 10.0))
                                    .display_format(im_str!("%.2f"))
                                    .build(&ui, &mut zoom_dampening)
                                {
                                    self.set_zoom_dampening(zoom_dampening)
                                }
                            });
                    });
            });
        }
    }
}
