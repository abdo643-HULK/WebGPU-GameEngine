use wgpu::{util::DeviceExt, Features, PolygonMode};
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

type Vertex = [f32; 3];

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub vertices: [Vertex; 3],
}

impl State {
    async fn new(window: &winit::window::Window) -> State {
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

        #[rustfmt::skip]
        let vertices = [
            [0.0, 0.5, 0.0],
            [-0.5, -0.5, 0.0],
            [0.5, -0.5, 0.0],
        ];

        Self {
            surface,
            device,
            queue,
            config,
            size,
            vertices,
        }
    }
}

pub fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("WebGPU Triangle")
        .build(&event_loop)
        .unwrap();

    let state = pollster::block_on(State::new(&window));

    let render_pipeline_layout =
        state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

    let vertex_buffer = state
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&state.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

    let desc = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3],
    };

    let render_pipeline = {
        let (vertex_shader, frag_shader) = create_shaders(&state.device);

        state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_shader,
                    entry_point: "main",
                    buffers: &[desc],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &frag_shader,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: state.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: PolygonMode::Fill,
                    ..Default::default()
                },
                multisample: wgpu::MultisampleState::default(),
                depth_stencil: None,
                multiview: None,
            })

        // glDeleteShader(vertexShader);
        // glDeleteShader(fragmentShader);
        // In Rust the shader get deleted automatically
        // because they get out of scope (trait drop)
    };

    event_loop.run(move |event, _, _| {
        return match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let _ = draw_triangle(&state, &render_pipeline, &vertex_buffer);
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        };
    });
}

pub fn draw_triangle(
    state: &State,
    render_pipeline: &wgpu::RenderPipeline,
    vertex_buffer: &wgpu::Buffer,
) -> Result<(), wgpu::SurfaceError> {
    let output = state.surface.get_current_texture()?;

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    // background color (gray)
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.5,
                        g: 0.5,
                        b: 0.5,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_pipeline(render_pipeline);
        render_pass.draw(0..state.vertices.len() as u32, 0..1);
    }

    state.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
}

fn create_shaders(device: &wgpu::Device) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    let vertex_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Triangle Vertex Shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/triangle/shader.vert.simple.wgsl"
            ))
            .into(),
        ),
    });

    let frag_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Triangle Fragment Shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/triangle/shader.frag.simple.wgsl"
            ))
            .into(),
        ),
    });

    (vertex_shader, frag_shader)
}
