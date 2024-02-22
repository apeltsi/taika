use std::sync::{Arc, Mutex};

use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

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
        event_loop.set_control_flow(ControlFlow::Wait);
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
                force_fallback_adapter: false,
                compatible_surface: self.windows[0].lock().unwrap().get_surface().as_ref(),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");
        let adapter_info = adapter.get_info();
        println!(
            "Backend: {:?} | Adapter: {:?} | Driver: {:?}",
            adapter_info.backend, adapter_info.name, adapter_info.driver_info
        );

        for window in &self.windows {
            window.lock().unwrap().configure_surface(&adapter, &device);
        }
        let windows = self.windows.clone();
        let mut first_frame = true;
        self.handle
            .run(move |event, window_target| {
                if first_frame {
                    // lets request a redraw for all windows
                    for window in windows.iter() {
                        window.lock().unwrap().request_redraw();
                    }
                    first_frame = false;
                }
                #[cfg(not(target_arch = "wasm32"))]
                window_target.set_control_flow(ControlFlow::Poll);
                #[cfg(target_arch = "wasm32")]
                window_target.set_control_flow(ControlFlow::Wait);
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
                                    let frame = surface.get_current_texture();
                                    if let Err(err) = frame {
                                        println!("Failed to get current frame. Window state listed below:");
                                        window.print_debug();
                                        println!("Error: {:?}", err);
                                        return;
                                    }
                                    let frame = frame.unwrap();
                                    let view = frame
                                        .texture
                                        .create_view(&wgpu::TextureViewDescriptor::default());
                                    let mut encoder = device.create_command_encoder(
                                        &wgpu::CommandEncoderDescriptor { label: None },
                                    );
                                    window.get_render_pipeline().lock().unwrap().render(
                                        &device,
                                        &mut encoder,
                                        &queue,
                                        &view,
                                    );
                                    queue.submit(Some(encoder.finish()));
                                    frame.present();
                                    window.request_redraw();
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
