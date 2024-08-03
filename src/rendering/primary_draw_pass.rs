use std::{
    collections::BinaryHeap,
    sync::{Arc, Mutex},
};

use wgpu::{CommandEncoder, Device, Queue};

use crate::window::TargetProperties;

use super::{drawable::Drawable, RenderPass};

/// A basic [`RenderPass`] that draws drawables in order of their z value
pub struct PrimaryDrawPass {
    drawables: BinaryHeap<DrawableElement>,
    new_drawables: Vec<(Arc<Mutex<dyn Drawable>>, u32)>,
    name: String,
    target: Option<Arc<Mutex<wgpu::TextureView>>>,
    clear_color: wgpu::Color,
}

struct DrawableElement {
    drawable: Arc<Mutex<dyn Drawable>>,
    z: u32,
}

impl PartialEq for DrawableElement {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.drawable, &other.drawable)
    }
}

impl Eq for DrawableElement {}

impl PartialOrd for DrawableElement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DrawableElement {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.z.cmp(&other.z).reverse()
    }
}

impl PrimaryDrawPass {
    /// Initializes the render pass with a name (shown in error messages and renderdoc) and an optional target. If no target is provided the pass will use the previous or default target of the [`super::RenderPipeline`]
    pub fn new(name: &str, target: Option<Arc<Mutex<wgpu::TextureView>>>) -> Self {
        PrimaryDrawPass {
            drawables: BinaryHeap::new(),
            new_drawables: Vec::new(),
            name: name.to_string(),
            target,
            clear_color: wgpu::Color::TRANSPARENT,
        }
    }

    /// Adds a drawable to the render pass with a z value. The drawables will be drawn in order of their z value. The highest number is drawn last == visible on top
    pub fn add_drawable(&mut self, drawable: Arc<Mutex<dyn Drawable>>, z: u32) {
        self.new_drawables.push((drawable, z));
    }

    /// Removes the drawable from the render pass
    pub fn remove_drawable(&mut self, drawable: Arc<Mutex<dyn Drawable>>) {
        self.drawables
            .retain(|d| !Arc::ptr_eq(&d.drawable, &drawable));
        // if no frame has been rendered between adding the drawable and removing it, it will be in new_drawables
        // hence we have to check it aswell
        self.new_drawables.retain(|d| !Arc::ptr_eq(&d.0, &drawable));
    }

    /// Set the target of the render pass. If no target is provided the pass will use the previous or default target of the [`super::RenderPipeline`]
    pub fn set_target(&mut self, target: Option<Arc<Mutex<wgpu::TextureView>>>) {
        self.target = target;
    }

    /// Set the clear color (or background color) of the render pass
    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.clear_color = color;
    }

    /// Returns the number of drawables assigned to this [`RenderPass`]
    pub fn drawable_count(&self) -> usize {
        self.drawables.len()
    }
}

impl RenderPass for PrimaryDrawPass {
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
