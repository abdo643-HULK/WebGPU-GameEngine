use gltf::{
    buffer::Data,
    mesh::{BoundingBox, Mode},
};
use image::DynamicImage;
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap, HashSet},
    fs::File,
    io::BufReader,
    ops::Range,
};
use thiserror::Error;
use wgpu::{
    util::DeviceExt, BlendComponent, DepthBiasState, DepthStencilState, Extent3d, Features, LoadOp,
    Operations, PolygonMode, PrimitiveTopology, RenderPassDepthStencilAttachment, RenderPipeline,
    ShaderStages, StencilState, TextureDescriptor, TextureUsages, TextureViewDescriptor,
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

use crate::camera::{Camera, CameraController};

#[derive(Error, Debug)]
pub enum GltfError {
    #[error("unsupported primitive mode: `{0:#?}`")]
    UnsupportedPrimitive(Mode),
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

// pub struct Mesh {
//     pub name: String,
//     pub vertex_buffer: wgpu::Buffer,
//     pub index_buffer: wgpu::Buffer,
//     pub num_elements: u32,
//     pub material: usize,
// }

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct AttributeValuesId(usize);

#[derive(Debug, Clone)]
pub struct Mesh {
    pub topology: wgpu::PrimitiveTopology,
    pub name: Option<String>,
    pub bounding_box: Option<BoundingBox>,
    indecies: Vec<u32>,
    attributes: BTreeMap<AttributeValuesId, AttributeValues>,
}

impl Mesh {
    pub const POSITIONS: AttributeValuesId = AttributeValuesId(0);
    pub const NORMALS: AttributeValuesId = AttributeValuesId(1);
    pub const TEX_COORDS: AttributeValuesId = AttributeValuesId(2);
    pub const COLORS: AttributeValuesId = AttributeValuesId(3);

    pub fn new(
        name: Option<String>,
        topology: wgpu::PrimitiveTopology,
        bb: Option<BoundingBox>,
    ) -> Self {
        Self {
            name,
            topology,
            indecies: vec![],
            attributes: BTreeMap::<AttributeValuesId, AttributeValues>::new(),
            bounding_box: bb,
        }
    }

    #[inline]
    pub fn insert_attribute(
        &mut self,
        attribute: AttributeValuesId,
        values: impl Into<AttributeValues>,
    ) {
        self.attributes.insert(attribute, values.into());
    }

    #[inline]
    pub fn indecies(&self) -> &Vec<u32> {
        &self.indecies
    }

    #[inline]
    pub fn get(&self, attribute: &AttributeValuesId) -> Option<&AttributeValues> {
        self.attributes.get(attribute)
    }

    pub fn insert(
        &mut self,
        attribute: AttributeValuesId,
        values: impl Into<AttributeValues>,
    ) -> &mut Self {
        self.attributes.insert(attribute, values.into());

        return self;
    }

    pub fn set_indecies(&mut self, indecies: Vec<u32>) -> &mut Self {
        self.indecies = indecies;

        return self;
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum AttributeValues {
    Float32x3(Vec<[f32; 3]>),
    Float32x2(Vec<[f32; 2]>),
}

impl AttributeValues {
    pub fn len(&self) -> usize {
        match *self {
            AttributeValues::Float32x2(ref values) => values.len(),
            AttributeValues::Float32x3(ref values) => values.len(),
        }
    }
}

impl From<Vec<[f32; 3]>> for AttributeValues {
    fn from(vec: Vec<[f32; 3]>) -> Self {
        AttributeValues::Float32x3(vec)
    }
}

impl From<Vec<[f32; 2]>> for AttributeValues {
    fn from(vec: Vec<[f32; 2]>) -> Self {
        AttributeValues::Float32x2(vec)
    }
}

#[derive(Debug)]
struct CameraState {
    pub camera: Camera,
    pub camera_buffer: wgpu::Buffer,
    pub camera_uniform: CameraUniform,
    pub camera_controller: CameraController,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl CameraState {
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let camera = Camera {
            eye: glam::vec3(0.0, -90.0, 25.0),
            target: glam::vec3(0.0, 0.0, 0.0),
            up: glam::Vec3::Y,
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            z_near: 0.01,
            z_far: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let camera_controller = CameraController::new(0.4);

        CameraState {
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
            camera_controller,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct HandleId(usize);

#[derive(Debug)]
pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    camera_state: CameraState,
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
        let camera_state = CameraState::new(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            camera_state,
        }
    }

    fn update(&mut self) {
        self.camera_state
            .camera_controller
            .update_camera(&mut self.camera_state.camera);
        self.camera_state
            .camera_uniform
            .update_view_proj(&self.camera_state.camera);
        self.queue.write_buffer(
            &self.camera_state.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_state.camera_uniform]),
        );
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_state.camera_controller.process_events(event)
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
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Matrix4x4([[f32; 4]; 4]);

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view: Matrix4x4,
    projection: Matrix4x4,
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view: Matrix4x4(glam::Mat4::IDENTITY.to_cols_array_2d()),
            projection: Matrix4x4(glam::Mat4::IDENTITY.to_cols_array_2d()),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        let (view, projection) = camera.build_view_projection_matrix();
        self.view = Matrix4x4(view.to_cols_array_2d());
        self.projection = Matrix4x4(projection.to_cols_array_2d());
    }
}

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Float32x4([f32; 4]);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Float32x3([f32; 3]);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Float32x2([f32; 2]);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub normal: Float32x3,
    pub position: Float32x3,
    pub tex_coords: Float32x2,
}

impl Vertex for ModelVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Float32x3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<Float32x3>() + mem::size_of::<Float32x3>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Debug)]
pub struct MeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub indecies_len: u32,
}

