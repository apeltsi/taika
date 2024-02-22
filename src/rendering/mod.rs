use wgpu::{CommandEncoder, Device, Queue};

use self::drawable::Drawable;

pub mod drawable;
pub mod shader;

pub trait RenderPass {
    fn render<'a>(
        &'a mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        queue: &Queue,
        target: &wgpu::TextureView,
        global_bind_group: &'a wgpu::BindGroup,
    );
}

pub trait RenderPipeline {
    fn render<'a>(
        &'a mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        queue: &Queue,
        target: &'a wgpu::TextureView,
    );
}

pub struct DefaultRenderPipeline {
    render_passes: Vec<Box<dyn RenderPass>>,
    global_bind_group: Option<wgpu::BindGroup>,
}

impl DefaultRenderPipeline {
    pub fn new() -> Self {
        DefaultRenderPipeline {
            render_passes: Vec::new(),
            global_bind_group: None,
        }
    }

    pub fn add_render_pass(&mut self, render_pass: Box<dyn RenderPass>) {
        self.render_passes.push(render_pass);
    }

    fn init(&mut self, device: &Device) {
        let camera_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Matrix Buffer"),
            size: (std::mem::size_of::<[[f32; 4]; 4]>() as u64) * 2,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let global_bind_group_layout = get_global_bind_group_layout(&device);
        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &camera_matrix_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new((std::mem::size_of::<[[f32; 4]; 4]>() as u64) * 2),
                }),
            }],
            label: None,
        });
        self.global_bind_group = Some(global_bind_group);
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
        if self.global_bind_group.is_none() {
            self.init(device);
        }
        for render_pass in &mut self.render_passes {
            render_pass.render(
                device,
                encoder,
                queue,
                target,
                self.global_bind_group.as_ref().unwrap(),
            );
        }
    }
}

pub struct PrimaryDrawPass<'a> {
    drawables: Vec<Box<dyn Drawable<'a>>>,
}

impl<'a> PrimaryDrawPass<'a> {
    pub fn new() -> Self {
        PrimaryDrawPass {
            drawables: Vec::new(),
        }
    }

    pub fn add_drawable(&mut self, drawable: Box<dyn Drawable<'a>>) {
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
        // Draw some basic shapes
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
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
        for d in self.drawables.iter_mut() {
            d.draw(0, device, queue, &mut rpass, &global_bind_group);
        }
    }
}

pub fn get_global_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}
