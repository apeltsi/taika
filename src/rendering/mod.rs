use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use wgpu::{CommandEncoder, Device, Queue};

use crate::window::TargetProperties;

pub mod compute;
pub mod drawable;
mod primary_draw_pass;
pub mod shader;
pub mod vertex;
pub use primary_draw_pass::PrimaryDrawPass;

pub trait RenderPass {
    fn render(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        queue: &Queue,
        target: &wgpu::TextureView,
        global_bind_group: &wgpu::BindGroup,
        bind_group_layout: &wgpu::BindGroupLayout,
        target_properties: &TargetProperties,
    );

    fn init(
        &mut self,
        device: &Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        target_properties: &TargetProperties,
    );
}

pub trait RenderPipeline {
    fn render(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        queue: &Queue,
        target: &wgpu::TextureView,
        target_properties: &TargetProperties,
    );

    fn init(
        &mut self,
        device: &Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        target_properties: &TargetProperties,
    );
}

pub struct DefaultRenderPipeline {
    render_passes: Vec<Arc<Mutex<dyn RenderPass>>>,
    global_bind_group: Box<dyn GlobalBindGroup>,
    initialized: bool,
    #[allow(dead_code)]
    name: String,
}

impl DefaultRenderPipeline {
    pub fn new(global_bind_group: Box<dyn GlobalBindGroup>, name: &str) -> Self {
        DefaultRenderPipeline {
            render_passes: Vec::new(),
            initialized: false,
            global_bind_group,
            name: name.to_string(),
        }
    }

    pub fn add_render_pass(&mut self, render_pass: Arc<Mutex<dyn RenderPass>>) {
        self.render_passes.push(render_pass);
    }
}

impl RenderPipeline for DefaultRenderPipeline {
    fn render(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        queue: &Queue,
        target: &wgpu::TextureView,
        target_properties: &TargetProperties,
    ) {
        if !self.initialized {
            let bind_group_layout = self.global_bind_group.get_layout(device);
            self.init(device, &bind_group_layout, target_properties);
            self.initialized = true;
        }
        self.global_bind_group.pre_render(device, queue);
        for render_pass in &mut self.render_passes {
            render_pass.lock().unwrap().render(
                device,
                encoder,
                queue,
                target,
                &self.global_bind_group.get_group(),
                self.global_bind_group.get_layout(device).as_ref(),
                target_properties,
            )
        }
    }

    fn init(
        &mut self,
        device: &Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        target_properties: &TargetProperties,
    ) {
        self.global_bind_group.init(device);
        for pass in self.render_passes.iter_mut() {
            pass.lock()
                .unwrap()
                .init(device, bind_group_layout, target_properties);
        }
    }
}

/// A bind group that is applied render-pipeline wide
pub trait GlobalBindGroup {
    fn get_layout(&mut self, device: &wgpu::Device) -> Rc<wgpu::BindGroupLayout>;
    fn init(&mut self, device: &wgpu::Device);
    fn get_group(&mut self) -> Rc<wgpu::BindGroup>;
    fn pre_render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
}
