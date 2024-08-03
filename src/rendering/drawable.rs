use wgpu::RenderPass;

use crate::window::TargetProperties;

/// A drawable object that can be drawn to the screen. If using the
/// [PrimaryDrawPass](crate::rendering::PrimaryDrawPass) the drawables assigned to it will be
/// initialized and then drawn in their order
pub trait Drawable {
    fn init(
        &mut self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        target_properties: &TargetProperties,
    );

    fn draw(
        &mut self,
        frame_num: u64,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut RenderPass, // NOTE: This is a wgpu render pass
        global_bind_group: &wgpu::BindGroup,
    );
}
