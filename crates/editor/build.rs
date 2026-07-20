use renderer131::build_tools::ShaderCompiler;
use renderer131::build_tools::{ShaderBuilderError, ShaderSources};
use vulkan_renderer::build_tools::VulkanShaderBuilder;

fn main() -> Result<(), ShaderBuilderError> {
    let shader_meta = ShaderSources::read()?;
    let vulkan_shader_builder = VulkanShaderBuilder::default();

    vulkan_shader_builder.build_compatible_shaders(shader_meta)?;

    Ok(())
}
