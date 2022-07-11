use async_trait::async_trait;
use gltf::mesh::Mode;
use std::{
    borrow::{Borrow, Cow},
    collections::BTreeMap,
    f32::{consts::PI, INFINITY},
    ops::{Deref, Range},
    sync::{Arc, Mutex},
};
use thiserror::Error;
use wgpu::{
    util::DeviceExt, BlendComponent, DepthBiasState, DepthStencilState, Extent3d, Features, LoadOp,
    Operations, PolygonMode, PrimitiveTopology, RenderPassDepthStencilAttachment, RenderPipeline,
    StencilState, TextureDescriptor, TextureUsages, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, event::*};

use crate::{
    app::{Context, Resource, SharedContext, System},
    camera::first::{CameraController, MyCamera},
    mesh::AABB,
    vertex::{Float32x2, Float32x3, ModelVertex, Vertex},
};

#[derive(Error, Debug)]
pub enum GltfError {
    #[error("unsupported primitive mode: `{0:#?}`")]
    UnsupportedPrimitive(Mode),
}

pub struct Model {
    pub meshes: Vec<Primitive>,
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

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct AttributeValuesId(usize);

#[derive(Debug, Clone)]
pub struct Primitive {
    pub topology: wgpu::PrimitiveTopology,
    pub name: Option<String>,
    pub aabb: Option<AABB>,
    indecies: Vec<u32>,
    attributes: BTreeMap<AttributeValuesId, AttributeValues>,
}

impl Primitive {
    pub const POSITIONS: AttributeValuesId = AttributeValuesId(0);
    pub const NORMALS: AttributeValuesId = AttributeValuesId(1);
    pub const TEX_COORDS: AttributeValuesId = AttributeValuesId(2);
    pub const COLORS: AttributeValuesId = AttributeValuesId(3);

    pub fn new(
        name: Option<String>,
        topology: wgpu::PrimitiveTopology,
        aabb: Option<AABB>,
    ) -> Self {
        Self {
            name,
            topology,
            indecies: vec![],
            attributes: BTreeMap::<AttributeValuesId, AttributeValues>::new(),
            aabb,
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
    // pub camera: Camera,
    pub camera_buffer: wgpu::Buffer,
    pub camera_uniform: CameraUniform,
    pub camera_controller: CameraController,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl CameraState {
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let camera = MyCamera::new(config);
        let mut camera_controller = CameraController::new(camera, 0.4);
        let view = camera_controller.update_camera();

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera_controller.camera, &view);

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

        CameraState {
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
    pub size: winit::dpi::PhysicalSize<u32>,
    camera_state: CameraState,
    render_pipeline: Option<RenderPipeline>,
    meshes: Option<Vec<MeshBuffer>>,
}

impl State {
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> State {
        let camera_state = CameraState::new(&device, &config);

        Self {
            size: PhysicalSize::new(config.width, config.height),
            camera_state,
            render_pipeline: None,
            meshes: None,
        }
    }

    fn update(&mut self, context: &Context, model: &glam::Mat4) {
        self.camera_state.camera_controller.update_camera();

        self.camera_state
            .camera_uniform
            .update_view_proj(&self.camera_state.camera_controller.camera, &model);

        context.queue.write_buffer(
            &self.camera_state.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_state.camera_uniform]),
        );
    }

    pub fn animate(&mut self) {
        let controller = &mut self.camera_state.camera_controller;
        controller.animate();
    }

    fn input(&mut self, event: &WindowEvent, context: &Context) -> bool {
        self.camera_state
            .camera_controller
            .process_events(event, context)
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
    view_proj: Matrix4x4,
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view: Matrix4x4(glam::Mat4::IDENTITY.to_cols_array_2d()),
            projection: Matrix4x4(glam::Mat4::IDENTITY.to_cols_array_2d()),
            view_proj: Matrix4x4(glam::Mat4::IDENTITY.to_cols_array_2d()),
        }
    }

