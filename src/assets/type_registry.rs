
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

static mut TYPE_REGISTRY: Option<Arc<Mutex<TypeRegistry>>> = None;

pub type DeserFunc = fn(&[u8]) -> Option<Box<dyn Any>>;
pub type SerFunc = fn(&dyn Any) -> Option<Vec<u8>>;




pub trait RegisteredType: Send + Sync {
    fn get_deserialize_function(&self) -> DeserFunc;
    fn get_ser_function(&self) -> SerFunc;
}

struct TypeRegistry {
    map: HashMap<TypeId, Box<dyn RegisteredType>>,
}
#[allow(static_mut_refs)]
impl TypeRegistry {
    pub fn get_instance() -> Arc<Mutex<Self>>{
        unsafe{
            if TYPE_REGISTRY.is_none() {
                TYPE_REGISTRY = Some(Arc::new(Mutex::new(TypeRegistry::new())));
            }
            TYPE_REGISTRY.as_mut().unwrap().clone()
        }
    }
    
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register(&mut self, type_id: TypeId, type_to_register: Box<dyn RegisteredType>) {
        self.map.insert(type_id, type_to_register);
    }

    pub fn deserialize(&self, type_id: TypeId, value: &[u8]) -> Option<Box<dyn Any>> {
        self.map.get(&type_id)?.get_deserialize_function()(value)
    }
    
    pub fn serialize(&self, type_id: TypeId, value: &dyn Any) -> Option<Vec<u8>>{
        self.map.get(&type_id)?.get_ser_function()(value)
    }
}
