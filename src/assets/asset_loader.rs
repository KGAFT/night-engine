use std::path::Path;
use asset_importer::Importer;
use glam::Vec3;

pub fn import_file(path: & Path){
    let importer = Importer::new();
    let scene = importer.import_file(path).unwrap();
    let mut vertices: Vec<Vec3> = Vec::new();
    scene.meshes().for_each(|mesh| {
        for i in 0..mesh.num_vertices(){
            vertices.push(mesh.vertices()[i]);
            vertices.push(mesh.normals().unwrap()[i]);
            vertices.push(mesh.texture_coords(0).unwrap()[i]);
        }
    })
}