    fn update_view_proj(&mut self, camera: &MyCamera, model: &glam::Mat4) {
        let (projection, view) = camera.build_view_projection_matrix();
        self.view = Matrix4x4(view.to_cols_array_2d());
        self.projection = Matrix4x4(projection.to_cols_array_2d());
        // self.view_proj = Matrix4x4((projection * (*view)).to_cols_array_2d());
        self.view_proj = Matrix4x4((projection * view * *model).to_cols_array_2d());
    }
}

#[derive(Debug)]
pub struct MeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub indecies_len: u32,
}

#[derive(Debug, PartialEq)]
pub enum AnimationLoop {
    Enable,
    Disable,
}

impl Default for AnimationLoop {
    fn default() -> Self {
        AnimationLoop::Disable
    }
}

#[derive(Debug, Default)]
pub struct GLTFLoader {
    path: String,
    name: Option<String>,
    meshes: Vec<Vec<Primitive>>,
    state: Option<State>,
    is_animation_anabled: AnimationLoop,
}

impl GLTFLoader {
    pub fn new(path: String, enable_animation: AnimationLoop) -> GLTFLoader {
        GLTFLoader {
            state: None,
            path,
            name: None,
            meshes: vec![],
            is_animation_anabled: enable_animation,
        }
    }

    pub fn load(&mut self) {
        let (document, buffers, _) = gltf::import(&self.path).unwrap();
        let label = document.meshes().collect::<Vec<_>>()[0]
            .name()
            .unwrap()
            .to_owned();

        let meshes = document
            .meshes()
            .map(|ref mesh| {
                mesh.primitives()
                    .map(|ref primitive| {
                        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                        let label = primitive_label(mesh, &primitive);
                        let topology = primitive.mode().primitive_topology().unwrap();
                        let mut mesh = Primitive::new(
                            Some(label),
                            topology,
                            Some(primitive.bounding_box().into()),
                        );

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

                        println!("Vertices: {}", positions.len() + indecies.len());

                        mesh.insert(Primitive::POSITIONS, positions)
                            .insert(Primitive::NORMALS, normals)
                            .insert(Primitive::TEX_COORDS, tex_coords)
                            .set_indecies(indecies);

                        mesh
                    })
                    .collect()
            })
            .collect::<Vec<Vec<Primitive>>>();

        // print!("\n");

        self.meshes = meshes;
        self.name = Some(label);
    }

    fn draw(&self, state: &State, context: &Context) -> Result<(), wgpu::SurfaceError> {
        let output = context.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = context.device.create_texture(&TextureDescriptor {
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
        let mut encoder = context
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
                            r: 0.3,
                            g: 0.5,
                            b: 0.7,
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
                        load: LoadOp::Load,
                        store: true,
                    }),
                }),
            });

            render_pass.set_pipeline(state.render_pipeline.as_ref().unwrap());
            render_pass.set_bind_group(0, &state.camera_state.camera_bind_group, &[]);
            state
                .meshes
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|mesh| render_pass.draw_mesh(mesh));
        }

        context.queue.submit(std::iter::once(encoder.finish()));
        // context.queue.on_submitted_work_done() // Fence Equivalent
        output.present();
        Ok(())
    }
}

