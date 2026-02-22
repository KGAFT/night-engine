use std::path::Path;
use asset_importer::Importer;
use asset_importer::material::TextureType;
use asset_importer::texture::TextureData as AiTextureData;
use glam::{Vec2, Vec3, Vec4};
use crate::assets::vertex_data::{Mesh, Vertex, VertexData};
use crate::assets::texture_data::{MaterialData, TextureData, TextureSlot};

pub struct ImportedScene {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<MaterialData>,
}

pub fn import_file(path: &Path) -> ImportedScene {
    let importer = Importer::new();
    let scene = importer.import_file(path).unwrap();
    let model_dir = path.parent().unwrap_or(Path::new("."));

    let meshes = build_meshes(&scene);
    let materials = build_materials(&scene, model_dir);

    ImportedScene { meshes, materials }
}


fn build_meshes(scene: &asset_importer::scene::Scene) -> Vec<Mesh> {
    scene.meshes().map(|ai_mesh| {
        let num_verts = ai_mesh.num_vertices();

        // Build per-vertex bone influence: (bone_idx, weight)
        let mut bone_data: Vec<Vec<(u16, f32)>> = vec![Vec::new(); num_verts];
        for (bone_idx, bone) in ai_mesh.bones().enumerate() {
            let idx = bone_idx as u16;
            for vw in bone.weights() {
                let vid = vw.vertex_id as usize;
                if vid < num_verts {
                    bone_data[vid].push((idx, vw.weight));
                }
            }
        }
        for influences in &mut bone_data {
            influences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            influences.truncate(4);
        }

        let positions = ai_mesh.vertices();
        let normals   = ai_mesh.normals();
        let tangents  = ai_mesh.tangents();
        let uvs       = ai_mesh.texture_coords(0); // Vec<Vector3D>, use x/y
        let colors    = ai_mesh.vertex_colors(0);  // Color4D = Vector4D: x=r y=g z=b w=a

        let vertices: Vec<Vertex> = (0..num_verts).map(|i| {
            let p = &positions[i];

            let normal = normals.as_ref()
                .map(|n| { let v = &n[i]; Vec3::new(v.x, v.y, v.z) })
                .unwrap_or(Vec3::ZERO);

            let tangent = tangents.as_ref()
                .map(|t| { let v = &t[i]; Vec4::new(v.x, v.y, v.z, 1.0) })
                .unwrap_or(Vec4::ZERO);

            let uv = uvs.as_ref()
                .map(|u| { let v = &u[i]; Vec2::new(v.x, v.y) })
                .unwrap_or(Vec2::ZERO);

            let color = colors.as_ref()
                .map(|c| { let v = &c[i]; Vec4::new(v.x, v.y, v.z, v.w) })
                .unwrap_or(Vec4::ONE);

            let mut bone_indices = [0u16; 4];
            let mut bone_weights_arr = [0.0f32; 4];
            for (j, &(bi, bw)) in bone_data[i].iter().enumerate() {
                bone_indices[j] = bi;
                bone_weights_arr[j] = bw;
            }

            Vertex {
                position: Vec3::new(p.x, p.y, p.z),
                normal,
                tangent,
                uv,
                color,
                bone_indices,
                bone_weights: Vec4::from_array(bone_weights_arr),
            }
        }).collect();

        let indices: Vec<u32> = ai_mesh.faces()
            .flat_map(|face| {
                let idx = face.indices();
                if idx.len() == 3 { idx.to_vec() } else { Vec::new() }
            })
            .collect();

        Mesh {
            name: ai_mesh.name(),
            data: VertexData { vertices, indices },
            material_index: ai_mesh.material_index(),
        }
    }).collect()
}

const SLOT_MAP: &[(TextureType, TextureSlot)] = &[
    (TextureType::BaseColor,             TextureSlot::Albedo),
    (TextureType::Diffuse,               TextureSlot::Albedo),
    (TextureType::Normals,               TextureSlot::Normal),
    (TextureType::NormalCamera,          TextureSlot::Normal),
    (TextureType::Height,                TextureSlot::Normal),
    (TextureType::GltfMetallicRoughness, TextureSlot::MetallicRoughness),
    (TextureType::Metalness,             TextureSlot::Metalness),
    (TextureType::DiffuseRoughness,      TextureSlot::Roughness),
    (TextureType::EmissionColor,         TextureSlot::Emissive),
    (TextureType::Emissive,              TextureSlot::Emissive),
    (TextureType::AmbientOcclusion,      TextureSlot::AmbientOcclusion),
    (TextureType::Lightmap,              TextureSlot::AmbientOcclusion),
    (TextureType::Opacity,               TextureSlot::Opacity),
];

fn build_materials(
    scene: &asset_importer::scene::Scene,
    model_dir: &Path,
) -> Vec<MaterialData> {
    scene.materials().map(|material| {
        let mut textures: Vec<TextureData> = Vec::new();

        for (tex_type, slot) in SLOT_MAP {
            if textures.iter().any(|t| &t.slot == slot) {
                continue;
            }

            let Some(info) = material.texture(*tex_type, 0) else { continue };

            let loaded = if info.path.starts_with('*') {
                resolve_embedded(scene, &info.path)
            } else {
                load_from_disk(&model_dir.join(&info.path))
            };

            if let Some((width, height, pixels)) = loaded {
                textures.push(TextureData { slot: slot.clone(), width, height, pixels });
            }
        }

        MaterialData { textures }
    }).collect()
}

fn resolve_embedded(
    scene: &asset_importer::scene::Scene,
    hint: &str,
) -> Option<(u32, u32, Vec<u8>)> {
    let tex = scene.embedded_texture_by_name(hint)?;
    match tex.data().ok()? {
        AiTextureData::Compressed(bytes) => load_from_bytes(&bytes),
        AiTextureData::Texels(texels) => {
            let pixels = texels.iter()
                .flat_map(|t| [t.r, t.g, t.b, t.a])
                .collect();
            Some((tex.width(), tex.height(), pixels))
        }
    }
}

fn load_from_disk(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let img = image::open(path).ok()?.to_rgba8();
    Some((img.width(), img.height(), img.into_raw()))
}

fn load_from_bytes(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    Some((img.width(), img.height(), img.into_raw()))
}