use std::sync::{Arc, Mutex};

use super::RenderPass;

pub trait ComputeTask {
    fn init<'a>(&'a mut self, device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout);
    fn compute<'a>(
        &'a mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        global_bind_group: &'a wgpu::BindGroup,
    );
}

pub struct ComputePass {
    tasks: Vec<Arc<Mutex<dyn ComputeTask>>>,
    initialized: bool,
    name: String,
}

impl ComputePass {
    pub fn new(name: &str) -> Self {
        ComputePass {
            tasks: Vec::new(),
            initialized: false,
            name: name.to_string(),
        }
    }

    pub fn add_task(&mut self, task: Arc<Mutex<dyn ComputeTask>>) {
        self.tasks.push(task);
    }
}

impl RenderPass for ComputePass {
    fn render<'a>(
        &'a mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        _target: &wgpu::TextureView,
        global_bind_group: &'a wgpu::BindGroup,
    ) {
        if !self.initialized {
            panic!("ComputePass '{}' not initialized", self.name);
        }

        for task in &mut self.tasks {
            task.lock()
                .unwrap()
                .compute(device, encoder, queue, global_bind_group);
        }
    }

    fn init<'a>(
        &'a mut self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        _target_properties: &super::TargetProperties,
    ) {
        for task in &mut self.tasks {
            task.lock().unwrap().init(device, bind_group_layout);
        }
        self.initialized = true;
    }
}
