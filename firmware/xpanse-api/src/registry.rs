use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::any::{Any, TypeId};

use crate::metadata::{ModuleID, ModuleSlot};

/// Identifies the physical resource behind a registered logical capability.
///
/// Drivers should use the same `ResourceId` when registering multiple logical
/// capabilities that are backed by the same physical hardware. For example, if
/// one physical button can be used as both `A` and `X`, register both logical
/// button capabilities with the same `ResourceId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
pub struct ResourceId {
    kind: ResourceIdKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
enum ResourceIdKind {
    Physical { slot: ModuleSlot, local_id: u16 },
    Generated(u32),
}

impl ResourceId {
    /// Creates a physical resource id scoped to a module slot.
    ///
    /// The `local_id` only needs to be unique within the module driver for the
    /// given slot. Reuse the same `local_id` for logical capabilities that are
    /// aliases of the same physical resource.
    pub const fn new(slot: ModuleSlot, local_id: u16) -> Self {
        Self {
            kind: ResourceIdKind::Physical { slot, local_id },
        }
    }

    /// Returns the physical `(slot, local_id)` pair for explicitly identified
    /// resources, or `None` for automatically generated ids.
    pub const fn physical_parts(self) -> Option<(ModuleSlot, u16)> {
        match self.kind {
            ResourceIdKind::Physical { slot, local_id } => Some((slot, local_id)),
            ResourceIdKind::Generated(_) => None,
        }
    }

    const fn generated(id: u32) -> Self {
        Self {
            kind: ResourceIdKind::Generated(id),
        }
    }
}

pub struct RegisteredResource<T> {
    metadata: ResourceMetadata,
    pub resource: T,
}

/// Describes where a registered resource originated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
pub enum ResourceOrigin {
    /// A resource provided directly by the main board.
    Platform,
    /// A resource provided by a detected expansion module.
    Module,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
enum ResourceMetadata {
    Platform {
        generated_id: u32,
    },
    Module {
        id: ResourceId,
        slot: ModuleSlot,
        module_id: ModuleID,
    },
}

pub struct CapabilityList<T> {
    items: Vec<RegisteredResource<T>>,
}

impl<T> RegisteredResource<T> {
    pub const fn id(&self) -> ResourceId {
        match self.metadata {
            ResourceMetadata::Platform { generated_id } => ResourceId::generated(generated_id),
            ResourceMetadata::Module { id, .. } => id,
        }
    }

    pub const fn origin(&self) -> ResourceOrigin {
        match self.metadata {
            ResourceMetadata::Platform { .. } => ResourceOrigin::Platform,
            ResourceMetadata::Module { .. } => ResourceOrigin::Module,
        }
    }

    pub const fn slot(&self) -> Option<ModuleSlot> {
        match self.metadata {
            ResourceMetadata::Platform { .. } => None,
            ResourceMetadata::Module { slot, .. } => Some(slot),
        }
    }

    pub const fn module_id(&self) -> Option<ModuleID> {
        match self.metadata {
            ResourceMetadata::Platform { .. } => None,
            ResourceMetadata::Module { module_id, .. } => Some(module_id),
        }
    }
}

impl<T> CapabilityList<T> {
    pub fn iter(&self) -> impl Iterator<Item = &RegisteredResource<T>> {
        self.items.iter()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T> Default for CapabilityList<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

pub struct Registry {
    entries: BTreeMap<TypeId, Box<dyn Any + Send>>,
    next_generated_resource_id: u32,
}

mod private {
    pub trait Sealed {}
}

/// A tuple of different resource types that must use distinct physical ids.
///
/// Implementations are provided for tuples containing two through eight
/// resource types. This trait is sealed and is only intended to be used as the
/// type parameter to [`Registry::has_distinct_set`] and
/// [`Registry::take_distinct_set`].
pub trait DistinctResourceSet: private::Sealed {
    /// The tuple of registered resources returned for this set.
    type Taken;

    #[doc(hidden)]
    fn has_distinct(registry: &Registry) -> bool;

    #[doc(hidden)]
    fn take_distinct(registry: &mut Registry) -> Option<Self::Taken>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum RegistryError {
    SlotMismatch,
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
            next_generated_resource_id: 0,
        }
    }

    /// Registers a resource with an automatically generated unique id.
    ///
    /// Use [`Registry::register_with_id`] instead when multiple logical
    /// capabilities are backed by the same physical resource and need to be
    /// recognized as aliases.
    pub fn register<T: 'static + Send>(
        &mut self,
        slot: ModuleSlot,
        module_id: ModuleID,
        resource: T,
    ) {
        let id = ResourceId::generated(self.next_generated_resource_id());

        self.register_with_id(slot, module_id, id, resource)
            .expect("generated resource IDs do not contain a slot");
    }

    /// Registers a resource provided directly by the main board.
    pub fn register_platform<T: 'static + Send>(&mut self, resource: T) {
        let generated_id = self.next_generated_resource_id();
        self.insert_resource(ResourceMetadata::Platform { generated_id }, resource);
    }

