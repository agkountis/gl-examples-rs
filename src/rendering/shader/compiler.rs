use crate::rendering::shader::ShaderStage;
use shaderc::{CompilationArtifact, CompileOptions, ShaderKind, TargetEnv};

// TODO: This is extremely bad and should be re-architected
pub(crate) static mut SHADER_COMPILER: Compiler = Compiler::new();

pub(crate) struct Compiler {
    compiler: shaderc::Compiler,
}

impl Default for Compiler {
    fn default() -> Self {
        let compiler = shaderc::Compiler::new().expect("Failed to initialize underlying compiler");

        Self { compiler }
    }
}

impl Compiler {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn compile(
        &mut self,
        source: &str,
        source_file_name: &str,
        stage: ShaderStage,
        defines: &[&str],
    ) -> shaderc::Result<CompilationArtifact> {
        let compile_options = Self::setup_compile_options(defines);

        self.compiler.compile_into_spirv(
            source,
            stage.into(),
            source_file_name,
            "main",
            Some(&compile_options),
        )
    }

    pub fn preprocess(
        &mut self,
        source: &str,
        source_file_name: &str,
        defines: &[&str],
    ) -> shaderc::Result<CompilationArtifact> {
        let compile_options = Self::setup_compile_options(defines);

        self.compiler
            .preprocess(source, source_file_name, "main", Some(&compile_options))
    }

    fn setup_compile_options<'a>(defines: &[&str]) -> CompileOptions<'a> {
        let mut compile_options = shaderc::CompileOptions::new()
            .expect("Failed to initialize underlying compiler options object.");
        compile_options.set_target_env(TargetEnv::OpenGL, 450);

        defines
            .iter()
            .for_each(|&define| compile_options.add_macro_definition(define, None));

        compile_options
    }
}

impl From<ShaderStage> for ShaderKind {
    fn from(stage: ShaderStage) -> Self {
        match stage {
            ShaderStage::Vertex => ShaderKind::Vertex,
            ShaderStage::TesselationControl => ShaderKind::TessControl,
            ShaderStage::TesselationEvaluation => ShaderKind::TessEvaluation,
            ShaderStage::Geometry => ShaderKind::Geometry,
            ShaderStage::Fragment => ShaderKind::Fragment,
        }
    }
}

impl From<ShaderKind> for ShaderStage {
    fn from(kind: ShaderKind) -> Self {
        match kind {
            ShaderKind::Vertex => ShaderStage::Vertex,
            ShaderKind::TessControl => ShaderStage::TesselationControl,
            ShaderKind::TessEvaluation => ShaderStage::TesselationEvaluation,
            ShaderKind::Geometry => ShaderStage::Geometry,
            ShaderKind::Fragment => ShaderStage::Fragment,
            _ => panic!("Unsupported Shader Kind. Cannot convert to shader stage."),
        }
    }
}
