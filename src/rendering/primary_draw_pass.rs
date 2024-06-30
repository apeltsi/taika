use std::{
    collections::BinaryHeap,
    sync::{Arc, Mutex},
};

use wgpu::{CommandEncoder, Device, Queue};

use crate::window::TargetProperties;

use super::{drawable::Drawable, RenderPass};

pub struct PrimaryDrawPass<'a> {
    drawables: BinaryHeap<DrawableElement<'a>>,
    new_drawables: Vec<(Arc<Mutex<dyn Drawable<'a>>>, u32)>,
    name: String,
    target: Option<Arc<Mutex<wgpu::TextureView>>>,
    clear_color: wgpu::Color,
}

struct DrawableElement<'a> {
    drawable: Arc<Mutex<dyn Drawable<'a>>>,
    z: u32,
}

impl PartialEq for DrawableElement<'_> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.drawable, &other.drawable)
    }
}

impl Eq for DrawableElement<'_> {}

impl PartialOrd for DrawableElement<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DrawableElement<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.z.cmp(&other.z).reverse()
    }
}

impl<'a> PrimaryDrawPass<'a> {
    pub fn new(name: &str, target: Option<Arc<Mutex<wgpu::TextureView>>>) -> Self {
        PrimaryDrawPass {
            drawables: BinaryHeap::new(),
            new_drawables: Vec::new(),
            name: name.to_string(),
            target,
            clear_color: wgpu::Color::TRANSPARENT,
        }
    }

    pub fn add_drawable(&mut self, drawable: Arc<Mutex<dyn Drawable<'a>>>, z: u32) {
        self.new_drawables.push((drawable, z));
    }

    pub fn remove_drawable(&mut self, drawable: Arc<Mutex<dyn Drawable<'a>>>) {
        self.drawables
            .retain(|d| !Arc::ptr_eq(&d.drawable, &drawable));
        // if no frame has been rendered between adding the drawable and removing it, it will be in new_drawables
        // hence we have to check it aswell
        self.new_drawables.retain(|d| !Arc::ptr_eq(&d.0, &drawable));
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
            d.0.lock()
                .unwrap()
                .init(device, bind_group_layout, target_properties);
            self.drawables.push(DrawableElement {
                drawable: d.0,
                z: d.1,
            });
        }
        let mut drawables = Vec::new();
        for d in self.drawables.iter() {
            drawables.push(d.drawable.lock().unwrap());
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

    fn init(
        &mut self,
        device: &Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        target_properties: &TargetProperties,
    ) {
        for d in self.drawables.iter() {
            d.drawable
                .lock()
                .unwrap()
                .init(device, bind_group_layout, target_properties);
        }
    }
}
