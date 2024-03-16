use wgpu::RenderPass;

use crate::window::TargetProperties;

pub trait Drawable<'b> {
    fn init<'a>(
        &mut self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        target_properties: &TargetProperties,
    ) where
        'a: 'b;

    fn draw<'a>(
        &'a mut self,
        frame_num: u64,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut RenderPass<'a>, // NOTE: This is a wgpu render pass
        global_bind_group: &'a wgpu::BindGroup,
    );
}
