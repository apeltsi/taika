use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use wgpu::{CommandEncoder, Device, Queue};

use self::drawable::Drawable;

pub mod compute;
pub mod drawable;
pub mod shader;
pub mod vertex;

pub trait RenderPass {
    fn render<'a>(
        &'a mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        queue: &Queue,
        target: &wgpu::TextureView,
        global_bind_group: &'a wgpu::BindGroup,
    );

    fn init<'a>(&'a mut self, device: &Device, bind_group_layout: &wgpu::BindGroupLayout);
}

pub trait RenderPipeline {
    fn render<'a>(
        &'a mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        queue: &Queue,
        target: &'a wgpu::TextureView,
    );

    fn init<'a>(&'a mut self, device: &Device, bind_group_layout: &wgpu::BindGroupLayout);
}

pub struct DefaultRenderPipeline {
    render_passes: Vec<Box<dyn RenderPass>>,
    global_bind_group: Box<dyn GlobalBindGroup>,
    initialized: bool,
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

    pub fn add_render_pass(&mut self, render_pass: Box<dyn RenderPass>) {
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
    ) {
        if !self.initialized {
            let bind_group_layout = self.global_bind_group.get_layout(device);
            self.init(device, &bind_group_layout);
            self.initialized = true;
        }
        self.global_bind_group.pre_render(device, queue);
        for render_pass in &mut self.render_passes {
            render_pass.render(
                device,
                encoder,
                queue,
                target,
                &self.global_bind_group.get_group(),
            )
        }
    }

    fn init<'a>(&'a mut self, device: &Device, bind_group_layout: &wgpu::BindGroupLayout) {
        self.global_bind_group.init(device);
        for pass in self.render_passes.iter_mut() {
            pass.init(device, bind_group_layout);
        }
    }
}

pub struct PrimaryDrawPass<'a> {
    drawables: Vec<Arc<Mutex<dyn Drawable<'a>>>>,
    name: String,
}

impl<'a> PrimaryDrawPass<'a> {
    pub fn new(name: &str) -> Self {
        PrimaryDrawPass {
            drawables: Vec::new(),
            name: name.to_string(),
        }
    }

    pub fn add_drawable(&mut self, drawable: Arc<Mutex<dyn Drawable<'a>>>) {
        self.drawables.push(drawable);
    }
}

impl RenderPass for PrimaryDrawPass<'_> {
    fn render(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        queue: &Queue,
        target: &wgpu::TextureView,
        global_bind_group: &wgpu::BindGroup,
    ) {
        // first lets acquire all mutexes
        let mut drawables = Vec::new();
        for d in self.drawables.iter_mut() {
            drawables.push(d.lock().unwrap());
        }
        // Draw some basic shapes
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&self.name),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        for d in drawables.iter_mut() {
            d.draw(0, device, queue, &mut rpass, &global_bind_group);
        }
    }

    fn init<'a>(&'a mut self, device: &Device, bind_group_layout: &wgpu::BindGroupLayout) {
        for d in self.drawables.iter_mut() {
            d.lock().unwrap().init(device, bind_group_layout);
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
