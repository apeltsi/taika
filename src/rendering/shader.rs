use crate::asset_management::ResolvableAsset;

/// Wrapper around [`wgpu::ShaderModule`]
pub struct Shader {
    module: wgpu::ShaderModule,
}

impl Shader {
    /// Creates a new shader from a source file
    pub fn new(device: &wgpu::Device, source: Box<dyn ResolvableAsset>) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                String::from_utf8(source.resolve()).unwrap().as_str().into(),
            ),
        });
        Shader { module }
    }

    /// Returns the [`wgpu::ShaderModule`] of the shader
    pub fn get_module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
}
