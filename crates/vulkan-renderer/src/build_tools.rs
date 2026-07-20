use std::{path::PathBuf, process::Command};

use renderer131::build_tools::{ShaderBuilderError, ShaderCompiler};

#[derive(Default)]
pub struct VulkanShaderBuilder {}

impl ShaderCompiler for VulkanShaderBuilder {
    fn backend_name(&self) -> &'static str {
        "Vulkan"
    }

    fn build_shader(
        &self,
        name: &str,
        stage: renderer131::build_tools::ShaderStage,
        source: Vec<u8>,
    ) -> Result<Vec<u8>, renderer131::build_tools::ShaderBuilderError> {
        let temp_folder = PathBuf::from(std::env::var("OUT_DIR")?).join("temp/");
        std::fs::create_dir_all(&temp_folder)?;

        let stage_name = match stage {
            renderer131::build_tools::ShaderStage::Vertex => "vert",
            renderer131::build_tools::ShaderStage::Pixel => "frag",
            renderer131::build_tools::ShaderStage::Compute => todo!(),
        };

        let temp_source_file = temp_folder.join(name);
        std::fs::write(&temp_source_file, source)?;

        let mut glslc = Command::new("glslc");

        let temp_bin_file = temp_folder.join(format!("{name}.bin"));
        glslc.args([
            temp_source_file
                .to_str()
                .ok_or(ShaderBuilderError::CompilerError(
                    "Could not convert path to string".to_string(),
                ))?,
            &format!("-fshader-stage={stage_name}"),
            "-o",
            temp_bin_file
                .to_str()
                .ok_or(ShaderBuilderError::CompilerError(
                    "Could not convert path to string".to_string(),
                ))?,
        ]);

        eprintln!("Building:\n\t{glslc:?}");

        let exit_status = glslc.status()?;
        if !exit_status.success() {
            return Err(ShaderBuilderError::CompilerError(
                "Shader compilation failed".to_string(),
            ));
        }

        let shader_bin = std::fs::read(&temp_bin_file)?;
        std::fs::remove_file(temp_bin_file)?;
        std::fs::remove_file(temp_source_file)?;

        Ok(shader_bin)
    }
}
