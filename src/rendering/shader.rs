use std::collections::HashMap;

use crate::asset_management::ResolvableAsset;

pub struct Shader {
    module: wgpu::ShaderModule,
}

impl Shader {
    pub fn new(device: &wgpu::Device, source: Box<dyn ResolvableAsset>) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(&source.resolve()),
        });
        Shader { module }
    }
}

pub struct ShaderManager {
    shaders: HashMap<String, Shader>,
}
