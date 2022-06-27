use crate::vertex::{Float32x2, Float32x3, Float32x4, Vertex};
use std::{mem, sync::Arc};

#[derive(Clone, Debug)]
pub struct ModelVertex {
    pub position: Float32x3,
    pub normal: Float32x3,
    pub tex_coords_0: Float32x2,
    pub tex_coords_1: Float32x2,
    pub tangent: Float32x4,
    pub weights: Float32x4,
    pub joints: [u32; 4],
    pub colors: Float32x4,
}

const POSITION_LOCATION: u32 = 0;
const NORMAL_LOCATION: u32 = 1;
const TEX_COORDS_0_LOCATION: u32 = 2;
const TEX_COORDS_1_LOCATION: u32 = 3;
const TANGENT_LOCATION: u32 = 4;
const WEIGHTS_LOCATION: u32 = 5;
const JOINTS_LOCATION: u32 = 6;
const COLOR_LOCATION: u32 = 7;

const POSITION_OFFSET: usize = 0;
const NORMAL_OFFSET: usize = mem::size_of::<Float32x3>() + POSITION_OFFSET;
const TEX_COORDS_0_OFFSET: usize = mem::size_of::<Float32x2>() + NORMAL_OFFSET;
const TEX_COORDS_1_OFFSET: usize = mem::size_of::<Float32x2>() + TEX_COORDS_0_OFFSET;
const TANGENT_OFFSET: usize = mem::size_of::<Float32x4>() + TEX_COORDS_1_OFFSET;
const WEIGHTS_OFFSET: usize = mem::size_of::<Float32x4>() + TANGENT_OFFSET;
const JOINTS_OFFSET: usize = mem::size_of::<Float32x4>() + WEIGHTS_OFFSET;
const COLOR_OFFSET: usize = mem::size_of::<Float32x4>() + JOINTS_OFFSET;

impl Vertex for ModelVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: POSITION_OFFSET as wgpu::BufferAddress,
                    shader_location: POSITION_LOCATION,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: NORMAL_OFFSET as wgpu::BufferAddress,
                    shader_location: NORMAL_LOCATION,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: TEX_COORDS_0_OFFSET as wgpu::BufferAddress,
                    shader_location: TEX_COORDS_0_LOCATION,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: TEX_COORDS_1_OFFSET as wgpu::BufferAddress,
                    shader_location: TEX_COORDS_1_LOCATION,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: TANGENT_OFFSET as wgpu::BufferAddress,
                    shader_location: TANGENT_LOCATION,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: WEIGHTS_OFFSET as wgpu::BufferAddress,
                    shader_location: WEIGHTS_LOCATION,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: JOINTS_OFFSET as wgpu::BufferAddress,
                    shader_location: JOINTS_LOCATION,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: COLOR_OFFSET as wgpu::BufferAddress,
                    shader_location: COLOR_LOCATION,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct VertexBuffer {
    buffer: Arc<wgpu::Buffer>,
    offset: wgpu::BufferAddress,
    element_count: u32,
}

impl VertexBuffer {
    pub fn new(buffer: Arc<wgpu::Buffer>, offset: wgpu::BufferAddress, element_count: u32) -> Self {
        Self {
            buffer,
            offset,
            element_count,
        }
    }
}

impl VertexBuffer {
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn offset(&self) -> wgpu::BufferAddress {
        self.offset
    }

    pub fn element_count(&self) -> u32 {
        self.element_count
    }
}

pub struct IndexBuffer {
    buffer: Arc<wgpu::Buffer>,
    offset: wgpu::BufferAddress,
    element_count: u32,
    index_type: wgpu::IndexFormat,
}

impl IndexBuffer {
    pub fn new(buffer: Arc<wgpu::Buffer>, offset: wgpu::BufferAddress, element_count: u32) -> Self {
        Self {
            buffer,
            offset,
            element_count,
            index_type: wgpu::IndexFormat::Uint32,
        }
    }
}

impl IndexBuffer {
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn offset(&self) -> wgpu::BufferAddress {
        self.offset
    }

    pub fn element_count(&self) -> u32 {
        self.element_count
    }

    pub fn index_type(&self) -> wgpu::IndexFormat {
        self.index_type
    }
}

pub struct Model {
    matrix: glam::Mat4,
}

impl Model {
    pub fn new() -> Self {
        use glam::{mat4, vec3, Mat4};

        let angle: f32 = -55.0;
        let axis = vec3(1.0, 0.0, 0.0);
        let model =
            Mat4::from_axis_angle(axis, angle.to_radians()) * Mat4::from_cols_array(&[1.0; 16]);

        Model { matrix: model }
    }
}
