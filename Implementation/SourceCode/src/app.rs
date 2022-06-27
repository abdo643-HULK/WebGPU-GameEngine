use super::ASSETS_PATH;
use crate::assets::AssetServer;

use std::{
    borrow::{BorrowMut, Cow},
    mem,
    path::Path,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::{Icon, Window, WindowBuilder},
};

pub trait System: std::fmt::Debug {
    fn init(&mut self, context: &Context);
    fn process_events<'a>(&mut self, event: &'a Event<()>, context: &Context);
}

#[derive(Debug)]
pub struct Resource<T: 'static> {
    pub window: Window,
    pub event_loop: EventLoop<T>,
}

impl<T: 'static> Resource<T> {
    fn new(window: Window, event_loop: EventLoop<T>) -> Self {
        Self { window, event_loop }
    }
}

pub struct World {}

pub type RenderDevice = Arc<wgpu::Device>;
pub type RenderQueue = Arc<wgpu::Queue>;
pub type RenderInstance = wgpu::Instance;

pub type SharedContext = Arc<Context>;

pub struct Context {
    window: Arc<Window>,
    pub queue: RenderQueue,
    pub device: RenderDevice,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub config: wgpu::SurfaceConfiguration,
}

impl Context {
    pub fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppOptions {
    pub title: Option<&'static str>,
    pub assets_root: Option<&'static str>,
    pub icon: Option<&'static str>,
}

#[derive(Debug)]
pub struct App {
    resources: Resource<()>,
    systems: Vec<Box<dyn System>>,
    asset_server: AssetServer,
}

impl App {
    pub fn new() -> Self {
        App::default()
    }

    pub fn init(options: AppOptions) -> Self {
        // winit::event_loop::EventLoop::with_user_event();
        let event_loop = EventLoop::new();
        let primary_monitor = event_loop.primary_monitor().unwrap();

        let asset_server = match options.assets_root {
            Some(root) => AssetServer::new(root),
            None => AssetServer::default(),
        };

        let icon = load_icon(&Path::new(ASSETS_PATH).join("icon.png"));

        let mut builder = WindowBuilder::new()
            .with_inner_size(primary_monitor.size())
            .with_window_icon(Some(icon));

        builder = match options.title {
            Some(title) => builder.with_title(Cow::Borrowed(title)),
            None => builder,
        };

        let window = builder.build(&event_loop).unwrap();

        Self {
            resources: Resource::new(window, event_loop),
            asset_server,
            systems: vec![],
        }
    }

    pub fn add_system(mut self, system: Box<dyn System>) -> Self {
        self.systems.push(system);
        self
    }

    fn empty() -> App {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(event_loop.primary_monitor().unwrap().size())
            .build(&event_loop)
            .unwrap();

        Self {
            resources: Resource::new(window, event_loop),
            asset_server: AssetServer::default(),
            systems: vec![],
        }
    }

    pub fn run(self) {
        pollster::block_on(self.run_async());
    }

    async fn run_async(mut self) {
        log::info!("Initializing the surface...");

        let App { resources, .. } = self;

        let Resource { window, .. } = resources;

        let WgpuResources {
            surface,
            adapter,
            device,
            queue,
            instance,
            config,
        } = init_wgpu_resources(&window).await;

        let device = Arc::new(device);
        let queue = Arc::new(queue);
        let window = Arc::new(window);

        let context = Arc::new(Mutex::new(Context {
            window,
            surface,
            adapter,
            device,
            queue,
            config,
        }));

        context
            .lock()
            .and_then(|val| {
                self.systems
                    .iter_mut()
                    .for_each(|system| system.init(&*val));

                Ok(())
            })
            .unwrap();

        resources.event_loop.run(move |event, _, control_flow| {
            use winit::event::Event::*;
            use winit::event::WindowEvent::*;

            *control_flow = ControlFlow::Wait;

            {
                context
                    .lock()
                    .and_then(|val| {
                        self.systems
                            .iter_mut()
                            .for_each(|system| system.process_events(&event, &*val));

                        Ok(())
                    })
                    .unwrap();
            }

            match event {
                WindowEvent {
                    ref event,
                    window_id,
                } if window_id == context.lock().unwrap().window().id() => match event {
                    CloseRequested => *control_flow = ControlFlow::Exit,
                    Resized(size) => {
                        context
                            .lock()
                            .as_mut()
                            .and_then(|val| Ok(val.resize(size)))
                            .unwrap();
                    }
                    ScaleFactorChanged { new_inner_size, .. } => {
                        context
                            .lock()
                            .as_mut()
                            .and_then(|val| Ok(val.resize(new_inner_size)))
                            .unwrap();
                    }
                    _ => {}
                },
                _ => (),
            }
        });
    }
}

impl Default for App {
    fn default() -> Self {
        App::empty()
    }
}

struct WgpuResources {
    instance: RenderInstance,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

fn check_gpu_features(adapter: &wgpu::Adapter) {
    let required_features = wgpu::Features::POLYGON_MODE_LINE | wgpu::Features::DEPTH_CLIP_CONTROL;
    let supported_features = adapter.features();

    assert!(
        supported_features.contains(required_features),
        "Adapter Missing required: {:#?}",
        (required_features - supported_features)
    );
    println!("{:#?}", adapter.get_info());
}

async fn init_wgpu_resources(window: &Window) -> WgpuResources {
    use wgpu::Features;

    let size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: Features::POLYGON_MODE_LINE | Features::DEPTH_CLIP_CONTROL,
                limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        )
        .await
        .unwrap();

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_preferred_format(&adapter).unwrap(),
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };

    surface.configure(&device, &config);

    check_gpu_features(&adapter);

    return WgpuResources {
        surface,
        adapter,
        device,
        queue,
        config,
        instance,
    };
}

fn load_icon(path: &Path) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
