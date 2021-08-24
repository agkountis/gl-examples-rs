use gl_bindings as gl;

pub struct StateManager;

#[repr(u32)]
pub enum BlendFactor {
    Zero = gl::ZERO,
    One = gl::ONE,
    SourceColor = gl::SRC_COLOR,
    OneMinusSourceColor = gl::ONE_MINUS_SRC_COLOR,
    DestinationColor = gl::DST_COLOR,
    OneMinusDestinationColor = gl::ONE_MINUS_DST_COLOR,
    SourceAlpha = gl::SRC_ALPHA,
    OneMinusSourceAlpha = gl::ONE_MINUS_SRC_ALPHA,
    DestinationAlpha = gl::DST_ALPHA,
    OneMinusDestinationAlpha = gl::ONE_MINUS_DST_ALPHA,
    ConstantColor = gl::CONSTANT_COLOR,
    OneMinusConstantColor = gl::ONE_MINUS_CONSTANT_COLOR,
    ConstantAlpha = gl::CONSTANT_ALPHA,
    OneMinusConstantAlpha = gl::ONE_MINUS_CONSTANT_ALPHA,
    SourceAlphaSaturate = gl::SRC_ALPHA_SATURATE,
    Source1Color = gl::SRC1_COLOR,
    OneMinusSource1Color = gl::ONE_MINUS_SRC1_COLOR,
    Source1Alpha = gl::SRC1_ALPHA,
    OneMinusSource1Alpha = gl::ONE_MINUS_SRC1_ALPHA,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum DepthFunction {
    Less = gl::LESS,
    LessOrEqual = gl::LEQUAL,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum FaceCulling {
    Front = gl::FRONT,
    Back = gl::BACK,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum FrontFace {
    Clockwise = gl::CW,
    CounterClockwise = gl::CCW,
}

impl StateManager {
    pub fn set_viewport(x: i32, y: i32, width: i32, height: i32) {
        unsafe { gl::Viewport(x, y, width, height) }
    }

    pub fn enable_blending() {
        unsafe { gl::Enable(gl::BLEND) }
    }

    pub fn disable_blending() {
        unsafe { gl::Disable(gl::BLEND) }
    }

    pub fn set_blend_function(source_factor: BlendFactor, destination_factor: BlendFactor) {
        unsafe { gl::BlendFunc(source_factor as u32, destination_factor as u32) }
    }

    pub fn set_depth_function(depth_function: DepthFunction) {
        unsafe { gl::DepthFunc(depth_function as u32) }
    }

    pub fn set_face_culling(culling: FaceCulling) {
        unsafe { gl::CullFace(culling as u32) }
    }

    pub fn set_front_face(front_face: FrontFace) {
        unsafe { gl::FrontFace(front_face as u32) }
    }
}
