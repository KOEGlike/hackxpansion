use alloc::boxed::Box;
use crate::metadata::{Module, Slots};

pub struct RegisteredResource<T: ?Sized> {
    pub slot: Slots,
    pub module: Module,
    pub resource: Box<T>,
}

// Implement this to accept a trait object
pub trait Register<T: ?Sized> {
    fn register(&mut self, slot: Slots, module: Module, item: Box<T>);
}

// Implement this to hand out a trait object
pub trait Request<T: ?Sized> {
    fn request(&mut self) -> Option<Box<T>>;
}
