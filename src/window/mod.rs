use std::sync::{Arc, Mutex};

use winit::dpi::PhysicalSize;

use crate::{events::EventHandler, rendering::RenderPipeline, EventLoop};

pub struct Window<'a> {
    handle: Arc<winit::window::Window>,
    surface: Option<wgpu::Surface<'a>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Arc<Mutex<dyn RenderPipeline>>,
    event_handler: Box<dyn EventHandler>,
    target_properties: TargetProperties,
}

impl Window<'_> {
    pub fn new<'a>(
        event_loop: &mut EventLoop<'a>,
        pipeline: Arc<Mutex<dyn RenderPipeline>>,
        event_handler: Box<dyn EventHandler>,
    ) -> Arc<Mutex<Window<'a>>> {
        let window = winit::window::WindowBuilder::new()
            .with_title("Taika window")
            .with_min_inner_size(winit::dpi::LogicalSize::new(20.0, 20.0))
            .build(event_loop.get_event_loop())
            .unwrap();
        let window = Window {
            handle: Arc::new(window),
            surface: None,
            surface_config: None,
            render_pipeline: pipeline,
            event_handler,
            target_properties: TargetProperties {
                format: wgpu::TextureFormat::Rgba8UnormSrgb, // Temporary values will be replaced
                view_format: wgpu::TextureFormat::Rgba8Unorm, // later in runtime
            },
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

    pub(crate) fn request_redraw(&self) {
        self.handle.request_redraw();
    }

    pub(crate) fn configure_surface(&mut self, adapter: &wgpu::Adapter, device: &wgpu::Device) {
        let size = self.handle.inner_size();
        let size: PhysicalSize<u32> = (size.width.max(1), size.height.max(1)).into();

        let swapchain_capabilities = self.surface.as_mut().unwrap().get_capabilities(adapter);
        let mut swapchain_format = swapchain_capabilities.formats[0];

        #[cfg(not(target_arch = "wasm32"))]
        {
            swapchain_format = swapchain_format.add_srgb_suffix();
            self.target_properties.format = swapchain_format.remove_srgb_suffix();
        }
        self.target_properties.view_format = swapchain_format;
        #[cfg(target_arch = "wasm32")]
        {
            self.target_properties.format = swapchain_format;
        }
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.target_properties.format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![self.target_properties.view_format],
            desired_maximum_frame_latency: 2,
        };
        self.surface.as_mut().unwrap().configure(device, &config);
        self.surface_config = Some(config);
    }

    pub(crate) fn resize_surface(
        &mut self,
        device: &wgpu::Device,
        size: winit::dpi::PhysicalSize<u32>,
        queue: &wgpu::Queue,
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
        self.surface
            .as_mut()
            .unwrap()
            .configure(device, self.surface_config.as_ref().unwrap());
        self.event_handler
            .window_resize(size.width.max(1), size.height.max(1), device, queue)
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

    pub fn get_render_pipeline(&self) -> Arc<Mutex<dyn RenderPipeline>> {
        self.render_pipeline.clone()
    }

    pub fn print_debug(&self) {
        // print the title, size, current state of the window
        println!("Title: {}", self.handle.title());
        println!("Size: {:?}", self.handle.inner_size());
        println!("Position: {:?}", self.handle.outer_position());
        println!("Fullscreen: {:?}", self.handle.fullscreen());
        println!("Visible: {:?}", self.handle.is_visible());
        if let Some(surface) = &self.surface {
            println!("Surface Ok: {:?}", surface.get_current_texture().is_ok());
        }
    }

    pub(crate) fn do_frame(&mut self) {
        self.event_handler.window_frame();
    }

    pub(crate) fn do_after_frame(&mut self) {
        self.event_handler.window_after_frame();
    }

    pub(crate) fn do_focus(&mut self, focused: bool) {
        if focused {
            self.event_handler.window_focus();
        } else {
            self.event_handler.window_unfocus();
        }
    }

    pub(crate) fn do_open(&mut self) {
        self.event_handler.window_open();
    }

    pub(crate) fn do_closed(&mut self) {
        self.event_handler.window_close();
    }

    pub(crate) async fn do_device_init(
        &mut self,
        device: Arc<Mutex<wgpu::Device>>,
        queue: Arc<Mutex<wgpu::Queue>>,
    ) {
        self.event_handler.device_init(device, queue).await;
    }

    pub(crate) fn do_window_event(&mut self, event: &winit::event::WindowEvent) {
        self.event_handler.window_event(event);
    }

    pub fn get_target_properties(&self) -> &TargetProperties {
        &self.target_properties
    }
}

#[derive(Debug, Clone)]
pub struct TargetProperties {
    pub format: wgpu::TextureFormat,
    pub view_format: wgpu::TextureFormat,
}
