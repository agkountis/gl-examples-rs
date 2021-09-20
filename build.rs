use std::process::Command;

fn main() {
    if !cfg!(feature = "auto-compile-spirv") {
        return;
    }

    let paths = [
        "**/assets/**/sdr/*.vert",
        "**/assets/**/sdr/*.tesc",
        "**/assets/**/sdr/*.tese",
        "**/assets/**/sdr/*.geom",
        "**/assets/**/sdr/*.frag",
        "**/assets/**/sdr/*.comp",
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

    println!("cargo:rerun-if-changed=assets/sdr");
    paths.iter().for_each(|path| {
        let output_fname = path.file_name().unwrap().to_str().unwrap().to_owned() + ".spv";
        let output = Command::new("glslangValidator")
            .current_dir("assets/sdr")
            .args(&[
                "-G",
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
                "-G",
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
