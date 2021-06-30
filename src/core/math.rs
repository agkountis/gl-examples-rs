pub use matrix::*;
pub use quaternion::*;
pub use vector::*;

pub mod vector {
    use nalgebra_glm as glm;

    pub type Vec2 = glm::Vec2;
    pub type UVec2 = glm::UVec2;
    pub type IVec2 = glm::IVec2;

    pub type Vec3 = glm::Vec3;
    pub type UVec3 = glm::UVec3;
    pub type IVec3 = glm::IVec3;

    pub type Vec4 = glm::Vec4;
    pub type UVec4 = glm::UVec4;
    pub type IVec4 = glm::IVec4;

    pub fn vec3_lerp(a: &Vec3, b: &Vec3, t: f32) -> Vec3 {
        glm::lerp(a, b, t)
    }

    pub fn vec4_lerp(a: &Vec4, b: &Vec4, t: f32) -> Vec4 {
        glm::lerp(a, b, t)
    }

    pub struct Axes;

    impl Axes {
        pub fn up() -> Vec3 {
            Vec3::new(0.0, 1.0, 0.0)
        }

        pub fn right() -> Vec3 {
            Vec3::new(1.0, 0.0, 0.0)
        }

        pub fn forward() -> Vec3 {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }
}

pub mod quaternion {
    use crate::math::vector::Axes;
    use crate::math::{Mat4, Vec3};
    use nalgebra_glm as glm;

    pub type Quat = glm::Quat;

    pub fn from_euler(yaw: f32, pitch: f32, roll: f32) -> Quat {
        let y = glm::quat_angle_axis(yaw.to_radians(), &Axes::up());
        let p = glm::quat_angle_axis(pitch.to_radians(), &Axes::right());
        let r = glm::quat_angle_axis(roll.to_radians(), &Axes::forward());

        glm::quat_normalize(&(y * p * r))
    }

    pub fn quat_look_at(target: &Vec3, up: &Vec3) -> Quat {
        glm::quat_look_at(target, up)
    }

    pub fn to_mat4(quat: &Quat) -> Mat4 {
        glm::quat_to_mat4(&quat)
    }

    pub fn lerp(a: &Quat, b: &Quat, t: f32) -> Quat {
        glm::quat_lerp(a, b, t)
    }

    pub fn slerp(a: &Quat, b: &Quat, t: f32) -> Quat {
        glm::quat_normalize(&glm::quat_slerp(a, b, t))
    }

    pub fn rotate_vec3(quat: &Quat, vec: &Vec3) -> Vec3 {
        glm::quat_rotate_vec3(quat, vec)
    }
}

pub mod matrix {
    use super::vector::Vec3;
    use crate::core::math::Quat;
    use nalgebra_glm as glm;

    pub type Mat4 = glm::Mat4;

    pub fn translate(matrix: &Mat4, position: &Vec3) -> Mat4 {
        glm::translate(matrix, &position)
    }

    pub fn translate_xyz(matrix: &Mat4, x: f32, y: f32, z: f32) -> Mat4 {
        glm::translate(matrix, &Vec3::new(x, y, z))
    }

    pub fn rotate(matrix: &Mat4, angle_deg: f32, axis: &Vec3) -> Mat4 {
        glm::rotate(matrix, angle_deg.to_radians(), axis)
    }

    pub fn scale(matrix: &Mat4, scale: &Vec3) -> Mat4 {
        glm::scale(matrix, &scale)
    }

    pub fn scale_xyz(matrix: &Mat4, x: f32, y: f32, z: f32) -> Mat4 {
        glm::scale(matrix, &Vec3::new(x, y, z))
    }

    pub fn perspective(win_width: u32, win_height: u32, fov_deg: u32, near: f32, far: f32) -> Mat4 {
        glm::perspective(
            win_width as f32 / win_height as f32,
            f32::to_radians(fov_deg as f32),
            near,
            far,
        )
    }

    pub fn look_at(position: &Vec3, target: &Vec3, up: &Vec3) -> Mat4 {
        glm::look_at(position, target, up)
    }

    pub fn transpose(mat: &Mat4) -> Mat4 {
        glm::transpose(mat)
    }

    pub fn inverse(mat: &Mat4) -> Mat4 {
        glm::inverse(mat)
    }

    pub fn inverse_transpose(mat: Mat4) -> Mat4 {
        glm::inverse_transpose(mat)
    }

    pub fn to_rotation_quat(mat: &Mat4) -> Quat {
        glm::quat_normalize(&glm::mat3_to_quat(&glm::mat4_to_mat3(&mat)))
    }
}

use nalgebra_glm as glm;

pub fn clamp_scalar(x: f32, min: f32, max: f32) -> f32 {
    glm::clamp_scalar(x, min, max)
}

pub fn lerp_scalar(a: f32, b: f32, t: f32) -> f32 {
    glm::lerp_scalar(a, b, t)
}

pub fn spherical_to_cartesian(theta: f32, phi: f32) -> Vec3 {
    let theta = theta.to_radians();
    let phi = phi.to_radians();
    Vec3::new(phi.sin() * theta.sin(), phi.sin() * theta.cos(), phi.cos())
}
