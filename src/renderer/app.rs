use std::{result::Result, sync::Arc, time::Instant};

use wgpu::Surface;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

pub trait App: 'static + Sized {
    const SRGB: bool = true;

    #[must_use]
    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    #[must_use]
    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    #[must_use]
    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::empty(),
            ..wgpu::DownlevelCapabilities::default()
        }
    }

    #[must_use]
    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::downlevel_defaults() // These downlevel limits will allow the code to run on all possible hardware
    }

    fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self;

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );

    fn update(&mut self, event: WindowEvent);

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue);
}

/// Wrapper type which manages the surface and surface configuration.
///
/// As surface usage varies per platform, wrapping this up cleans up the event loop code.
struct SurfaceWrapper {
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
}

impl SurfaceWrapper {
    /// Create a new surface wrapper with no surface or configuration.
    const fn new() -> Self {
        Self {
            surface: None,
            config: None,
        }
    }

    /// Called when an event which matches [`Self::start_condition`] is received.
    ///
    /// On all native platforms, this is where we create the surface.
    ///
    /// Additionally, we configure the surface based on the (now valid) window size.
    fn resume(&mut self, context: &AppContext, window: Arc<Window>, srgb: bool) {
        // Window size is only actually valid after we enter the event loop.
        let window_size = window.inner_size();
        let width = window_size.width.max(1);
        let height = window_size.height.max(1);

        tracing::info!("Surface resume {window_size:?}");

        // We didn't create the surface in pre_adapter, so we need to do so now.
        self.surface = Some(context.instance.create_surface(window).unwrap());

        // From here on, self.surface should be Some.
        let surface = self.surface.as_ref().unwrap();

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
        self.config = Some(config);
    }

    /// Resize the surface, making sure to not resize to zero.
    fn resize(&mut self, context: &AppContext, size: PhysicalSize<u32>) {
        tracing::info!("Surface resize {size:?}");

        let config = self.config.as_mut().unwrap();
        config.width = size.width.max(1);
        config.height = size.height.max(1);
        let surface = self.surface.as_ref().unwrap();
        surface.configure(&context.device, config);
    }

    /// Acquire the next surface texture.
    fn acquire(&mut self, context: &AppContext) -> wgpu::SurfaceTexture {
        let surface = self.surface.as_ref().unwrap();

        match surface.get_current_texture() {
            Ok(frame) => frame,
            // If we timed out, just try again
            Err(wgpu::SurfaceError::Timeout) => surface
                .get_current_texture()
                .expect("Failed to acquire next surface texture!"),
            Err(
                // If the surface is outdated, or was lost, reconfigure it.
                wgpu::SurfaceError::Outdated
                | wgpu::SurfaceError::Lost
                // If OutOfMemory happens, reconfiguring may not help, but we might as well try
                | wgpu::SurfaceError::OutOfMemory,
            ) => {
                surface.configure(&context.device, self.config());
                surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        }
    }

    /// On suspend on android, we drop the surface, as it's no longer valid.
    ///
    /// A suspend event is always followed by at least one resume event.
    fn suspend(&mut self) {
        if cfg!(target_os = "android") {
            self.surface = None;
        }
    }

    const fn get(&self) -> Option<&Surface> {
        self.surface.as_ref()
    }

    fn config(&self) -> &wgpu::SurfaceConfiguration {
        self.config.as_ref().unwrap()
    }
}

/// Context containing global wgpu resources.
struct AppContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}
impl AppContext {
    /// Initializes the example context.
    async fn init_async<A: App>(surface: &SurfaceWrapper) -> Self {
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

        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, surface.get())
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
    app: Option<A>,
    context: AppContext,
    surface: SurfaceWrapper,
    window: Option<Arc<Window>>,
    frame_counter: FrameCounter,
}

impl<A: App> AppHandler<A> {
    fn new(context: AppContext, surface: SurfaceWrapper) -> Self {
        Self {
            app: None,
            context,
            surface,
            window: None,
            frame_counter: FrameCounter::new(),
        }
    }
}
impl<A: App> ApplicationHandler for AppHandler<A> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(WindowAttributes::default())
                    .unwrap(),
            );
            self.window = Some(window.clone());
            self.surface.resume(&self.context, window, A::SRGB);
            if self.app.is_none() {
                self.app = Some(A::init(
                    self.surface.config(),
                    &self.context.adapter,
                    &self.context.device,
                    &self.context.queue,
                ));
            }
        }
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.surface.suspend();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().unwrap().id() != window_id {
            return;
        }
        match event {
            WindowEvent::Resized(size) => {
                self.surface.resize(&self.context, size);
                self.app.as_mut().unwrap().resize(
                    self.surface.config(),
                    &self.context.device,
                    &self.context.queue,
                );

                self.window.as_mut().unwrap().request_redraw();
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
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                // On MacOS, currently redraw requested comes in _before_ Init does.
                // If this happens, just drop the requested redraw on the floor.
                //
                // See https://github.com/rust-windowing/winit/issues/3235 for some discussion
                if self.app.is_none() {
                    return;
                }

                self.frame_counter.update();

                let frame = self.surface.acquire(&self.context);
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                    format: Some(self.surface.config().view_formats[0]),
                    ..wgpu::TextureViewDescriptor::default()
                });

                self.app
                    .as_mut()
                    .unwrap()
                    .render(&view, &self.context.device, &self.context.queue);

                frame.present();

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => self.app.as_mut().unwrap().update(event),
        }
    }
}
#[allow(clippy::future_not_send)]
async fn start<A: App>() -> Result<(), EventLoopError> {
    tracing_subscriber::fmt().init();
    let event_loop = EventLoop::new().unwrap();
    let surface = SurfaceWrapper::new();
    let context = AppContext::init_async::<A>(&surface).await;
    let mut app_handler = AppHandler::<A>::new(context, surface);
    tracing::info!("Entering event loop...");
    event_loop.run_app(&mut app_handler)
}
#[allow(clippy::missing_errors_doc)]
pub fn run<A: App>() -> Result<(), EventLoopError> {
    pollster::block_on(start::<A>())
}
