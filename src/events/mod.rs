use async_trait::async_trait;
use std::sync::{Arc, Mutex};

#[async_trait]
pub trait EventHandler {
    fn window_open(&mut self);
    fn window_close(&mut self);
    fn window_resize(
        &mut self,
        width: u32,
        height: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );
    fn window_focus(&mut self);
    fn window_unfocus(&mut self);
    fn window_frame(&mut self);
    fn window_after_frame(&mut self);
    fn device_init(&mut self, device: Arc<Mutex<wgpu::Device>>, queue: Arc<Mutex<wgpu::Queue>>);
    fn window_event(&mut self, event: &winit::event::WindowEvent);
}
