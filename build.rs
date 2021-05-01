use std::process::Command;

fn main() {
    if !cfg!(feature = "auto-compile-spirv") {
        return;
    }

    let paths = [
        "**/examples/**/sdr/*.vert",
        "**/examples/**/sdr/*.tesc",
        "**/examples/**/sdr/*.tese",
        "**/examples/**/sdr/*.geom",
        "**/examples/**/sdr/*.frag",
        "**/examples/**/sdr/*.comp",
    ]
    .iter()
    .flat_map(|&pattern| {
        glob::glob(pattern)
            .unwrap()
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();

    println!("cargo:rerun-if-changed=examples/assets/sdr");
    paths.iter().for_each(|path| {
        let output_fname = path.file_name().unwrap().to_str().unwrap().to_owned() + ".spv";
        let output = Command::new("glslangValidator")
            .current_dir("examples/assets/sdr")
            .args(&[
                "-G450",
                "-e main",
                "-Os",
                "-o",
                &output_fname,
                path.file_name().unwrap().to_str().unwrap(),
            ])
            .output()
            .expect("Failed to run glslangValidator");

        if !output.status.success() {
            panic!("{:?}", output)
        }

        println!("cargo:rerun-if-changed={:?}", path);
    });

    let internal_paths = [
        "**/src/**/shaders/*.vert",
        "**/src/**/shaders/*.tesc",
        "**/src/**/shaders/*.tese",
        "**/src/**/shaders/*.geom",
        "**/src/**/shaders/*.frag",
        "**/src/**/shaders/*.comp",
    ]
    .iter()
    .flat_map(|&pattern| {
        glob::glob(pattern)
            .unwrap()
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();

    println!("cargo:rerun-if-changed=src/rendering/postprocess/shaders");
    internal_paths.iter().for_each(|path| {
        let output_fname = path.file_name().unwrap().to_str().unwrap().to_owned() + ".spv";
        let output = Command::new("glslangValidator")
            .current_dir("src/rendering/postprocess/shaders")
            .args(&[
                "-G450",
                "-e main",
                "-Os",
                "-o",
                &output_fname,
                path.file_name().unwrap().to_str().unwrap(),
            ])
            .output()
            .expect("Failed to run glslangValidator");

        if !output.status.success() {
            panic!("{:?}", output)
        }

        println!("cargo:rerun-if-changed={:?}", path);
    });
}
