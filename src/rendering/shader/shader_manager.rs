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

        let keyword_sets = Self::extract_keyword_set_combinations(create_info);

        let default_variant_bitfield = keyword_sets
            .iter()
            .map(|vec| vec.iter().copied().next().unwrap())
            .filter(|&a| a != "_")
            .map(String::from)
            .fold(0u32, |acc, keyword| acc | keyword_bitfield_map[&keyword]);

        let shader_variants = Self::create_shader_variants(
            stages,
            keyword_sets,
            &keyword_bitfield_map,
            &mut self.compiler,
            &mut self.shader_module_cache,
        );

        println!("Shader variants: {:?}", shader_variants);

        // TODO: This takes the 1st variant as default. This is not always correct

        let (default_variant_bitfield, default_variant_id) = shader_variants
            .iter()
            .sorted_by(|(&a, _), (&b, _)| Ord::cmp(&a, &b))
            .next()
            .map(|(a, b)| (*a, b.id()))
            .unwrap();

        let shader = Rc::new(Shader {
            name: create_info.name.clone(),
            active_variant: RefCell::new(shader_variants[&default_variant_bitfield].id()), // TODO: Have to figure out which one should be the default active shader variant.
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

            println!("Keyword set: {:?}", keyword_set);

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
                        println!("Filtered keywords: {:?}", filtered_keywords);

                        let maybe_keywords =
                            (!filtered_keywords.is_empty()).then(|| filtered_keywords.as_slice());

                        println!("Compile keyword set opt: {:?}", maybe_keywords);

                        let compiled_artifact = compiler
                            .compile(source, file_name, *shader_stage, maybe_keywords)
                            .unwrap();

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

mod tests {
    use crate::rendering::shader::ShaderCreateInfo;
    use crate::shader::shader_manager::ShaderManager;
    use itertools::Itertools;

    #[test]
    fn test_extract_keyword_set_combinations() {
        let create_info = ShaderCreateInfo::builder("foo")
            .keyword_set(&["_", "foo", "bla", "kek", "lol"])
            .keyword_set(&["_", "ji", "jo"])
            .keyword_set(&["asd", "aaa", "123sasd"])
            .build();

        let combinations = ShaderManager::extract_keyword_set_combinations(&create_info);

        println!("{:?}", combinations)
    }

    #[test]
    fn test_create_keyword_bitfield_map() {
        let create_info = ShaderCreateInfo::builder("foo")
            .keyword_set(&["_", "foo", "bla", "kek", "lol"])
            .keyword_set(&["_", "ji", "jo"])
            .keyword_set(&["asd", "aaa", "123sasd"])
            .build();

        let bitfield_map = ShaderManager::create_keyword_bitfield_map(&create_info);
        println!("{:?}", bitfield_map)
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