#[derive(Debug)]
pub struct GLTFLoader<'a> {
    name: Cow<'a, str>,
    meshes: Vec<Vec<(Mesh, Option<HandleId>)>>,
}

impl<'a> GLTFLoader<'a> {
    pub fn load(label: &'a str, path: &String) -> GLTFLoader<'a> {
        let (document, buffers, images) = gltf::import(path.clone()).unwrap();
        println!("{label}");
        println!("animations: {}", document.animations().count());
        println!("buffers: {}", document.buffers().count());
        println!("camers: {}", document.cameras().count());
        println!("images: {}", document.images().count());
        println!("meshes: {}", document.meshes().count());
        println!("nodes: {}", document.nodes().count());
        println!("samplers: {}", document.samplers().count());
        println!("textures: {}", document.textures().count());
        println!("skins: {}", document.skins().count());
        println!("views: {}", document.views().count());
        println!("materials: {}", document.materials().count());

        let mut named_materials = HashMap::<String, HandleId>::default();
        let mut linear_textures = HashSet::<usize>::default();

        let materials: Vec<HandleId> = document
            .materials()
            .enumerate()
            .map(|(i, material)| {
                let handle = HandleId(i);

                if let Some(name) = material.name() {
                    named_materials.insert(name.to_string(), handle.clone());
                }

                if let Some(texture) = material.normal_texture() {
                    linear_textures.insert(texture.texture().index());
                }

                if let Some(texture) = material.occlusion_texture() {
                    linear_textures.insert(texture.texture().index());
                }

                if let Some(texture) = material
                    .pbr_metallic_roughness()
                    .metallic_roughness_texture()
                {
                    linear_textures.insert(texture.texture().index());
                }

                handle
            })
            .collect();

        let textures = document
            .textures()
            .map(|texture| load_texture(&texture, &buffers, &images))
            .collect::<Vec<DynamicImage>>();

        let meshes = document
            .meshes()
            .map(|ref mesh| {
                mesh.primitives()
                    .map(|ref primitive| {
                        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                        let label = primitive_label(mesh, &primitive);
                        let topology = primitive.mode().primitive_topology().unwrap();
                        let mut mesh =
                            Mesh::new(Some(label), topology, Some(primitive.bounding_box()));

                        reader
                            .read_colors(0)
                            .and_then(|colors| Some(println!("has colors")));

                        let positions = reader
                            .read_positions()
                            .map(|v| AttributeValues::Float32x3(v.collect()))
                            .expect("Positions From Reader");

                        let normals = reader
                            .read_normals()
                            .map(|v| AttributeValues::Float32x3(v.collect()))
                            .expect("Normals From Reader");

                        let tex_coords = reader
                            .read_tex_coords(0)
                            .map(|v| AttributeValues::Float32x2(v.into_f32().collect()))
                            .or_else(|| {
                                let len = positions.len();
                                let uvs = vec![[0.0, 0.0]; len];
                                Some(AttributeValues::from(uvs))
                            })
                            .expect("TexCoords From Reader");

                        let indecies = reader
                            .read_indices()
                            .expect("Indecies From Reader")
                            .into_u32()
                            .collect::<Vec<u32>>();

                        if let AttributeValues::Float32x2(tex_coords) = &tex_coords {
                            println!("{:#?}", tex_coords.len());
                        }

                        mesh.insert(Mesh::POSITIONS, positions)
                            .insert(Mesh::NORMALS, normals)
                            .insert(Mesh::TEX_COORDS, tex_coords)
                            .set_indecies(indecies);

                        let material = primitive
                            .material()
                            .index()
                            .and_then(|i| materials.get(i).cloned());

                        (mesh, material)
                    })
                    .collect()
            })
            .collect::<Vec<Vec<(Mesh, Option<HandleId>)>>>();

        meshes.iter().for_each(|mesh| {
            mesh[0]
                .1
                .as_ref()
                .and_then(|handle| Some(println!("material: {handle:#?}")));
        });

        print!("\n");

        GLTFLoader {
            meshes,
            name: Cow::Borrowed(label),
        }
    }

    pub async fn run<'b>(label: &'static str, path: &'b String) {
        let event_loop = EventLoop::new();
        let monitor = event_loop.primary_monitor().unwrap();
        let window = WindowBuilder::new()
            .with_title("WebGPU GLTF")
            .with_inner_size(monitor.size())
            .build(&event_loop)
            .unwrap();

        let mut state = State::new(&window).await;

        let State { device, .. } = &state;

        let gltf_loader = GLTFLoader::load(label, path);

        let buffers = gltf_loader
            .meshes
            .iter()
            .map(|mesh| {
                mesh.iter()
                    .map(|(primitive, _)| {
                        let default_vec_3 = vec![];
                        let default_vec_2 = vec![];

                        let positions = if let AttributeValues::Float32x3(positions) =
                            primitive.get(&Mesh::POSITIONS).unwrap()
                        {
                            positions
                        } else {
                            &default_vec_3
                        };

                        let normals = if let AttributeValues::Float32x3(normals) =
                            primitive.get(&Mesh::NORMALS).unwrap()
                        {
                            normals
                        } else {
                            &default_vec_3
                        };

                        let tex_coords = if let AttributeValues::Float32x2(tex_coords) =
                            primitive.get(&Mesh::TEX_COORDS).unwrap()
                        {
                            tex_coords
                        } else {
                            &default_vec_2
                        };

                        let vertices = (0..positions.len())
                            .map(|i| ModelVertex {
                                tex_coords: Float32x2(tex_coords[i]),
                                position: Float32x3(positions[i]),
                                normal: Float32x3(normals[i]),
                            })
                            .collect::<Vec<_>>();

                        let vertex_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(&format!("{:?} Vertex Buffer", gltf_loader.name)),
                                contents: bytemuck::cast_slice(&vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            });

                        let index_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(&format!("{:?} Index Buffer", gltf_loader.name)),
                                contents: bytemuck::cast_slice(&primitive.indecies()),
                                usage: wgpu::BufferUsages::INDEX,
                            });

                        println!("{}, {}", vertices.len(), primitive.indecies().len());

                        MeshBuffer {
                            vertex_buffer,
                            index_buffer,
                            indecies_len: primitive.indecies().len() as u32,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        let render_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&state.camera_state.camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

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
                        buffers: &[ModelVertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &frag_shader,
                        entry_point: "main",
                        targets: &[wgpu::ColorTargetState {
                            format: state.config.format,
                            blend: Some(wgpu::BlendState {
                                alpha: BlendComponent {
                                    operation: wgpu::BlendOperation::Add,
                                    src_factor: wgpu::BlendFactor::Zero,
                                    dst_factor: wgpu::BlendFactor::One,
                                },
                                color: BlendComponent::default(),
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        }],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(DepthStencilState {
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        format: wgpu::TextureFormat::Depth24Plus,
                        stencil: StencilState::default(),
                        bias: DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                })
        };

        event_loop.run(move |event, _, control_flow| {
            return match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    if !state.input(event) {
                        match event {
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
                        }
                    }
                }
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    state.update();
                    let _ = gltf_loader.draw(&state, &buffers, &render_pipeline);
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                _ => {}
            };
        });
    }

    fn draw(
        &self,
        state: &State,
        meshes: &Vec<MeshBuffer>,
        render_pipeline: &RenderPipeline,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = state.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = state.device.create_texture(&TextureDescriptor {
            label: Default::default(),
            size: Extent3d {
                width: state.size.width,
                height: state.size.height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: TextureUsages::RENDER_ATTACHMENT,
        });

        let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());
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
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.5,
                            b: 0.5,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: Some(Operations {
                        load: LoadOp::Clear(0),
                        store: true,
                    }),
                }),
            });

            render_pass.set_pipeline(render_pipeline);
            render_pass.set_bind_group(0, &state.camera_state.camera_bind_group, &[]);
            meshes.iter().for_each(|mesh| render_pass.draw_mesh(mesh));
        }

        state.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a MeshBuffer);
    fn draw_mesh_instanced(&mut self, mesh: &'a MeshBuffer, instances: Range<u32>);
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b MeshBuffer) {
        self.draw_mesh_instanced(mesh, 0..1);
    }

    fn draw_mesh_instanced(&mut self, mesh: &'b MeshBuffer, instances: Range<u32>) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.indecies_len as u32, 0, instances);
    }
}

