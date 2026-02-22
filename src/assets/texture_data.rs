use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureSlot {
    Albedo,
    Normal,
    MetallicRoughness,
    Metalness,
    Roughness,
    Emissive,
    AmbientOcclusion,
    Opacity,
    Unknown,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureData {
    pub slot: TextureSlot,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialData {
    pub textures: Vec<TextureData>,
}