use wgpu::{CommandEncoder, Device};

pub mod shader;

pub trait RenderPass {
    fn render(&mut self, device: &Device, encoder: &mut CommandEncoder, target: &wgpu::TextureView);
}

pub trait RenderPipeline {
    fn render(&mut self, device: &Device, encoder: &mut CommandEncoder, target: &wgpu::TextureView);
}

pub struct DefaultRenderPipeline {
    render_passes: Vec<Box<dyn RenderPass>>,
}

impl DefaultRenderPipeline {
    pub fn new() -> Self {
        DefaultRenderPipeline {
            render_passes: Vec::new(),
        }
    }

    pub fn add_render_pass(&mut self, render_pass: Box<dyn RenderPass>) {
        self.render_passes.push(render_pass);
    }
}

impl RenderPipeline for DefaultRenderPipeline {
    fn render(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        for render_pass in &mut self.render_passes {
            render_pass.render(device, encoder, target);
        }
    }
}