/// Returns the label for the `mesh`.
fn mesh_label(mesh: &gltf::Mesh) -> String {
    format!("Mesh{}", mesh.index())
}

/// Returns the label for the `mesh` and `primitive`.
fn primitive_label(mesh: &gltf::Mesh, primitive: &gltf::Primitive) -> String {
    format!("Mesh{}/Primitive{}", mesh.index(), primitive.index())
}

trait WgpuConverter {
    fn primitive_topology(&self) -> Result<PrimitiveTopology, GltfError>;
}

impl WgpuConverter for Mode {
    fn primitive_topology(&self) -> Result<PrimitiveTopology, GltfError> {
        match self {
            Mode::Points => Ok(PrimitiveTopology::PointList),
            Mode::Lines => Ok(PrimitiveTopology::LineList),
            Mode::LineStrip => Ok(PrimitiveTopology::LineStrip),
            Mode::Triangles => Ok(PrimitiveTopology::TriangleList),
            Mode::TriangleStrip => Ok(PrimitiveTopology::TriangleStrip),
            mode => Err(GltfError::UnsupportedPrimitive(mode.clone())),
        }
    }
}

fn load_texture(
    texture: &gltf::Texture,
    buffers: &Vec<gltf::buffer::Data>,
    images: &Vec<gltf::image::Data>,
) -> image::DynamicImage {
    match texture.source().source() {
        gltf::image::Source::View { view, mime_type } => {
            println!("view: {mime_type}");
            // let image = &images[texture.index()];

            let start = view.offset() as usize;
            let end = (view.offset() + view.length()) as usize;
            let buffer = &buffers[view.buffer().index()][start..end];
            let image = image::load_from_memory(buffer).unwrap();
            image
        }
        gltf::image::Source::Uri { uri, mime_type } => {
            let asset_path = super::ASSETS_PATH;
            let mime_type = mime_type.unwrap();
            println!("uri: {uri}, {}", mime_type);

            let file = File::open(format!("{asset_path}/{uri}")).unwrap();
            let reader = BufReader::new(file);
            image::load(
                reader,
                image::ImageFormat::from_mime_type(mime_type).unwrap(),
            )
            .unwrap()
        }
    }
}

fn create_shaders(device: &wgpu::Device) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    // vertex_shader = glCreateShader(GL_VERTEX_SHADER);
    // glCompileShader(vertex_shader);
    let vertex_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("GLTF Vertex Shader"),
        // glShaderSource
        source: wgpu::ShaderSource::Wgsl(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/gltf.vert.wgsl"
            ))
            .into(),
        ),
    });

    // frag_shader = glCreateShader(GL_FRAGMENT_SHADER);
    // glCompileShader(frag_shader);
    let frag_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("GLTF Fragment Shader"),
        // glShaderSource
        source: wgpu::ShaderSource::Wgsl(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/shaders/gltf.frag.wgsl"
            ))
            .into(),
        ),
    });
    // device.create_shader_module(&include_wgsl!("shader.wgsl"));

    (vertex_shader, frag_shader)
}
