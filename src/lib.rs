//! A low-cost abstraction layer on top of [wgpu](https://crates.io/crates/wgpu) and [winit](https://crates.io/crates/winit) to make their APIs more ergonomic.
//!
//! # State
//! Taika is early in development, meaning large API changes are bound to happen. However it is currently being used for a production ready game
//! which serves as a good testbed for the library.
//!
//! # Goals
//! 1. Simplify window creation
//! 2. Introduce "RenderPasses" and "RenderPipelines", which are common tropes in game
//!    engines.
//! 3. Make API changes in WGPU and Winit less frustrating by providing a semi-stable API. API
//!    changes will still happen though.
//! 4. Give full access to WGPU
//!
//! In addition to these goals taika also includes some common utilities mainly targeted towards
//! game-development. Taika also includes a super basic form of asset management. It is designed to
//! be built upon, not to be a full-fledged asset management system.
//!
//! ## What taika doesn't do:
//! - Input-handling, you can do this yourself by listening to the winit events that are passed
//!   through to your event-handler
//! - Audio, use other libraries
//! - Make rendering easy. You still have to write shaders, and implement the drawable trait, to
//!   actually issue the drawcalls to the GPU. Taika doesn't make any drawcalls by itself
//!
//! # Notes
//! - The naming of [`rendering::RenderPass`] and [`rendering::RenderPipeline`] is a bit confusing at they are also used in
//!   wgpu.
//! - No examples currently!
//!
//!
//! # Getting Started
//! Use the [`EventLoop`] struct to get started
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

#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub vsync: bool,
    pub required_features: wgpu::Features,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            vsync: true,
            required_features: wgpu::Features::empty(),
        }
    }
}

/// Request the event loop to quit, closing all windows
pub fn request_quit() {
    let mut quit = QUIT.lock().unwrap();
    *quit = true;
}

/// Used to create windows and run the main loop of the application.
pub struct EventLoop<'a> {
    handle: winit::event_loop::EventLoop<()>,
    windows: Vec<Arc<Mutex<window::Window<'a>>>>,
    pub(crate) render_settings: RenderSettings,
}

impl<'a> EventLoop<'a> {
    /// Initializes a new taika event loop.
    /// The event loop is used to create windows and run the main loop of the application.
    pub fn new(
        render_settings: RenderSettings,
    ) -> Result<EventLoop<'a>, winit::error::EventLoopError> {
        let event_loop = winit::event_loop::EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);
        Ok(EventLoop {
            handle: event_loop,
            windows: Vec::new(),
            render_settings,
        })
    }

    /// Returns the underlying winit event loop
    pub fn get_event_loop(&self) -> &winit::event_loop::EventLoop<()> {
        &self.handle
    }

    /// Runs the event loop. This function will block until all windows are closed.
    pub async fn run(self) {
        let instance = wgpu::Instance::default();
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
                    required_features: self.render_settings.required_features,
                    required_limits: wgpu::Limits::downlevel_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::Performance,
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
            render_settings: self.render_settings.clone(),
        };
        self.handle.run_app(&mut state).unwrap()
    }
}
