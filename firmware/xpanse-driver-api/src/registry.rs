use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::any::{Any, TypeId};

use crate::metadata::{ModuleID, ModuleSlot};

pub struct RegisteredResource<T> {
    pub slot: ModuleSlot,
    pub module_id: ModuleID,
    pub resource: T,
}

pub struct CapabilityList<T> {
    pub items: Vec<RegisteredResource<T>>,
}

impl<T> Default for CapabilityList<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
        }
    }
}

pub struct Registry {
    entries: BTreeMap<TypeId, Box<dyn Any>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn register<T: 'static>(&mut self, slot: ModuleSlot, module_id: ModuleID, resource: T) {
        self.entries
            .entry(TypeId::of::<CapabilityList<T>>())
            .or_insert_with(|| Box::new(CapabilityList::<T>::default()))
            .downcast_mut::<CapabilityList<T>>()
            .expect("TypeId key guarantees the stored type")
            .items
            .push(RegisteredResource {
                slot,
                module_id,
                resource,
            });
    }

    pub fn capabilities<T: 'static>(&mut self) -> Option<&mut CapabilityList<T>> {
        self.entries
            .get_mut(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_mut::<CapabilityList<T>>())
    }

    pub fn has<T: 'static>(&self) -> bool {
        self.entries.contains_key(&TypeId::of::<CapabilityList<T>>())
    }
}
