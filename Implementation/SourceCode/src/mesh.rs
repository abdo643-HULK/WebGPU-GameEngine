use glam::Vec3A;
use wgpu::Device;

use crate::loader::AttributeValues;

// Axis-Aligned Bounding Box
#[derive(Clone, Debug)]
pub struct AABB {
    pub min: Vec3A,
    pub max: Vec3A,
}

impl AABB {
    pub fn new(min: Vec3A, max: Vec3A) -> Self {
        AABB { min, max }
    }
}

impl AABB {
    pub fn radius(&self) -> f32 {
        let diagonal = self.max - self.min;
        diagonal.length()
    }

    pub fn center(&self) -> Vec3A {
        (self.max + self.min) * 0.5
    }

    pub fn bounds(&self) -> glam::Vec3A {
        self.max - self.min
    }
}

impl From<gltf::mesh::BoundingBox> for AABB {
    fn from(bb: gltf::mesh::BoundingBox) -> Self {
        let min: Vec3A = bb.min.into();
        let max: Vec3A = bb.max.into();

        AABB { min, max }
    }
}

#[derive(Clone, Debug)]
pub struct Primitive {
    index: usize,
    indices: Option<Vec<AttributeValues>>,
    vertices: Vec<AttributeValues>,
    aabb: AABB,
}

impl Primitive {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn vertices(&self) -> &Vec<AttributeValues> {
        &self.vertices
    }

    pub fn indices(&self) -> &Option<Vec<AttributeValues>> {
        &self.indices
    }

    pub fn aabb(&self) -> &AABB {
        &self.aabb
    }
}

pub struct Mesh {
    primitives: Vec<Primitive>,
}

impl Mesh {
    fn new(primitives: Vec<Primitive>) -> Self {
        Mesh { primitives }
    }
}

impl Mesh {
    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }

    pub fn primitive_count(&self) -> usize {
        self.primitives.len()
    }
}

pub struct Meshes {
    pub meshes: Vec<Mesh>,
    pub vertices: wgpu::Buffer,
    pub indices: Option<wgpu::Buffer>,
}

impl Meshes {
    fn create_meshes(device: &Device) {}
}
