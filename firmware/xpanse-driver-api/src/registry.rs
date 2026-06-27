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

    /// Returns how many resources of type `T` are currently available.
    pub fn resource_count<T: 'static + Send>(&self) -> usize {
        self.entries
            .get(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_ref::<CapabilityList<T>>())
            .map_or(0, |list| list.items.len())
    }

    /// Returns whether at least one resource of type `T` is currently available.
    pub fn has<T: 'static + Send>(&self) -> bool {
        self.has_at_least::<T>(1)
    }

    /// Returns whether at least `count` resources of type `T` are currently available.
    pub fn has_at_least<T: 'static + Send>(&self, count: usize) -> bool {
        self.resource_count::<T>() >= count
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

    /// Takes exactly `count` resources of type `T` if enough are available.
    ///
    /// If fewer than `count` resources are available, the registry is left unchanged.
    pub fn take_resources<T: 'static + Send>(
        &mut self,
        count: usize,
    ) -> Option<Vec<RegisteredResource<T>>> {
        if count == 0 {
            return Some(Vec::new());
        }

        let list = self
            .entries
            .get_mut(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_mut::<CapabilityList<T>>())?;

        if list.items.len() < count {
            return None;
        }

        Some(list.items.split_off(list.items.len() - count))
    }
}
