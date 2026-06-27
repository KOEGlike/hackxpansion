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
        Self { items: Vec::new() }
    }
}

pub struct Registry {
    entries: BTreeMap<TypeId, Box<dyn Any + Send>>,
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

    pub fn register<T: 'static + Send>(
        &mut self,
        slot: ModuleSlot,
        module_id: ModuleID,
        resource: T,
    ) {
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

    pub fn capabilities<T: 'static + Send>(&mut self) -> Option<&mut CapabilityList<T>> {
        self.entries
            .get_mut(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_mut::<CapabilityList<T>>())
    }

    pub fn has<T: 'static + Send>(&self) -> bool {
        self.entries
            .get(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_ref::<CapabilityList<T>>())
            .is_some_and(|list| !list.items.is_empty())
    }

    pub fn return_resource<T: 'static + Send>(&mut self, resource: RegisteredResource<T>) {
        self.register(resource.slot, resource.module_id, resource.resource);
    }

    pub fn take_resource<T: 'static + Send>(&mut self) -> Option<RegisteredResource<T>> {
        self.entries
            .get_mut(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_mut::<CapabilityList<T>>())
            .and_then(|list| list.items.pop())
    }
}
