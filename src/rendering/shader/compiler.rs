use crate::rendering::shader::ShaderStage;
use shaderc::{
    CompilationArtifact, CompileOptions, EnvVersion, IncludeCallbackResult, IncludeType,
    ResolvedInclude, ShaderKind, TargetEnv,
};
use std::fs::File;
use std::io::Read;

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
        defines: Option<&[&str]>,
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
        defines: Option<&[&str]>,
    ) -> shaderc::Result<CompilationArtifact> {
        let mut compile_options = Self::setup_compile_options(defines);
        compile_options.set_target_env(TargetEnv::OpenGL, EnvVersion::OpenGL4_5 as u32);
        compile_options.set_include_callback(Self::include_resolve_callback);
        self.compiler
            .preprocess(source, source_file_name, "main", Some(&compile_options))
    }

    fn setup_compile_options<'a>(defines: Option<&'a [&str]>) -> CompileOptions<'a> {
        let mut compile_options = shaderc::CompileOptions::new()
            .expect("Failed to initialize underlying compiler options object.");

        compile_options.set_target_env(TargetEnv::OpenGL, EnvVersion::OpenGL4_5 as u32);

        compile_options.set_include_callback(Self::include_resolve_callback);

        if let Some(defines) = defines {
            defines
                .iter()
                .for_each(|&define| compile_options.add_macro_definition(define, None))
        }

        compile_options
    }

    fn include_resolve_callback(
        requested_file_name: &str,
        include_type: IncludeType,
        source_file_name: &str,
        include_depth: usize,
    ) -> IncludeCallbackResult {
        println!("Attempting to resolve library: {}", requested_file_name);
        println!("Include Type: {:?}", include_type);
        println!("Directive source file: {}", source_file_name);
        println!("Current library depth: {}", include_depth);

        let mut content = String::new();

        {
            let mut file = File::open(requested_file_name).unwrap();
            file.read_to_string(&mut content).unwrap();
        }

        IncludeCallbackResult::Ok(ResolvedInclude {
            resolved_name: requested_file_name.to_string(),
            content,
        })
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
