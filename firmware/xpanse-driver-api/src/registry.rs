use crate::metadata::{ModuleID, ModuleSlot};

pub struct RegisteredResource<T> {
    pub slot: ModuleSlot,
    pub module_id: ModuleID,
    pub resource: T,
}

// Implement this to accept a trait object
pub trait Register<T> {
    fn register(&mut self, slot: ModuleSlot, module_id: ModuleID, item: T);
}

// Implement this to hand out a trait object
pub trait Request<T> {
    fn request(&mut self) -> Option<T>;
}
