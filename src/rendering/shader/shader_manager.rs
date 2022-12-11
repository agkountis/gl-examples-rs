use crate::imgui::{Gui, Ui};
use crate::rendering::shader::compiler::Compiler;
use crate::rendering::shader::module::ShaderModule;
use crate::rendering::shader::program::{ShaderProgram, ShaderProgramBuilder};
use crate::rendering::shader::Shader;
use crate::shader::{ShaderCreateInfo, ShaderStage};
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

type ShaderModuleCache = HashMap<String, Rc<ShaderModule>>;

#[derive(Debug)]
struct CompileItem {
    shader_stage: ShaderStage,
    file_name: String,
    source: String,
}

#[derive(Default)]
pub struct ShaderManager {
    compiler: Compiler,
    shaders: Vec<Rc<Shader>>,
    shader_module_cache: ShaderModuleCache,
}

impl ShaderManager {
    pub fn create_shader(&mut self, create_info: &ShaderCreateInfo) -> Rc<Shader> {
        let keyword_bitfield_map = Self::create_keyword_bitfield_map(create_info);

        let stages = Self::create_compile_items(create_info);

        let keyword_combinations = Self::extract_keyword_set_combinations(create_info);

        let mut default_variant_bitfield = 0u32;
        for keyword_set in create_info.keyword_sets.iter() {
            let kw = *keyword_set.iter().next().unwrap();
            default_variant_bitfield |= keyword_bitfield_map[kw];
        }

        let shader_variants = Self::create_shader_variants(
            stages,
            keyword_combinations,
            &keyword_bitfield_map,
            &mut self.compiler,
            &mut self.shader_module_cache,
        );

        let shader = Rc::new(Shader {
            name: create_info.name.clone(),
            bound: RefCell::new(false),
            active_variant: RefCell::new(shader_variants[&default_variant_bitfield].id()),
            active_variant_bitfield: RefCell::new(default_variant_bitfield),
            shader_variants,
            keyword_bitfield_map,
        });

        self.shaders.push(Rc::clone(&shader));

        shader
    }

    pub fn find_shader(&self, name: &str) -> Option<Rc<Shader>> {
        self.shaders
            .iter()
            .find(|&entry| entry.name == name)
            .cloned()
    }

    fn create_keyword_bitfield_map(create_info: &ShaderCreateInfo) -> HashMap<String, u32> {
        create_info
            .keyword_sets
            .iter()
            .flatten()
            .unique()
            .copied()
            .fold((vec![], 0u32), |(mut tuples, mut acc), keyword| {
                if keyword == "_" {
                    tuples.push((String::from(keyword), 0));
                    return (tuples, acc);
                }

                tuples.push((String::from(keyword), 1u32 << acc));
                acc += 1;

                (tuples, acc)
            })
            .0 // Vec of tuples
            .into_iter()
            .collect::<HashMap<_, _>>()
    }

    fn create_compile_items(create_info: &ShaderCreateInfo) -> Vec<CompileItem> {
        create_info
            .stages
            .iter()
            .map(|(stage, path)| {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let source = load_shader_source(path);

                CompileItem {
                    shader_stage: *stage,
                    file_name,
                    source,
                }
            })
            .collect_vec()
    }

    fn extract_keyword_set_combinations<'a>(
        create_info: &'a ShaderCreateInfo,
    ) -> Vec<Vec<&'a str>> {
        create_info
            .keyword_sets
            .iter()
            .multi_cartesian_product()
            .map(|keywords| keywords.into_iter().unique().copied().collect_vec())
            .collect_vec()
    }

    fn create_shader_variants(
        compile_items: Vec<CompileItem>,
        keyword_sets: Vec<Vec<&str>>,
        keyword_bitfield_map: &HashMap<String, u32>,
        compiler: &mut Compiler,
        shader_module_cache: &mut ShaderModuleCache,
    ) -> HashMap<u32, ShaderProgram> {
        let mut shader_variants: HashMap<u32, ShaderProgram> =
            HashMap::with_capacity(keyword_sets.len());

        let mut cache_additions = vec![];

        for keyword_set in keyword_sets.into_iter() {
            let mut shader_modules = Vec::with_capacity(compile_items.len());

            for CompileItem {
                shader_stage,
                file_name,
                source,
            } in compile_items.iter()
            {
                match shader_module_cache.get(file_name.as_str()) {
                    Some(module) => shader_modules.push(Rc::clone(module)),
                    None => {
                        let filtered_keywords = keyword_set
                            .iter()
                            .filter(|&keyword| *keyword != "_")
                            .copied()
                            .collect_vec();

                        let maybe_keywords =
                            (!filtered_keywords.is_empty()).then(|| filtered_keywords.as_slice());

                        let compiled_artifact = if cfg!(feature = "use-spirv") {
                            compiler
                                .compile(source, file_name, *shader_stage, maybe_keywords)
                                .unwrap()
                        } else {
                            compiler
                                .preprocess(source, file_name, maybe_keywords)
                                .unwrap()
                        };

                        let module =
                            Rc::new(ShaderModule::new(*shader_stage, &compiled_artifact).unwrap());
                        shader_modules.push(Rc::clone(&module));
                        cache_additions.push((file_name.clone(), module))
                    }
                }
            }

            let bitfield = keyword_set
                .iter()
                .map(|&s| String::from(s))
                .fold(0u32, |acc, s| acc | keyword_bitfield_map[&s]);

            let mut program_builder = ShaderProgramBuilder::new();

            for shader_module in shader_modules.iter() {
                program_builder = program_builder.with_shader_module(shader_module);
            }

            let program = program_builder
                .build()
                .expect("Failed to create shader program.");

            shader_variants.insert(bitfield, program);
        }

        for (key, value) in cache_additions.into_iter() {
            shader_module_cache.insert(key, value);
        }

        shader_variants
    }
}

impl Gui for ShaderManager {
    fn gui(&mut self, ui: &Ui) {
        // TODO
    }
}

fn load_shader_source<P: AsRef<Path> + Debug>(path: P) -> String {
    let mut source = String::new();

    {
        let mut file = match File::open(path.as_ref()) {
            Err(why) => panic!("couldn't open {:?}: {}", path, why),
            Ok(file) => file,
        };

        let size = file.read_to_string(&mut source).unwrap();

        assert_eq!(
            size,
            source.len(),
            "Could not read the entirety of the file."
        );
    }

    source
}
