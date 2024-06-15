use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use wgpu::{CommandEncoder, Device, Queue};

use crate::window::TargetProperties;

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
        bind_group_layout: &wgpu::BindGroupLayout,
        target_properties: &TargetProperties,
    );

    fn init<'a>(
        &'a mut self,
        device: &Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        target_properties: &TargetProperties,
    );
}

pub trait RenderPipeline {
    fn render<'a>(
        &'a mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        queue: &Queue,
        target: &'a wgpu::TextureView,
        target_properties: &TargetProperties,
    );

    fn init<'a>(
        &'a mut self,
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

pub struct PrimaryDrawPass<'a> {
    drawables: Vec<Arc<Mutex<dyn Drawable<'a>>>>,
    new_drawables: Vec<Arc<Mutex<dyn Drawable<'a>>>>,
    name: String,
    target: Option<Arc<Mutex<wgpu::TextureView>>>,
    clear_color: wgpu::Color,
}

impl<'a> PrimaryDrawPass<'a> {
    pub fn new(name: &str, target: Option<Arc<Mutex<wgpu::TextureView>>>) -> Self {
        PrimaryDrawPass {
            drawables: Vec::new(),
            new_drawables: Vec::new(),
            name: name.to_string(),
            target,
            clear_color: wgpu::Color::TRANSPARENT,
        }
    }

    pub fn add_drawable(&mut self, drawable: Arc<Mutex<dyn Drawable<'a>>>) {
        self.new_drawables.push(drawable);
    }

    pub fn remove_drawable(&mut self, drawable: Arc<Mutex<dyn Drawable<'a>>>) {
        self.drawables.retain(|d| !Arc::ptr_eq(d, &drawable));
        // if no frame has been rendered between adding the drawable and removing it, it will be in new_drawables
        // hence we have to check it aswell
        self.new_drawables.retain(|d| !Arc::ptr_eq(d, &drawable));
    }

    pub fn set_target(&mut self, target: Option<Arc<Mutex<wgpu::TextureView>>>) {
        self.target = target;
    }

    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.clear_color = color;
    }

    pub fn drawable_count(&self) -> usize {
        self.drawables.len()
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
        bind_group_layout: &wgpu::BindGroupLayout,
        target_properties: &TargetProperties,
    ) {
        for d in self.new_drawables.drain(..) {
            d.lock()
                .unwrap()
                .init(device, bind_group_layout, target_properties);
            self.drawables.push(d);
        }
        let mut drawables = Vec::new();
        for d in self.drawables.iter_mut() {
            drawables.push(d.lock().unwrap());
        }
        let lock = if self.target.is_some() {
            Some(self.target.as_ref().unwrap().lock().unwrap())
        } else {
            None
        };
        let target = if let Some(lock) = lock.as_ref() {
            lock
        } else {
            target
        };
        // Start wgpu render pass
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&self.name),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        for d in drawables.iter_mut() {
            d.draw(0, device, queue, &mut rpass, global_bind_group);
        }
    }

    fn init<'a>(
        &'a mut self,
        device: &Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        target_properties: &TargetProperties,
    ) {
        for d in self.drawables.iter_mut() {
            d.lock()
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
