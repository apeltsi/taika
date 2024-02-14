use std::sync::{Arc, Mutex};

use crate::EventLoop;

pub struct Window<'a> {
    handle: Arc<winit::window::Window>,
    surface: Option<wgpu::Surface<'a>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    pub clear_color: wgpu::Color,
}

impl Window<'_> {
    pub fn new<'a>(event_loop: &mut EventLoop<'a>) -> Arc<Mutex<Window<'a>>> {
        let window = winit::window::WindowBuilder::new()
            .with_title("Taika window")
            .build(event_loop.get_event_loop())
            .unwrap();
        let window = Window {
            handle: Arc::new(window),
            surface: None,
            surface_config: None,
            clear_color: wgpu::Color::BLACK,
        };
        let window = Arc::new(Mutex::new(window));
        event_loop.windows.push(window.clone());
        window
    }

    pub(crate) fn init_surface(
        &mut self,
        instance: &wgpu::Instance,
    ) -> Result<(), wgpu::CreateSurfaceError> {
        let surface = instance.create_surface(self.handle.clone())?;
        self.surface = Some(surface);
        Ok(())
    }

    pub(crate) fn configure_surface(&mut self, adapter: &wgpu::Adapter, device: &wgpu::Device) {
        let size = self.handle.inner_size();
        let config = self
            .surface
            .as_mut()
            .unwrap()
            .get_default_config(adapter, size.width, size.height)
            .unwrap();
        self.surface.as_mut().unwrap().configure(device, &config);
        self.surface_config = Some(config);
    }

    pub(crate) fn resize_surface(
        &mut self,
        device: &wgpu::Device,
        size: winit::dpi::PhysicalSize<u32>,
    ) {
        let config = self.surface_config.as_ref().unwrap().clone();
        self.surface_config = Some(wgpu::SurfaceConfiguration {
            width: size.width.max(1),
            height: size.height.max(1),
            format: config.format,
            present_mode: config.present_mode,
            alpha_mode: config.alpha_mode,
            usage: config.usage,
            desired_maximum_frame_latency: config.desired_maximum_frame_latency,
            view_formats: config.view_formats.clone(),
        });
        self.surface.as_mut().unwrap().configure(device, &config);
    }

    pub fn get_handle(&self) -> &winit::window::Window {
        &self.handle
    }

    pub fn get_window_id(&self) -> winit::window::WindowId {
        self.handle.id()
    }

    pub(crate) fn get_surface<'a>(&'a self) -> &Option<wgpu::Surface<'a>> {
        &self.surface
    }

    pub fn set_title(&mut self, title: &str) {
        self.handle.set_title(title);
    }

    pub fn set_size(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // for now we dont care about the result
        let _ = self.handle.request_inner_size(size);
    }

    pub fn set_position(&mut self, position: winit::dpi::PhysicalPosition<i32>) {
        self.handle.set_outer_position(position);
    }

    pub fn set_fullscreen(&mut self, fullscreen: Option<winit::window::Fullscreen>) {
        self.handle.set_fullscreen(fullscreen);
    }
}
