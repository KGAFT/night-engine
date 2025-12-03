use std::time::Instant;
use glam::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use crate::assets::mesh_data_manager::{BinData, DataManager};
use crate::assets::vertex_data::{Vertex, VertexData};

mod assets;
mod base_objects;
mod util;

#[derive(Serialize, Deserialize)]
pub struct TestBinData{
    binary: Vec<u8>,
}

impl BinData for TestBinData{
    fn calc_size(&self) -> usize {
        size_of::<u64>()+self.binary.len()
    }
}

fn main() {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for i in 0..500000{
        let vertex = Vertex{
            position: Some(Vec3::new(0f32, 23f32, 43f32)),
            normal: Some(Vec3::new(0f32, 23f32, 43f32)),
            tangent: Some(Vec4::new(0f32, 23f32, 43f32, 0f32)),
            uv: Some(Vec2::new(0f32, 23f32)),
            color: Some(Vec4::new(0f32, 23f32, 43f32, 0f32)),
            bone_indices: Some([54, 34, 34, 123]),
            bone_weights: Some(Vec4::new(0f32, 23f32, 43f32, 0f32)),
            extra: None,
        };
        vertices.push(vertex);
        indices.push((i) as u32);
    }
    let path = "storage_index.redb";
    let data_manager = DataManager::create_or_open(path.as_ref(), None).unwrap();
    let vertex_data = VertexData{ vertices, indices };

    let test_bin_data = TestBinData{ binary: vec![0u8; 4096]};

    let cur_time = Instant::now();
    let id = data_manager.store_binary_data(Box::new(vertex_data), true);
    let id2 = data_manager.store_binary_data(Box::new(test_bin_data), false);


    let end = cur_time.elapsed();

    println!("{:?}", end);

    let meta = data_manager.get_binary_meta(id).unwrap();
    let meta2 = data_manager.get_binary_meta(id2).unwrap();

    println!("Binary Meta: {:?}", meta);
    println!("Binary Meta2: {:?}", meta2);
}