impl GLTFLoader {
    fn create_mesh_buffers(&self, device: &wgpu::Device) -> Vec<MeshBuffer> {
        self.meshes
            .iter()
            .map(|mesh| {
                mesh.iter()
                    .map(|primitive| {
                        let default_vec_3 = vec![];
                        let default_vec_2 = vec![];

                        let positions = if let AttributeValues::Float32x3(positions) =
                            primitive.get(&Primitive::POSITIONS).unwrap()
                        {
                            positions
                        } else {
                            &default_vec_3
                        };

                        let normals = if let AttributeValues::Float32x3(normals) =
                            primitive.get(&Primitive::NORMALS).unwrap()
                        {
                            normals
                        } else {
                            &default_vec_3
                        };

                        let tex_coords = if let AttributeValues::Float32x2(tex_coords) =
                            primitive.get(&Primitive::TEX_COORDS).unwrap()
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
                                label: Some(&format!("{:?} Vertex Buffer", self.name)),
                                contents: bytemuck::cast_slice(&vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            });

                        let index_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some(&format!("{:?} Index Buffer", self.name)),
                                contents: bytemuck::cast_slice(&primitive.indecies()),
                                usage: wgpu::BufferUsages::INDEX,
                            });

                        MeshBuffer {
                            vertex_buffer,
                            index_buffer,
                            indecies_len: primitive.indecies().len() as u32,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>()
    }

    pub fn create_render_pipeline(
        &self,
        context: &Context,
        layout: &wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline {
        let (vertex_shader, frag_shader) = create_shaders(&context.device);

        context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: &vertex_shader,
                    entry_point: "main",
                    buffers: &[ModelVertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &frag_shader,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: context.config.format,
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
                    polygon_mode: PolygonMode::Line,
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
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            })
    }
}

impl System for GLTFLoader {
    fn init(&mut self, context: &Context) {
        let Context { device, .. } = context;
        self.load();

        let buffers = self.create_mesh_buffers(&*device);
        let primitive = &self.meshes[0][0];
        let aabb = primitive.aabb.as_ref().unwrap();

        let mut state = State::new(&context.device, &context.config);
        state.update(context, &get_transform(aabb));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&state.camera_state.camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        state.render_pipeline = Some(self.create_render_pipeline(context, &render_pipeline_layout));
        state.meshes = Some(buffers);
        self.state = Some(state);
    }

    fn process_events<'a>(&mut self, event: &'a Event<()>, context: &Context) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if *window_id == context.window().id() => {
                if self.state.as_mut().unwrap().input(event, context) {
                    let primitive = &self.meshes[0][0];
                    let aabb = primitive.aabb.as_ref().unwrap();
                    self.state
                        .as_mut()
                        .unwrap()
                        .update(context, &get_transform(aabb));
                };

                if let WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } = event
                {
                    match keycode {
                        VirtualKeyCode::Space => {
                            self.is_animation_anabled = AnimationLoop::Enable;
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if *window_id == context.window().id() => {
                if let AnimationLoop::Enable = self.is_animation_anabled {
                    self.state.as_mut().unwrap().animate();
                    let primitive = &self.meshes[0][0];
                    let aabb = primitive.aabb.as_ref().unwrap();
                    self.state
                        .as_mut()
                        .unwrap()
                        .update(context, &get_transform(aabb));
                }
                let _ = self.draw(self.state.as_ref().unwrap(), context);
            }
            Event::MainEventsCleared => {
                context.window().request_redraw();
            }
            _ => {}
        };
    }
}

fn get_transform(aabb: &AABB) -> glam::Mat4 {
    let larger_side = (aabb.max - aabb.min).max_element();

    let scale_factor = 1.0_f32 / larger_side;
    let aabb = AABB::new(aabb.min * scale_factor, aabb.max * scale_factor);
    let center = aabb.center();

    let model = glam::Mat4::IDENTITY;
    // let translation = glam::Mat4::from_translation((-center).into());

    let rotation_x = glam::Mat4::from_axis_angle(glam::Vec3::X, 90.0_f32.to_radians());
    // let rotation_x = glam::Mat4::from_axis_angle(glam::Vec3::X, 0.0_f32.to_radians());

    let rotation_y = glam::Mat4::from_axis_angle(glam::Vec3::Y, 0.0_f32.to_radians());

    // let rotation_z = glam::Mat4::from_axis_angle(glam::Vec3::Z, 0.0_f32.to_radians());
    // let rotation_z = glam::Mat4::from_axis_angle(glam::Vec3::Z, 180.0_f32.to_radians()); // Armadillo
    let rotation_z = glam::Mat4::from_axis_angle(glam::Vec3::Z, 270.0_f32.to_radians()); // Dragon

    let rotation = rotation_x * rotation_y * rotation_z;
    let rotation = glam::Quat::from_mat4(&rotation);

    // let scale = glam::Mat4::from_scale(glam::Vec3::from_array([scale_factor; 3]));
    let scale = glam::Vec3::from_array([scale_factor; 3]);

    // let translation = glam::vec3(0., 0., 0.);
    // let translation = glam::vec3(0., -0.2, 0.); // Armadillo
    let translation = glam::vec3(0., -0.5, 0.); // Dragon

    let tranform = glam::Mat4::from_scale_rotation_translation(scale, rotation, translation);

    tranform * model
    // translation * rotation * scale * model
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
