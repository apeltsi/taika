use std::sync::{Arc, Mutex};
pub use wgpu;
pub use winit;

use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

pub mod asset_management;
pub mod events;
pub mod math;
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
                    required_features: wgpu::Features::VERTEX_WRITABLE_STORAGE, // TODO: Make this configurable
                    required_limits: wgpu::Limits::downlevel_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");
        for window in &self.windows {
            window.lock().unwrap().configure_surface(&adapter, &device);
        }
        let windows = self.windows.clone();
        let mut first_frame = true;
        let device = Arc::new(Mutex::new(device));
        let queue = Arc::new(Mutex::new(queue));
        for window in &self.windows {
            window
                .lock()
                .unwrap()
                .do_device_init(device.clone(), queue.clone()).await;
        }
        self.handle
            .run(move |event, window_target| {
                if first_frame {
                    // lets request a redraw for all windows
                    for window in windows.iter() {
                        let mut window = window.lock().unwrap();
                        window.request_redraw();
                        window.do_open();
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
                                        .resize_surface(&device.lock().unwrap(), *physical_size, &queue.lock().unwrap());
                                }
                                WindowEvent::CloseRequested => {
                                    window.lock().unwrap().do_closed();
                                    window_target.exit();
                                },
                                WindowEvent::Focused(focused) => {
                                    window.lock().unwrap().do_focus(*focused);
                                }
                                WindowEvent::RedrawRequested => {
                                    let mut window = window.lock().unwrap();
                                    window.do_frame();
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
                                        .create_view(&wgpu::TextureViewDescriptor {
                                            format: Some(window.get_target_properties().view_format),
                                            ..Default::default()
                                        });
                                    let mut encoder = device.lock().unwrap().create_command_encoder(
                                        &wgpu::CommandEncoderDescriptor { label: None },
                                    );
                                    let pipeline = window.get_render_pipeline();
                                    let pipeline = pipeline.lock();
                                    pipeline.unwrap().render(
                                        &device.lock().unwrap(),
                                        &mut encoder,
                                        &queue.lock().unwrap(),
                                        &view,
                                        window.get_target_properties()
                                    );
                                    queue.lock().unwrap().submit(Some(encoder.finish()));
                                    frame.present();
                                    window.do_after_frame();
                                    window.request_redraw();
                                },
                                _ => {}
                            }
                            window.lock().unwrap().do_window_event(event);
                        }
                    }
                }
            })
            .unwrap();
    }
}
