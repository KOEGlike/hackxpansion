use crate::metadata::{ModuleID, ModuleSlot};

pub trait RegisteredResourceInner {
    type Info;
}

pub struct RegisteredResource<T: RegisteredResourceInner> {
    pub slot: ModuleSlot,
    pub module_id: ModuleID,
    pub resource: T,
    pub info: T::Info,
}

// Implement this to accept a trait object
pub trait Register<T: RegisteredResourceInner> {
    fn register(&mut self, slot: ModuleSlot, module_id: ModuleID, item: T, info: T::Info);
}

// Implement this to hand out a trait object
pub trait Request<T: RegisteredResourceInner> {
    fn request(&mut self) -> Option<T>;
}
