use std::{result::Result, sync::Arc, time::Instant};

use egui_wgpu::ScreenDescriptor;
use winit::{
    error::EventLoopError,
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};

use super::{egui_renderer::EguiRenderer, wgpu_renderer::Renderer};

pub trait App: Sized {
    const SRGB: bool = true;

    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::empty(),
            ..wgpu::DownlevelCapabilities::default()
        }
    }

    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::downlevel_defaults() // These downlevel limits will allow the code to run on all possible hardware
    }

    fn init(_renderer: &mut Renderer) -> Self;

    fn gui(&mut self, _gui: &egui::Context, _renderer: &Renderer) {}
    fn gui_register(&mut self, _egui_renderer: &mut EguiRenderer, _renderer: &mut Renderer) {}
    fn resize(&mut self, _renderer: &mut Renderer) {}
    fn input(&mut self, _event: WindowEvent, _renderer: &mut Renderer) {}
    fn render(&mut self, _view: &wgpu::TextureView, _renderer: &mut Renderer) {}
}

struct FrameCounter {
    // Instant of the last time we printed the frame time.
    last_printed_instant: Instant,
    // Number of frames since the last time we printed the frame time.
    frame_count: u32,
}

impl FrameCounter {
    fn new() -> Self {
        Self {
            last_printed_instant: Instant::now(),
            frame_count: 0,
        }
    }

    fn update(&mut self) {
        self.frame_count += 1;
        let new_instant = Instant::now();
        let elapsed_secs = (new_instant - self.last_printed_instant).as_secs_f32();
        if elapsed_secs > 1.0 {
            let elapsed_ms = elapsed_secs * 1000.0;
            let frame_time = elapsed_ms / self.frame_count as f32;
            let fps = self.frame_count as f32 / elapsed_secs;
            tracing::info!("Frame time {:.2}ms ({:.1} FPS)", frame_time, fps);

            self.last_printed_instant = new_instant;
            self.frame_count = 0;
        }
    }
}

struct AppHandler<A: App> {
    app: A,
    renderer: Renderer,
    window: Arc<Window>,
    frame_counter: FrameCounter,
    egui_renderer: EguiRenderer,
    scale_factor: f32,
}

impl<A: App> AppHandler<A> {
    fn new(mut renderer: Renderer, window: Arc<Window>) -> Self {
        let egui_renderer =
            EguiRenderer::new(&renderer.device, renderer.config.format, None, 1, &window);
        Self {
            app: A::init(&mut renderer),
            renderer,
            window,
            frame_counter: FrameCounter::new(),
            egui_renderer,
            scale_factor: 1.0,
        }
    }
}
impl<A: App> AppHandler<A> {
    // fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    //     if self.window.is_none() {
    //         let window = Arc::new(
    //             event_loop
    //                 .create_window(WindowAttributes::default())
    //                 .unwrap(),
    //         );
    //         self.window = Some(window.clone());
    //         self.surface.resume(&self.context, window, A::SRGB);
    //         if self.app.is_none() {
    //             self.app = Some(A::init(
    //                 self.surface.config(),
    //                 &self.context.adapter,
    //                 &self.context.device,
    //                 &self.context.queue,
    //             ));
    //         }
    //     }
    // }

    // fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
    //     self.surface.suspend();
    // }

    fn event(
        &mut self,
        // event_loop: &winit::event_loop::ActiveEventLoop,
        event: Event<()>,
        elwt: &EventLoopWindowTarget<()>,
    ) {
        match event {
            Event::WindowEvent { window_id, event } => {
                if self.window.id() != window_id {
                    return;
                }
                let response = self.egui_renderer.handle_input(&self.window, &event);
                if response.repaint {
                    self.window.request_redraw();
                }
                match event {
                    WindowEvent::Resized(size) => {
                        self.renderer.resize(size);
                        self.app.resize(&mut self.renderer);
                        self.window.request_redraw();
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    }
                    | WindowEvent::CloseRequested => {
                        elwt.exit();
                    }

                    WindowEvent::RedrawRequested => {
                        self.frame_counter.update();

                        let frame = self.renderer.acquire();
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                            format: Some(self.renderer.config.view_formats[0]),
                            ..wgpu::TextureViewDescriptor::default()
                        });
                        self.app
                            .gui_register(&mut self.egui_renderer, &mut self.renderer);

                        self.app.render(&view, &mut self.renderer);

                        let mut encoder = self.renderer.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor { label: None },
                        );
                        let screen_descriptor = ScreenDescriptor {
                            size_in_pixels: [
                                self.renderer.config.width,
                                self.renderer.config.height,
                            ],
                            pixels_per_point: self.window.scale_factor() as f32 * self.scale_factor,
                        };

                        self.egui_renderer.draw(
                            &self.renderer.device,
                            &self.renderer.queue,
                            &mut encoder,
                            &self.window,
                            &view,
                            &screen_descriptor,
                            |ctx| self.app.gui(ctx, &self.renderer),
                        );
                        self.renderer.queue.submit(Some(encoder.finish()));
                        frame.present();

                        self.window.request_redraw();
                    }
                    _ => {
                        if !response.consumed {
                            self.app.input(event, &mut self.renderer);
                        };
                    }
                }
            }
            Event::AboutToWait => self.window.request_redraw(),
            _ => {}
        }
    }
}
#[allow(clippy::future_not_send)]
async fn start<A: App>() -> Result<(), EventLoopError> {
    tracing_subscriber::fmt().init();
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    event_loop.set_control_flow(ControlFlow::Poll);
    let renderer = Renderer::new::<A>(window.clone(), A::SRGB).await;
    let mut handler = AppHandler::<A>::new(renderer, window);
    tracing::info!("Entering event loop...");
    event_loop.run(move |event, elwt| {
        handler.event(event, elwt);
    })
}
#[allow(clippy::missing_errors_doc)]
pub fn run<A: App>() -> Result<(), EventLoopError> {
    pollster::block_on(start::<A>())
}