    /// Registers a resource with an explicit physical identity.
    ///
    /// Logical capabilities that map to the same physical hardware should share
    /// the same `id`. Apps can then use distinct allocation methods such as
    /// [`Registry::take_distinct2`] to avoid mapping two required functions to
    /// the same physical resource.
    pub fn register_with_id<T: 'static + Send>(
        &mut self,
        slot: ModuleSlot,
        module_id: ModuleID,
        id: ResourceId,
        resource: T,
    ) -> Result<(), RegistryError> {
        if let Some((id_slot, _)) = id.physical_parts()
            && id_slot != slot
        {
            return Err(RegistryError::SlotMismatch);
        }

        self.insert_resource(
            ResourceMetadata::Module {
                id,
                slot,
                module_id,
            },
            resource,
        );
        Ok(())
    }

    fn next_generated_resource_id(&mut self) -> u32 {
        let id = self.next_generated_resource_id;
        self.next_generated_resource_id = self
            .next_generated_resource_id
            .checked_add(1)
            .expect("generated resource id counter overflowed");
        id
    }

    fn insert_resource<T: 'static + Send>(&mut self, metadata: ResourceMetadata, resource: T) {
        self.entries
            .entry(TypeId::of::<CapabilityList<T>>())
            .or_insert_with(|| Box::new(CapabilityList::<T>::default()))
            .downcast_mut::<CapabilityList<T>>()
            .expect("TypeId key guarantees the stored type")
            .items
            .push(RegisteredResource { metadata, resource });
    }

    pub fn capabilities<T: 'static + Send>(&self) -> Option<&CapabilityList<T>> {
        self.entries
            .get(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_ref::<CapabilityList<T>>())
    }

    /// Returns how many logical resources of type `T` are currently available.
    pub fn resource_count<T: 'static + Send>(&self) -> usize {
        self.entries
            .get(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_ref::<CapabilityList<T>>())
            .map_or(0, |list| list.items.len())
    }

    /// Returns whether at least one logical resource of type `T` is currently available.
    pub fn has<T: 'static + Send>(&self) -> bool {
        self.has_at_least::<T>(1)
    }

    /// Returns whether at least `count` logical resources of type `T` are currently available.
    pub fn has_at_least<T: 'static + Send>(&self, count: usize) -> bool {
        self.resource_count::<T>() >= count
    }

    /// Returns whether at least `count` resources of type `T` with distinct
    /// physical ids are currently available.
    pub fn has_distinct_resources<T: 'static + Send>(&self, count: usize) -> bool {
        if count == 0 {
            return true;
        }

        let Some(list) = self
            .entries
            .get(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_ref::<CapabilityList<T>>())
        else {
            return false;
        };

        count_distinct_ids(list) >= count
    }

    /// Returns whether one `T` and one `U` can be allocated with different
    /// physical ids.
    ///
    /// This is intended for apps that need two different logical capabilities,
    /// such as `Button<A>` and `Button<X>`, and must not receive two aliases of
    /// the same physical control.
    pub fn has_distinct2<T: 'static + Send, U: 'static + Send>(&self) -> bool {
        self.has_distinct_set::<(T, U)>()
    }

    /// Returns whether every resource type in `S` can be allocated with a
    /// different physical id.
    ///
    /// `S` is a tuple containing two through eight different resource types.
    pub fn has_distinct_set<S: DistinctResourceSet>(&self) -> bool {
        S::has_distinct(self)
    }

    pub fn return_resource<T: 'static + Send>(&mut self, resource: RegisteredResource<T>) {
        self.insert_resource(resource.metadata, resource.resource);
    }

    pub fn take_resource<T: 'static + Send>(&mut self) -> Option<RegisteredResource<T>> {
        self.entries
            .get_mut(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_mut::<CapabilityList<T>>())
            .and_then(|list| list.items.pop())
    }

    /// Takes exactly `count` logical resources of type `T` if enough are available.
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

    /// Takes exactly `count` resources of type `T` with distinct physical ids.
    ///
    /// If fewer than `count` distinct physical resources are available, the
    /// registry is left unchanged.
    pub fn take_distinct_resources<T: 'static + Send>(
        &mut self,
        count: usize,
    ) -> Option<Vec<RegisteredResource<T>>> {
        if count == 0 {
            return Some(Vec::new());
        }

        let indices = {
            let list = self
                .entries
                .get(&TypeId::of::<CapabilityList<T>>())
                .and_then(|boxed| boxed.downcast_ref::<CapabilityList<T>>())?;

            distinct_resource_indices(list, count)?
        };

        let list = self
            .entries
            .get_mut(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_mut::<CapabilityList<T>>())
            .expect("resource list existed while selecting distinct resources");

        let mut resources = Vec::new();
        for index in indices {
            resources.push(list.items.remove(index));
        }

        Some(resources)
    }

    /// Atomically takes one `T` and one `U` with different physical ids.
    ///
    /// If no distinct pair is available, the registry is left unchanged. This
    /// method is for different Rust resource types; use
    /// [`Registry::take_distinct_resources`] when taking multiple resources of
    /// the same type.
    pub fn take_distinct2<T: 'static + Send, U: 'static + Send>(
        &mut self,
    ) -> Option<(RegisteredResource<T>, RegisteredResource<U>)> {
        self.take_distinct_set::<(T, U)>()
    }

    /// Atomically takes one resource of every type in `S`, each with a
    /// different physical id.
    ///
    /// `S` is a tuple containing two through eight different resource types.
    /// If no complete distinct assignment exists, the registry is left
    /// unchanged.
    pub fn take_distinct_set<S: DistinctResourceSet>(&mut self) -> Option<S::Taken> {
        S::take_distinct(self)
    }

    fn resource_ids<T: 'static + Send>(&self) -> Option<Vec<ResourceId>> {
        self.entries
            .get(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_ref::<CapabilityList<T>>())
            .map(|list| list.items.iter().map(RegisteredResource::id).collect())
    }

    fn take_resource_at<T: 'static + Send>(&mut self, index: usize) -> RegisteredResource<T> {
        self.entries
            .get_mut(&TypeId::of::<CapabilityList<T>>())
            .and_then(|boxed| boxed.downcast_mut::<CapabilityList<T>>())
            .expect("resource list existed while taking a distinct set")
            .items
            .remove(index)
    }
}

