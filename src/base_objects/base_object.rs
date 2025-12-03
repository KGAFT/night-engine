use glam::Vec3;

pub trait BaseObject: Send+Sync {
    fn get_position(&self) -> Option<Vec3>;
}