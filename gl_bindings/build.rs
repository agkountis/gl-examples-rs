use gl_generator::{Api, Fallbacks, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let dest = Path::new(&out_dir);

    let mut file = File::create(&dest.join("gl_bindings.rs")).unwrap();

    Registry::new(
        Api::Gl,
        (4, 6),
        Profile::Core,
        Fallbacks::All,
        [
            "GL_ARB_framebuffer_object",
            "GL_ARB_draw_buffers_blend",
            "GL_ARB_program_interface_query",
            "GL_ARB_multitexture",
            "GLX_ARB_get_proc_address",
            "GL_ARB_transpose_matrix",
            "WGL_ARB_buffer_region",
            "GL_ARB_multisample",
            "GLX_ARB_multisample",
            "WGL_ARB_multisample",
            "GL_ARB_texture_env_add",
            "GL_ARB_texture_cube_map",
            "WGL_ARB_extensions_string",
            "WGL_ARB_pixel_format",
            "GL_ARB_gl_spirv",
            "GLX_ARB_create_context_no_error",
            "WGL_ARB_create_context_no_error",
            "KHR_parallel_shader_compile",
            "GL_ARB_polygon_offset_clamp",
            "GL_ARB_spirv_extensions",
            "GL_ARB_texture_filter_anisotropic",
        ],
    )
    .write_bindings(gl_generator::GlobalGenerator, &mut file)
    .unwrap();
}
