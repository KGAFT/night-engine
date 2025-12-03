use crate::assets::mesh_data_manager::BinData;
use glam::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Option<Vec3>,

    pub normal: Option<Vec3>,
    pub tangent: Option<Vec4>,
    pub uv: Option<Vec2>,
    pub color: Option<Vec4>,

    pub bone_indices: Option<[u16; 4]>,
    pub bone_weights: Option<Vec4>,

    pub extra: Option<HashMap<String, Vec<f32>>>,
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
