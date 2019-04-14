pub mod vector {
    use nalgebra_glm as glm;

    pub type Vec2 = glm::Vec2;
    pub type UVec2 = glm::UVec2;
    pub type IVec2 = glm::IVec2;
    pub type DVec2 = glm::DVec2;

    pub type Vec3 = glm::Vec3;
    pub type UVec3 = glm::UVec3;
    pub type IVec3 = glm::IVec3;
    pub type DVec3 = glm::DVec3;

    pub type Vec4 = glm::Vec4;
    pub type UVec4 = glm::UVec4;
    pub type IVec4 = glm::IVec4;
    pub type DVec4 = glm::DVec4;
}

pub mod quaternion {
    use nalgebra_glm as glm;

    pub type Quat = glm::Quat;
    pub type DQuat = glm::DQuat;
}

pub mod matrix {
    use nalgebra_glm as glm;

    use super::vector::Vec3;

    pub type Mat3 = glm::Mat3;
    pub type Mat4 = glm::Mat4;

    pub fn translate(matrix: &Mat4, position: Vec3) -> Mat4 {
        glm::translate(matrix, &position)
    }

    pub fn translate_xyz(matrix: &Mat4, x: f32, y: f32, z: f32) -> Mat4 {
        glm::translate(matrix, &Vec3::new(x, y, z))
    }

    pub fn rotate(matrix: &Mat4, angle_deg: f32, axis: Vec3) -> Mat4 {
        glm::rotate(matrix, angle_deg * glm::pi::<f32>() / 180.0, &axis)
    }

    pub fn scale(matrix: &Mat4, scale: Vec3) -> Mat4 {
        glm::scale(matrix, &scale)
    }

    pub fn scale_xyz(matrix: &Mat4, x: f32, y: f32, z: f32) -> Mat4 {
        glm::scale(matrix, &Vec3::new(x, y, z))
    }

    pub fn perspective(win_width: u32, win_height: u32, fov_deg: u32, near: f32, far: f32) -> Mat4 {
        glm::perspective(win_width as f32 / win_height as f32,
                         fov_deg as f32 * glm::pi::<f32>() / 180.0,
                         near,
                         far)
    }
}

pub mod utilities {
    use nalgebra_glm as glm;

    pub fn value_ptr<N: glm::Scalar,
                     R: glm::Dimension,
                     C: glm::Dimension>(value: &glm::TMat<N, R, C>) -> *const N
        where glm::DefaultAllocator: glm::Alloc<N, R, C> {
        glm::value_ptr(value).as_ptr()
    }
}


