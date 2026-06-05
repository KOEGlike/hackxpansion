use crate::metadata::{ModuleID, Slots};
use alloc::boxed::Box;

pub struct RegisteredResource<T: ?Sized> {
    pub slot: Slots,
    pub module_id: ModuleID,
    pub resource: Box<T>,
}

// Implement this to accept a trait object
pub trait Register<T: ?Sized> {
    fn register(&mut self, slot: Slots, module_id: ModuleID, item: Box<T>);
}

// Implement this to hand out a trait object
pub trait Request<T: ?Sized> {
    fn request(&mut self) -> Option<Box<T>>;
}
