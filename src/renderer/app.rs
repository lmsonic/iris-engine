use std::{result::Result, sync::Arc, time::Instant};

use egui_wgpu::ScreenDescriptor;
use winit::{
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};

use super::gui::EguiRenderer;

pub trait App: 'static + Sized {
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

    fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self;

    fn gui(&mut self, _ctx: &egui::Context, _queue: &wgpu::Queue) {}
    fn resize(
        &mut self,
        _config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
    }

    fn input(&mut self, _event: WindowEvent, _queue: &wgpu::Queue) {}

    fn render(&mut self, _view: &wgpu::TextureView, _device: &wgpu::Device, _queue: &wgpu::Queue) {}
}

/// Wrapper type which manages the surface and surface configuration.
///
/// As surface usage varies per platform, wrapping this up cleans up the event loop code.
#[derive(Debug)]
pub struct SurfaceWrapper {
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl SurfaceWrapper {
    // /// Create a new surface wrapper with no surface or configuration.
    // const fn new() -> Self {
    //     Self {
    //         surface: None,
    //         config: None,
    //     }
    // }

    /// Called when an event which matches [`Self::start_condition`] is received.
    ///
    /// On all native platforms, this is where we create the surface.
    ///
    /// Additionally, we configure the surface based on the (now valid) window size.
    fn new(context: &AppContext, window: Arc<Window>, srgb: bool) -> Self {
        // Window size is only actually valid after we enter the event loop.
        let window_size = window.inner_size();
        let width = window_size.width.max(1);
        let height = window_size.height.max(1);

        tracing::info!("Surface resume {window_size:?}");

        // We didn't create the surface in pre_adapter, so we need to do so now.
        let surface = context.instance.create_surface(window).unwrap();

        // Get the default configuration,
        let mut config = surface
            .get_default_config(&context.adapter, width, height)
            .expect("Surface isn't supported by the adapter.");
        if srgb {
            // Not all platforms (WebGPU) support sRGB swapchains, so we need to use view formats
            let view_format = config.format.add_srgb_suffix();
            config.view_formats.push(view_format);
        } else {
            // All platforms support non-sRGB swapchains, so we can just use the format directly.
            let format = config.format.remove_srgb_suffix();
            config.format = format;
            config.view_formats.push(format);
        };

        surface.configure(&context.device, &config);
        let config = config;
        Self { surface, config }
    }

    /// Resize the surface, making sure to not resize to zero.
    fn resize(&mut self, context: &AppContext, size: PhysicalSize<u32>) {
        tracing::info!("Surface resize {size:?}");

        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&context.device, &self.config);
    }

    /// Acquire the next surface texture.
    fn acquire(&mut self, context: &AppContext) -> wgpu::SurfaceTexture {
        match self.surface.get_current_texture() {
            Ok(frame) => frame,
            // If we timed out, just try again
            Err(wgpu::SurfaceError::Timeout) => self.surface
                .get_current_texture()
                .expect("Failed to acquire next surface texture!"),
            Err(
                // If the surface is outdated, or was lost, reconfigure it.
                wgpu::SurfaceError::Outdated
                | wgpu::SurfaceError::Lost
                // If OutOfMemory happens, reconfiguring may not help, but we might as well try
                | wgpu::SurfaceError::OutOfMemory,
            ) => {
                self.surface.configure(&context.device, &self.config);
                self.surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        }
    }
}

/// Context containing global wgpu resources.
#[derive(Debug)]
pub struct AppContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}
impl AppContext {
    /// Initializes the example context.
    async fn init_async<A: App>() -> Self {
        tracing::info!("Initializing wgpu...");

        let backends = wgpu::util::backend_bits_from_env().unwrap_or_default();
        let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
        let gles_minor_version = wgpu::util::gles_minor_version_from_env().unwrap_or_default();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::from_build_config().with_env(),
            dx12_shader_compiler,
            gles_minor_version,
        });

        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, None)
            .await
            .expect("No suitable GPU adapters found on the system!");

        let adapter_info = adapter.get_info();
        tracing::info!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

        let optional_features = A::optional_features();
        let required_features = A::required_features();
        let adapter_features = adapter.features();
        assert!(
            adapter_features.contains(required_features),
            "Adapter does not support required features for this example: {:?}",
            required_features - adapter_features
        );

        let required_downlevel_capabilities = A::required_downlevel_capabilities();
        let downlevel_capabilities = adapter.get_downlevel_capabilities();
        assert!(
            downlevel_capabilities.shader_model >= required_downlevel_capabilities.shader_model,
            "Adapter does not support the minimum shader model required to run this example: {:?}",
            required_downlevel_capabilities.shader_model
        );
        assert!(
            downlevel_capabilities
                .flags
                .contains(required_downlevel_capabilities.flags),
            "Adapter does not support the downlevel capabilities required to run this example: {:?}",
            required_downlevel_capabilities.flags - downlevel_capabilities.flags
        );

        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the surface.
        let needed_limits = A::required_limits().using_resolution(adapter.limits());

        let trace_dir = std::env::var("WGPU_TRACE");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: (optional_features & adapter_features) | required_features,
                    required_limits: needed_limits,
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .expect("Unable to find a suitable GPU adapter!");

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
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
    context: AppContext,
    surface: SurfaceWrapper,
    window: Arc<Window>,
    frame_counter: FrameCounter,
    egui_renderer: EguiRenderer,
    scale_factor: f32,
}

impl<A: App> AppHandler<A> {
    fn new(context: AppContext, surface: SurfaceWrapper, window: Arc<Window>) -> Self {
        let egui_renderer =
            EguiRenderer::new(&context.device, surface.config.format, None, 1, &window);
        Self {
            app: A::init(
                &surface.config,
                &context.adapter,
                &context.device,
                &context.queue,
            ),
            context,
            surface,
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
                        self.surface.resize(&self.context, size);
                        self.app.resize(
                            &self.surface.config,
                            &self.context.device,
                            &self.context.queue,
                        );

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

                        let frame = self.surface.acquire(&self.context);
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                            format: Some(self.surface.config.view_formats[0]),
                            ..wgpu::TextureViewDescriptor::default()
                        });

                        self.app
                            .render(&view, &self.context.device, &self.context.queue);

                        let mut encoder = self.context.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor { label: None },
                        );
                        let screen_descriptor = ScreenDescriptor {
                            size_in_pixels: [self.surface.config.width, self.surface.config.height],
                            pixels_per_point: self.window.scale_factor() as f32 * self.scale_factor,
                        };

                        self.egui_renderer.draw(
                            &self.context.device,
                            &self.context.queue,
                            &mut encoder,
                            &self.window,
                            &view,
                            &screen_descriptor,
                            |ctx| self.app.gui(ctx, &self.context.queue),
                        );
                        self.context.queue.submit(Some(encoder.finish()));
                        frame.present();

                        self.window.request_redraw();
                    }
                    _ => {
                        if !response.consumed {
                            self.app.input(event, &self.context.queue);
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
    let context = AppContext::init_async::<A>().await;
    let surface = SurfaceWrapper::new(&context, window.clone(), A::SRGB);
    let mut handler = AppHandler::<A>::new(context, surface, window);
    tracing::info!("Entering event loop...");
    event_loop.run(move |event, elwt| {
        handler.event(event, elwt);
    })
}
#[allow(clippy::missing_errors_doc)]
pub fn run<A: App>() -> Result<(), EventLoopError> {
    pollster::block_on(start::<A>())
}
