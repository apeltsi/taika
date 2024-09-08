use std::sync::{Arc, Mutex};

use winit::dpi::PhysicalSize;

use crate::{events::EventHandler, rendering::RenderPipeline, EventLoop, RenderSettings};

/// Represents a window
pub struct Window<'a> {
    instance: Option<WindowInstance<'a>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Arc<Mutex<dyn RenderPipeline>>,
    event_handler: Box<dyn EventHandler>,
    target_properties: TargetProperties,
    pub(crate) title: String,
}

pub struct WindowInstance<'a> {
    pub handle: Arc<winit::window::Window>,
    pub surface: wgpu::Surface<'a>,
}

impl Window<'_> {
    /// Creates a new window, the title can be set by calling [`Window::set_title`]
    pub fn new<'a>(
        event_loop: &mut EventLoop<'a>,
        pipeline: Arc<Mutex<dyn RenderPipeline>>,
        event_handler: Box<dyn EventHandler>,
    ) -> Arc<Mutex<Window<'a>>> {
        let window = Window {
            instance: None,
            surface_config: None,
            render_pipeline: pipeline,
            event_handler,
            target_properties: TargetProperties {
                format: wgpu::TextureFormat::Rgba8UnormSrgb, // Temporary values will be replaced
                view_format: wgpu::TextureFormat::Rgba8Unorm, // later in runtime
            },
            title: "Taika Window".to_string(),
        };
        let window = Arc::new(Mutex::new(window));
        event_loop.windows.push(window.clone());
        window
    }

    pub(crate) fn init(
        &mut self,
        instance: &wgpu::Instance,
        window: winit::window::Window,
    ) -> Result<(), wgpu::CreateSurfaceError> {
        let window = Arc::new(window);
        let surface = instance.create_surface(window.clone())?;
        self.instance = Some(WindowInstance {
            handle: window,
            surface,
        });
        Ok(())
    }

    pub(crate) fn request_redraw(&self) {
        self.instance.as_ref().unwrap().handle.request_redraw();
    }

    pub(crate) fn configure_surface(
        &mut self,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        render_settings: &RenderSettings,
    ) {
        let size = self.instance.as_ref().unwrap().handle.inner_size();
        let size: PhysicalSize<u32> = (size.width.max(1), size.height.max(1)).into();

        let swapchain_capabilities = self
            .instance
            .as_ref()
            .unwrap()
            .surface
            .get_capabilities(adapter);
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
        let mut present_mode = wgpu::PresentMode::AutoNoVsync;
        if render_settings.vsync {
            present_mode = wgpu::PresentMode::AutoVsync;
        }
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.target_properties.format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![self.target_properties.view_format],
            desired_maximum_frame_latency: 1,
        };
        self.instance
            .as_ref()
            .unwrap()
            .surface
            .configure(device, &config);
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
        self.instance
            .as_ref()
            .unwrap()
            .surface
            .configure(device, self.surface_config.as_ref().unwrap());
        self.event_handler
            .window_resize(size.width.max(1), size.height.max(1), device, queue)
    }

    /// Returns the underlying winit window handle
    pub fn get_handle(&self) -> &winit::window::Window {
        &self.instance.as_ref().unwrap().handle
    }

    /// Returns the window id
    pub fn get_window_id(&self) -> winit::window::WindowId {
        self.instance.as_ref().unwrap().handle.id()
    }

    pub(crate) fn get_surface<'a>(&'a self) -> &wgpu::Surface<'a> {
        &self.instance.as_ref().unwrap().surface
    }

    /// Sets the title of the window
    pub fn set_title(&mut self, title: &str) {
        if self.instance.is_none() {
            self.title = title.to_string();
        } else {
            self.instance.as_ref().unwrap().handle.set_title(title);
        }
    }

    /// Resize the window. This might fail silently
    pub fn set_size(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // for now we dont care about the result
        let _ = self
            .instance
            .as_ref()
            .unwrap()
            .handle
            .request_inner_size(size);
    }

    /// Sets the position of the window
    pub fn set_position(&mut self, position: winit::dpi::PhysicalPosition<i32>) {
        self.instance
            .as_ref()
            .unwrap()
            .handle
            .set_outer_position(position);
    }

    /// Sets the window to fullscreen
    pub fn set_fullscreen(&mut self, fullscreen: Option<winit::window::Fullscreen>) {
        self.instance
            .as_ref()
            .unwrap()
            .handle
            .set_fullscreen(fullscreen);
    }

    /// Sets the visibility of the cursor
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.instance
            .as_ref()
            .unwrap()
            .handle
            .set_cursor_visible(visible);
    }

    /// Returns the taika [`RenderPipeline`]
    pub fn get_render_pipeline(&self) -> Arc<Mutex<dyn RenderPipeline>> {
        self.render_pipeline.clone()
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

    pub(crate) fn do_closed(&mut self) {
        self.event_handler.window_close();
    }

    pub(crate) fn do_device_init(
        &mut self,
        adapter: &wgpu::Adapter,
        device: Arc<Mutex<wgpu::Device>>,
        queue: Arc<Mutex<wgpu::Queue>>,
    ) {
        self.event_handler
            .device_init(adapter, device, queue, self.target_properties.clone());
    }

    pub(crate) fn do_window_event(&mut self, event: &winit::event::WindowEvent) {
        self.event_handler.window_event(event);
    }

    /// Returns the `TargetProperties` of this window
    pub fn get_target_properties(&self) -> &TargetProperties {
        &self.target_properties
    }
}

/// Info about the texture format used by the window
#[derive(Debug, Clone)]
pub struct TargetProperties {
    pub format: wgpu::TextureFormat,
    pub view_format: wgpu::TextureFormat,
}
