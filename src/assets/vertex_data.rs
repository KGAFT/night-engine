use crate::assets::mesh_data_manager::BinData;
use glam::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Vec3,

    pub normal: Vec3,
    pub tangent: Vec4,
    pub uv: Vec2,
    pub color: Vec4,

    pub bone_indices: [u16; 4],
    pub bone_weights: Vec4
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl BinData for VertexData {
    fn calc_size(&self) -> usize {
        size_of::<usize>() * 2
            + self.vertices.len() * size_of::<Vertex>()
            + self.indices.len() * size_of::<u32>()
    }
}
