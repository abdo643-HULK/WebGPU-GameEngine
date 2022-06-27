use std::time::{SystemTime, UNIX_EPOCH};
use wgpu::{
    util::DeviceExt, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Features, PolygonMode,
    ShaderStages,
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
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

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FragmentLocals {
    color: [f32; 4],
}

pub async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("WebGPU Triangle")
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&window).await;

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let green_value = since_the_epoch.as_secs_f32().sin() / 2.0 + 0.5;

    let bind_group_layout = &state
        .device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Triangle Fragment Color"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let mut fragment_locals = FragmentLocals {
        color: [0.0, green_value, 0.0, 1.0],
    };

    let fragment_locals_buffer =
        state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Fragment Locals Buffer"),
                contents: bytemuck::cast_slice(&[fragment_locals]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Fragment Locals Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: fragment_locals_buffer.as_entire_binding(),
        }],
    });

    let render_pipeline_layout =
        state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

    // glBindBuffer(GL_ARRAY_BUFFER, vertex_buffer);
    // glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);
    let vertex_buffer = state
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&state.vertices),
            // GL_ARRAY_BUFFER
            usage: wgpu::BufferUsages::VERTEX,
        });

    // glEnableVertexAttribArray(0,...)
    // glVertexAttribPointer(0)
    let desc = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3]
        // equivalent:
        // attributes: &[wgpu::VertexAttribute {
        //     offset: 0,
        //     shader_location: 0,
        //     format: wgpu::VertexFormat::Float32x3,
        // }],
    };

    // render_pipeline = glCreateProgram();
    // glLinkProgram(render_pipeline);
    let render_pipeline = {
        let (vertex_shader, frag_shader) = create_shaders(&state.device);
        state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),

                // glAttachShader(shaderProgram, vertexShader);
                vertex: wgpu::VertexState {
                    module: &vertex_shader,
                    entry_point: "main",
                    buffers: &[desc],
                },
                // glAttachShader(shaderProgram, fragmentShader);
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
                    // glDrawElements(GL_TRIANGLES,..) but here we only specify mode but don't draw.
                    // The draw happens later when we create a RenderPass from the encoder
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // glPolygonMode(GL_FRONT_AND_BACK, GL_LINE)
                    // glPolygonMode(GL_FRONT_AND_BACK, GL_FILL)
                    polygon_mode: PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None, // 1.
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            })

        // glDeleteShader(vertexShader);
        // glDeleteShader(fragmentShader);
        // In Rust the shader get deleted automatically
        // because they get out of scope (trait drop)
    };

    event_loop.run(move |event, _, control_flow| {
        return match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let green_value = start.elapsed().unwrap().as_secs_f32().sin() / 2.0 + 0.5;
                fragment_locals.color = [0.0, green_value, 0.0, 1.0];

                state.queue.write_buffer(
                    &fragment_locals_buffer,
                    0,
                    bytemuck::cast_slice(&[fragment_locals]),
                );

                let _ = draw_triangle(&state, &render_pipeline, &vertex_buffer, &[&bind_group]);
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
    bind_groups: &[&wgpu::BindGroup],
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
            color_attachments: &[
                // This is what [[location(0)]] in the fragment shader targets
                wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.5,
                            b: 0.5,
                            a: 1.0,
                        }),
                        store: true,
                    },
                },
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        bind_groups.iter().enumerate().for_each(|(i, group)| {
            render_pass.set_bind_group(i as u32, group, &[]);
        });
        // glUseProgram(shaderProgram)
        render_pass.set_pipeline(render_pipeline);
        render_pass.draw(0..state.vertices.len() as u32, 0..1);
    }

    // submit will accept anything that implements IntoIter
    state.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
}

fn create_shaders(device: &wgpu::Device) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    // vertex_shader = glCreateShader(GL_VERTEX_SHADER);
    // glCompileShader(vertex_shader);
    let vertex_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Triangle Vertex Shader"),
        // glShaderSource
        source: wgpu::ShaderSource::Wgsl(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/triangle/shader.vert.wgsl"
            ))
            .into(),
        ),
    });

    // frag_shader = glCreateShader(GL_FRAGMENT_SHADER);
    // glCompileShader(frag_shader);
    let frag_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Triangle Fragment Shader"),
        // glShaderSource
        source: wgpu::ShaderSource::Wgsl(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/triangle/shader.frag.wgsl"
            ))
            .into(),
        ),
    });
    // device.create_shader_module(&include_wgsl!("shader.wgsl"));

    (vertex_shader, frag_shader)
}
