use crate::assets::vertex_data::{Vertex, VertexData};
use crate::util::generate_random_alphanumeric_string;
use bincode::config::{Configuration, Fixint, LittleEndian};
use redb::{
    Database, DatabaseError, Key, ReadableDatabase, ReadableTable, ReadableTableMetadata,
    TableDefinition, TypeName, Value,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use serde::de::DeserializeOwned;

pub trait BinData: Serialize+DeserializeOwned{
    fn calc_size(&self) -> usize;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BinaryDataInfo{
    pub is_vertex: bool,
    pub storage_id: u64,
    pub offset: usize,
    pub size: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Storage {
    pub relative_path: String,
    pub current_size: usize,
}

impl Storage {
    pub fn construct_path(&self, base_path: PathBuf) -> PathBuf {
        base_path.join(&self.relative_path)
    }
}

// Use a helper function that forces little-endian encoding
fn bincode_options() -> Configuration<LittleEndian, Fixint> {
    // Always little-endian, fixed-int encoding = stable & minimal
    bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
}

impl Value for BinaryDataInfo{
    type SelfType<'a>
    where
        Self: 'a,
    = BinaryDataInfo;

    type AsBytes<'a>
    where
        Self: 'a,
    = Vec<u8>;


    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bincode::serde::decode_from_slice(data, bincode_options())
            .unwrap()
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        bincode::serde::encode_to_vec(&value, bincode_options()).unwrap()
    }

    fn type_name() -> TypeName {
        TypeName::new("VertexData")
    }
}

impl Value for Storage {
    type SelfType<'a>
    where
        Self: 'a,
    = Storage;

    type AsBytes<'a>
    where
        Self: 'a,
    = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bincode::serde::decode_from_slice(data, bincode_options())
            .unwrap()
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        bincode::serde::encode_to_vec(&value, bincode_options()).unwrap()
    }

    fn type_name() -> TypeName {
        TypeName::new("Storage")
    }
}

static TEXTURE_STORAGE_INDEX: TableDefinition<u64, Storage> = TableDefinition::new("texture_index");
static VERTEX_STORAGE_INDEX: TableDefinition<u64, Storage> = TableDefinition::new("vertex");
static BINARY_META_INDEX: TableDefinition<u64, BinaryDataInfo> = TableDefinition::new("binary_data");
pub static DEFAULT_TEXTURE_STORAGE_SIZE: usize = 1024 * 1024 * 1024;
pub static DEFAULT_VERTEX_STORAGE_SIZE: usize = 1024 * 1024 * 20;
pub struct DataManager {
    db: Database,
    base_path: PathBuf,
    texture_storage_work_size: usize,
    vertex_storage_work_size: usize,
}

impl DataManager {
    pub fn create_or_open(path: &Path, blob_path: Option<PathBuf>) -> Result<Self, DatabaseError> {
        let db = if path.exists() {
            Database::open(path)
        } else {
            Database::create(path)
        };
        if db.is_err(){
            return Err(db.unwrap_err());
        }
        let db = db.unwrap();
        let absolute = path.canonicalize().unwrap();
        let write = db.begin_write().unwrap();
        write.open_table(VERTEX_STORAGE_INDEX).unwrap();
        write.open_table(TEXTURE_STORAGE_INDEX).unwrap();
        write.open_table(BINARY_META_INDEX).unwrap();
        write.commit().unwrap();

        Ok(Self {
            db,
            base_path: if blob_path.is_some() {
                blob_path.unwrap().canonicalize()?
            } else {
                absolute.parent().unwrap().canonicalize()?
            },
            texture_storage_work_size: DEFAULT_TEXTURE_STORAGE_SIZE,
            vertex_storage_work_size: DEFAULT_VERTEX_STORAGE_SIZE,
        })
    }

    pub fn store_binary_data(&self, data: Box<impl BinData>, is_vertex: bool) -> u64{
        let size = data.calc_size();
        let table_def = if is_vertex {VERTEX_STORAGE_INDEX} else {TEXTURE_STORAGE_INDEX};
        
        let mut storage = self.found_or_create_storage(size, table_def, is_vertex);

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(storage.1.construct_path(self.base_path.clone()))
            .unwrap();
        let offset = file.metadata().unwrap().len() as usize;
        bincode::serde::encode_into_std_write(data.deref(), &mut file, bincode_options()).unwrap();
        file.flush().unwrap();
        let size = file.metadata().unwrap().len() as usize - offset ;
        storage.1.current_size = file.metadata().unwrap().len() as usize;

        let storage_id = storage.0;

        self.save_storage(storage, table_def);

        let meta = BinaryDataInfo{
            is_vertex: true,
            storage_id,
            offset,
            size,
        };
        self.save_binary_meta(meta)
    }

    pub fn get_binary_meta(&self, id: u64) -> Option<BinaryDataInfo> {
        let read = self.db.begin_read().unwrap();
        let table = read.open_table(BINARY_META_INDEX).unwrap();
        let res = table.get(id).unwrap();
        if res.is_none(){
            return None;
        }
        let res = res.unwrap().value().clone();
        Some(res)
    }

    fn save_binary_meta(&self, meta: BinaryDataInfo) -> u64{
        let write = self.db.begin_write().unwrap();
        let mut table = write.open_table(BINARY_META_INDEX).unwrap();
        let id = table.len().unwrap();
        table.insert(id, meta).expect("Failed to insert data into table");
        drop(table);
        write.commit().unwrap();
        id
    }

    fn save_storage(&self, storage: (u64, Storage), table: TableDefinition<u64, Storage>) {
        let write = self.db.begin_write().unwrap();
        write
            .open_table(table)
            .unwrap()
            .insert(storage.0, storage.1)
            .unwrap();
        write.commit().unwrap();
    }

    fn found_or_create_storage(
        &self,
        requested_size: usize,
        table: TableDefinition<u64, Storage>,
        is_vertex: bool,
    ) -> (u64, Storage) {
        let read = self.db.begin_read().unwrap();
        let index = read.open_table(table).unwrap();
        for entry in index.iter().unwrap() {
            let Ok((key, value)) = entry else { continue };
            if if is_vertex {
                self.vertex_storage_work_size
            } else {
                self.texture_storage_work_size
            } >= value.value().current_size + requested_size
            {
                return (key.value().clone(), value.value().clone());
            }
        }
        drop(index);
        drop(read);
        self.create_storage(table)
    }

    fn create_storage(&self, table: TableDefinition<u64, Storage>) -> (u64, Storage) {
        let write = self.db.begin_write().unwrap();
        let mut table = write.open_table(table).unwrap();
        let storage = Self::generate_storage_file(self.base_path.clone());
        let len = table.len().unwrap();
        table.insert(len, storage.clone()).unwrap();
        drop(table);
        write.commit().unwrap();
        (len, storage)
    }

    fn generate_storage_file(base_path: PathBuf) -> Storage {
        let name = generate_random_alphanumeric_string(32);
        let mut storage = Storage {
            relative_path: name,
            current_size: 0,
        };
        while fs::exists(storage.construct_path(base_path.clone())).unwrap_or(false) {
            storage.relative_path = generate_random_alphanumeric_string(32);
        }
        storage
    }

    pub fn set_texture_storage_work_size(&mut self, texture_storage_work_size: usize) {
        self.texture_storage_work_size = texture_storage_work_size;
    }

    pub fn set_vertex_storage_work_size(&mut self, vertex_storage_work_size: usize) {
        self.vertex_storage_work_size = vertex_storage_work_size;
    }
}
