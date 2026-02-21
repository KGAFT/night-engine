use std::sync::Arc;
use std::time::Instant;
use glam::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};
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
#[tokio::main]
async fn main() {


    let path = "storage_index.redb";
    let data_manager = DataManager::create_or_open(path.as_ref(), None).unwrap();
    let mut streaming = StreamingSession::new(data_manager, 2048);
    streaming.run().await.unwrap();

    let streaming = Arc::new(streaming);

    let oneshot = streaming.dispatch_task(UserRequest{id: 0}).await.unwrap();
    let oneshot2 = streaming.dispatch_task(UserRequest{id: 1}).await.unwrap();
    let mut data = oneshot.await.unwrap();
    loop {
        data = streaming.dispatch_task(UserRequest{id: 0}).await.unwrap().await.unwrap();
        println!("len {}", data.data.as_ref().unwrap().len());
        println!("{:?}", streaming.dispatch_task(UserRequest{id: 1}).await.unwrap().await.unwrap().data.as_ref().unwrap().len());
    }

}
