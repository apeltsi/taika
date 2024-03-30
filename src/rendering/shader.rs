use std::collections::HashMap;

use crate::asset_management::ResolvableAsset;

pub struct Shader {
    module: wgpu::ShaderModule,
}

impl Shader {
    pub fn new(device: &wgpu::Device, source: Box<dyn ResolvableAsset>) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                String::from_utf8(source.resolve()).unwrap().as_str().into(),
            ),
        });
        Shader { module }
    }

    pub fn get_module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
}

#[allow(dead_code)]
pub struct ShaderManager {
    shaders: HashMap<String, Shader>,
}
