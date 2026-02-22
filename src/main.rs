use std::sync::Arc;
use std::time::Instant;
use glam::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use crate::assets::asset_loader::import_file;
use crate::assets::mesh_data_manager::{BinData, DataManager};
use crate::assets::streaming_session::{StreamingSession, UserRequest};
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

fn generate_data(){
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for i in 0..500000{
        let vertex = Vertex{
            position: Vec3::new(0f32, 23f32, 43f32),
            normal:Vec3::new(0f32, 23f32, 43f32),
            tangent: Vec4::new(0f32, 23f32, 43f32, 0f32),
            uv: Vec2::new(0f32, 23f32),
            color: Vec4::new(0f32, 23f32, 43f32, 0f32),
            bone_indices: [54, 34, 34, 123],
            bone_weights: Vec4::new(0f32, 23f32, 43f32, 0f32),
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
}


#[tokio::main]
async fn main() {
    let scene = import_file("/mnt/hdd/glTF-Sample-Assets/Models/Sponza/glTF/Sponza.gltf".as_ref());
    scene.meshes.iter().for_each(|mesh|{
        println!("{}",mesh.name);
        println!("{}", mesh.data.vertices.len());
    });

    scene.materials.iter().for_each(|material|{
        println!("{}", material.textures.len());
    })
}
