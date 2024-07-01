use std::sync::{Arc, Mutex};
pub use wgpu;
pub use winit;

use winit::event_loop::ControlFlow;

mod app_handler;
pub mod asset_management;
pub mod events;
pub mod math;
pub mod rendering;
pub mod window;

static QUIT: Mutex<bool> = Mutex::new(false);

pub fn request_quit() {
    let mut quit = QUIT.lock().unwrap();
    *quit = true;
}

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
        #[cfg(not(target_os = "windows"))]
        let instance = wgpu::Instance::default();
        // NOTE: As of wgpu 0.20 there are some performance issues with the vulkan backend on
        // windows, so we will use dx12 for now
        #[cfg(target_os = "windows")]
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            ..Default::default()
        });
        // now lets init our windows' surfaces
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await;
        let adapter = match adapter {
            Some(adapter) => adapter,
            None => {
                eprintln!("No suitable adapter found, taika will now exit.");
                return;
            }
        };
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
        let windows = self.windows.clone();
        let device = Arc::new(Mutex::new(device));
        let queue = Arc::new(Mutex::new(queue));
        let mut state = app_handler::AppState {
            device,
            queue,
            windows,
            adapter,
            instance,
        };
        self.handle.run_app(&mut state).unwrap()
    }
}
