use std::sync::{Arc, Mutex};

use winit::event::{Event, WindowEvent};

pub mod asset_management;
pub mod rendering;
pub mod window;

pub struct EventLoop<'a> {
    handle: winit::event_loop::EventLoop<()>,
    windows: Vec<Arc<Mutex<window::Window<'a>>>>,
}

impl<'a> EventLoop<'a> {
    pub fn new() -> Result<EventLoop<'a>, winit::error::EventLoopError> {
        let event_loop = winit::event_loop::EventLoop::new()?;
        Ok(EventLoop {
            handle: event_loop,
            windows: Vec::new(),
        })
    }

    pub fn get_event_loop(&self) -> &winit::event_loop::EventLoop<()> {
        &self.handle
    }

    pub async fn run(self) {
        let instance = wgpu::Instance::default();
        // now lets init our windows' surfaces
        for window in &self.windows {
            window.lock().unwrap().init_surface(&instance).unwrap();
        }
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: self.windows[0].lock().unwrap().get_surface().as_ref(),
                ..Default::default()
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .expect("Failed to create device");
        for window in &self.windows {
            window.lock().unwrap().configure_surface(&adapter, &device);
        }
        let windows = self.windows.clone();
        self.handle
            .run(move |event, window_target| {
                if let Event::WindowEvent { window_id, event } = &event {
                    for window in windows.iter() {
                        if window.lock().unwrap().get_window_id() == *window_id {
                            match event {
                                WindowEvent::Resized(physical_size) => {
                                    window
                                        .lock()
                                        .unwrap()
                                        .resize_surface(&device, *physical_size);
                                }
                                WindowEvent::CloseRequested => window_target.exit(),
                                WindowEvent::RedrawRequested => {
                                    let window = window.lock().unwrap();
                                    let surface = window.get_surface().as_ref().unwrap();
                                    let frame = surface.get_current_texture().unwrap();
                                    let view = frame
                                        .texture
                                        .create_view(&wgpu::TextureViewDescriptor::default());
                                    let mut encoder = device.create_command_encoder(
                                        &wgpu::CommandEncoderDescriptor { label: None },
                                    );
                                    window.get_render_pipeline().lock().unwrap().render(
                                        &device,
                                        &mut encoder,
                                        &view,
                                    );
                                    queue.submit(std::iter::once(encoder.finish()));
                                    frame.present();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            })
            .unwrap();
    }
}
