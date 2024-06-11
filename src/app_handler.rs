use std::sync::{Arc, Mutex};

use winit::{application::ApplicationHandler, event::WindowEvent};

use crate::{window::Window, QUIT};

pub(crate) struct AppState<'a> {
    pub device: Arc<Mutex<wgpu::Device>>,
    pub queue: Arc<Mutex<wgpu::Queue>>,
    pub windows: Vec<Arc<Mutex<Window<'a>>>>,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
}

impl<'a> ApplicationHandler<()> for AppState<'a> {
    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, _event: ()) {}

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        for window in &self.windows {
            let window_attributes = winit::window::WindowAttributes::default()
                .with_title(window.lock().unwrap().title.clone())
                .with_min_inner_size(winit::dpi::LogicalSize::new(20.0, 20.0));
            let win = event_loop.create_window(window_attributes).unwrap();
            window.lock().unwrap().init(&self.instance, win).unwrap();
        }

        for window in &self.windows {
            window
                .lock()
                .unwrap()
                .configure_surface(&self.adapter, &self.device.lock().unwrap());
        }
        let windows = self.windows.clone();
        let device = self.device.clone();
        let queue = self.queue.clone();
        // we have to block with tokio
        futures::executor::block_on(async move {
            for window in &windows {
                window
                    .lock()
                    .unwrap()
                    .do_device_init(device.clone(), queue.clone())
                    .await;
            }
        });
        for window in &self.windows {
            window.lock().unwrap().request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if *QUIT.lock().unwrap() {
            event_loop.exit();
        }
        for window in self.windows.iter() {
            if window.lock().unwrap().get_window_id() == window_id {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        window.lock().unwrap().resize_surface(
                            &self.device.lock().unwrap(),
                            physical_size,
                            &self.queue.lock().unwrap(),
                        );
                    }
                    WindowEvent::CloseRequested => {
                        window.lock().unwrap().do_closed();
                        event_loop.exit();
                    }
                    WindowEvent::Focused(focused) => {
                        window.lock().unwrap().do_focus(focused);
                    }
                    WindowEvent::RedrawRequested => {
                        let mut window = window.lock().unwrap();
                        window.do_frame();
                        let surface = window.get_surface();
                        let frame = surface.get_current_texture();
                        if let Err(err) = frame {
                            println!("Failed to get current frame. Window state listed below:");
                            println!("Error: {:?}", err);
                            return;
                        }
                        let frame = frame.unwrap();
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                            format: Some(window.get_target_properties().view_format),
                            ..Default::default()
                        });
                        let mut encoder = self.device.lock().unwrap().create_command_encoder(
                            &wgpu::CommandEncoderDescriptor { label: None },
                        );
                        let pipeline = window.get_render_pipeline();
                        let pipeline = pipeline.lock();
                        pipeline.unwrap().render(
                            &self.device.lock().unwrap(),
                            &mut encoder,
                            &self.queue.lock().unwrap(),
                            &view,
                            window.get_target_properties(),
                        );
                        self.queue.lock().unwrap().submit(Some(encoder.finish()));
                        frame.present();
                        window.do_after_frame();
                        window.request_redraw();
                    }
                    _ => {}
                }
                window.lock().unwrap().do_window_event(&event);
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        _event: winit::event::DeviceEvent,
    ) {
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}
}