fn count_distinct_ids<T>(list: &CapabilityList<T>) -> usize {
    let mut count = 0;

    for index in 0..list.items.len() {
        let id = list.items[index].id();
        if list.items[..index].iter().all(|item| item.id() != id) {
            count += 1;
        }
    }

    count
}

fn distinct_resource_indices<T>(list: &CapabilityList<T>, count: usize) -> Option<Vec<usize>> {
    let mut indices: Vec<usize> = Vec::new();

    for index in (0..list.items.len()).rev() {
        let id = list.items[index].id();
        if indices
            .iter()
            .all(|&existing_index| list.items[existing_index].id() != id)
        {
            indices.push(index);

            if indices.len() == count {
                return Some(indices);
            }
        }
    }

    None
}

fn distinct_assignment(candidates: &[Vec<ResourceId>]) -> Option<Vec<usize>> {
    fn assign(
        candidates: &[Vec<ResourceId>],
        candidate_index: usize,
        used: &mut Vec<ResourceId>,
        assignment: &mut Vec<usize>,
    ) -> bool {
        if candidate_index == candidates.len() {
            return true;
        }

        for resource_index in (0..candidates[candidate_index].len()).rev() {
            let id = candidates[candidate_index][resource_index];
            if used.contains(&id) {
                continue;
            }

            used.push(id);
            assignment.push(resource_index);
            if assign(candidates, candidate_index + 1, used, assignment) {
                return true;
            }
            assignment.pop();
            used.pop();
        }

        false
    }

    let mut used = Vec::new();
    let mut assignment = Vec::new();
    assign(candidates, 0, &mut used, &mut assignment).then_some(assignment)
}

fn types_are_distinct(types: &[TypeId]) -> bool {
    types
        .iter()
        .enumerate()
        .all(|(index, resource_type)| !types[..index].contains(resource_type))
}

macro_rules! impl_distinct_resource_set {
    ($($resource:ident),+) => {
        impl<$($resource: 'static + Send),+> private::Sealed for ($($resource,)+) {}

        impl<$($resource: 'static + Send),+> DistinctResourceSet for ($($resource,)+) {
            type Taken = ($(RegisteredResource<$resource>,)+);

            fn has_distinct(registry: &Registry) -> bool {
                if !types_are_distinct(&[$(TypeId::of::<$resource>()),+]) {
                    return false;
                }

                let candidates = [$(registry.resource_ids::<$resource>()),+];
                if candidates.iter().any(Option::is_none) {
                    return false;
                }
                let candidates = candidates.map(Option::unwrap);
                distinct_assignment(&candidates).is_some()
            }

            fn take_distinct(registry: &mut Registry) -> Option<Self::Taken> {
                if !types_are_distinct(&[$(TypeId::of::<$resource>()),+]) {
                    return None;
                }

                let candidates = [$(registry.resource_ids::<$resource>()?),+];
                let assignment = distinct_assignment(&candidates)?;
                let mut indices = assignment.into_iter();
                Some(($(
                    registry.take_resource_at::<$resource>(
                        indices.next().expect("distinct assignment contains every resource type"),
                    ),
                )+))
            }
        }
    };
}

impl_distinct_resource_set!(T1, T2);
impl_distinct_resource_set!(T1, T2, T3);
impl_distinct_resource_set!(T1, T2, T3, T4);
impl_distinct_resource_set!(T1, T2, T3, T4, T5);
impl_distinct_resource_set!(T1, T2, T3, T4, T5, T6);
impl_distinct_resource_set!(T1, T2, T3, T4, T5, T6, T7);
impl_distinct_resource_set!(T1, T2, T3, T4, T5, T6, T7, T8);
