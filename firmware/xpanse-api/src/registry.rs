use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::{vec, vec::Vec};
use core::any::{Any, TypeId};
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::metadata::{ModuleID, ModuleSlot};

type CapabilityMap = BTreeMap<TypeId, Box<dyn Any + Send>>;

static NEXT_REGISTRY_ID: AtomicU32 = AtomicU32::new(0);

/// Identifies one physical resource group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
pub struct ResourceId {
    kind: ResourceIdKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
enum ResourceIdKind {
    ModuleLocal { slot: ModuleSlot, local_id: u16 },
    RegistryAllocated(u32),
}

impl ResourceId {
    /// Creates an id scoped to a module slot.
    pub const fn module_local(slot: ModuleSlot, local_id: u16) -> Self {
        Self {
            kind: ResourceIdKind::ModuleLocal { slot, local_id },
        }
    }

    /// Returns the module-local `(slot, local_id)` pair, or `None` for a
    /// registry-allocated id.
    pub const fn module_local_parts(self) -> Option<(ModuleSlot, u16)> {
        match self.kind {
            ResourceIdKind::ModuleLocal { slot, local_id } => Some((slot, local_id)),
            ResourceIdKind::RegistryAllocated(_) => None,
        }
    }

    const fn registry_allocated(id: u32) -> Self {
        Self {
            kind: ResourceIdKind::RegistryAllocated(id),
        }
    }
}

/// Describes where a resource group originated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
pub enum ResourceOrigin {
    Platform,
    Module,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
enum ResourceMetadata {
    PlatformAllocated {
        allocated_id: u32,
    },
    ModuleAllocated {
        allocated_id: u32,
        slot: ModuleSlot,
        module_id: ModuleID,
    },
    ModuleLocal {
        local_id: u16,
        slot: ModuleSlot,
        module_id: ModuleID,
    },
}

impl ResourceMetadata {
    const fn id(self) -> ResourceId {
        match self {
            Self::PlatformAllocated { allocated_id }
            | Self::ModuleAllocated { allocated_id, .. } => {
                ResourceId::registry_allocated(allocated_id)
            }
            Self::ModuleLocal { slot, local_id, .. } => ResourceId::module_local(slot, local_id),
        }
    }

    const fn origin(self) -> ResourceOrigin {
        match self {
            Self::PlatformAllocated { .. } => ResourceOrigin::Platform,
            Self::ModuleAllocated { .. } | Self::ModuleLocal { .. } => ResourceOrigin::Module,
        }
    }

    const fn slot(self) -> Option<ModuleSlot> {
        match self {
            Self::PlatformAllocated { .. } => None,
            Self::ModuleAllocated { slot, .. } | Self::ModuleLocal { slot, .. } => Some(slot),
        }
    }

    const fn module_id(self) -> Option<ModuleID> {
        match self {
            Self::PlatformAllocated { .. } => None,
            Self::ModuleAllocated { module_id, .. } | Self::ModuleLocal { module_id, .. } => {
                Some(module_id)
            }
        }
    }
}

struct ResourceGroup {
    metadata: ResourceMetadata,
    capabilities: CapabilityMap,
}

struct ResourceGroupSlot {
    id: ResourceId,
    available: Option<ResourceGroup>,
}

/// Exclusive ownership of one physical resource group through capability `T`.
///
/// While this lease is outside the registry, every other capability in the
/// same group is unavailable. Return it with [`Registry::return_resource`] to
/// restore the complete group.
#[must_use = "dropping a resource lease permanently removes its complete group"]
pub struct ResourceLease<T> {
    registry_id: u32,
    group: ResourceGroup,
    resource_type: PhantomData<T>,
}

impl<T: 'static> ResourceLease<T> {
    pub const fn id(&self) -> ResourceId {
        self.group.metadata.id()
    }

    pub const fn origin(&self) -> ResourceOrigin {
        self.group.metadata.origin()
    }

    pub const fn slot(&self) -> Option<ModuleSlot> {
        self.group.metadata.slot()
    }

    pub const fn module_id(&self) -> Option<ModuleID> {
        self.group.metadata.module_id()
    }

    pub fn resource(&self) -> &T {
        self.group
            .capabilities
            .get(&TypeId::of::<T>())
            .and_then(|resource| resource.downcast_ref())
            .expect("resource lease contains its selected capability")
    }

    pub fn resource_mut(&mut self) -> &mut T {
        self.group
            .capabilities
            .get_mut(&TypeId::of::<T>())
            .and_then(|resource| resource.downcast_mut())
            .expect("resource lease contains its selected capability")
    }
}

mod private {
    pub trait GroupCapabilitiesSealed {}
    pub trait ResourceGroupsSealed {}
    pub trait ResourceSetSealed {}
}

/// A tuple of capabilities belonging to one physical resource group.
pub trait ResourceGroupCapabilities: private::GroupCapabilitiesSealed {
    #[doc(hidden)]
    fn into_capabilities(self) -> Result<CapabilityMap, RegistryError>;
}

/// A tuple of physical resource groups.
pub trait ResourceGroups: private::ResourceGroupsSealed {
    #[doc(hidden)]
    fn into_groups(self) -> Result<Vec<CapabilityMap>, RegistryError>;
}

/// A tuple of different resource types that must be allocated atomically.
pub trait ResourceSet: private::ResourceSetSealed {
    type Leases;

    #[doc(hidden)]
    fn is_available(registry: &Registry) -> bool;

    #[doc(hidden)]
    fn take(registry: &mut Registry) -> Option<Self::Leases>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum RegistryError {
    DuplicateCapability,
    DuplicateResourceId,
}

pub struct Registry {
    id: u32,
    groups: Vec<ResourceGroupSlot>,
    next_registry_allocated_id: u32,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self {
            id: NEXT_REGISTRY_ID
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .expect("registry id counter overflowed"),
            groups: Vec::new(),
            next_registry_allocated_id: 0,
        }
    }

    /// Registers a module resource as a single-capability physical group.
    pub fn register<T: 'static + Send>(
        &mut self,
        slot: ModuleSlot,
        module_id: ModuleID,
        resource: T,
    ) {
        let allocated_id = self.next_registry_allocated_id();
        self.insert_group(
            ResourceMetadata::ModuleAllocated {
                allocated_id,
                slot,
                module_id,
            },
            single_capability(resource),
        )
        .expect("registry-allocated resource IDs are unique");
    }

    /// Registers several logical capabilities backed by one physical module
    /// resource.
    pub fn register_group<C: ResourceGroupCapabilities>(
        &mut self,
        slot: ModuleSlot,
        module_id: ModuleID,
        capabilities: C,
    ) -> Result<(), RegistryError> {
        let capabilities = capabilities.into_capabilities()?;
        let allocated_id = self.next_registry_allocated_id();
        self.insert_group(
            ResourceMetadata::ModuleAllocated {
                allocated_id,
                slot,
                module_id,
            },
            capabilities,
        )
    }

    /// Registers a physical module group with a stable driver-defined local ID.
    pub fn register_local_group<C: ResourceGroupCapabilities>(
        &mut self,
        slot: ModuleSlot,
        module_id: ModuleID,
        local_id: u16,
        capabilities: C,
    ) -> Result<(), RegistryError> {
        self.insert_group(
            ResourceMetadata::ModuleLocal {
                local_id,
                slot,
                module_id,
            },
            capabilities.into_capabilities()?,
        )
    }

    /// Atomically registers several physical groups from one module.
    pub fn register_groups<G: ResourceGroups>(
        &mut self,
        slot: ModuleSlot,
        module_id: ModuleID,
        groups: G,
    ) -> Result<(), RegistryError> {
        let groups = groups.into_groups()?;
        for capabilities in groups {
            let allocated_id = self.next_registry_allocated_id();
            self.insert_group(
                ResourceMetadata::ModuleAllocated {
                    allocated_id,
                    slot,
                    module_id,
                },
                capabilities,
            )
            .expect("registry-allocated resource IDs are unique");
        }
        Ok(())
    }

    /// Registers a platform resource as a single-capability physical group.
    pub fn register_platform<T: 'static + Send>(&mut self, resource: T) {
        let allocated_id = self.next_registry_allocated_id();
        self.insert_group(
            ResourceMetadata::PlatformAllocated { allocated_id },
            single_capability(resource),
        )
        .expect("registry-allocated resource IDs are unique");
    }

    /// Registers several logical capabilities backed by one physical platform
    /// resource.
    pub fn register_platform_group<C: ResourceGroupCapabilities>(
        &mut self,
        capabilities: C,
    ) -> Result<(), RegistryError> {
        let capabilities = capabilities.into_capabilities()?;
        let allocated_id = self.next_registry_allocated_id();
        self.insert_group(
            ResourceMetadata::PlatformAllocated { allocated_id },
            capabilities,
        )
    }

    /// Returns the number of currently available groups providing `T`.
    pub fn resource_count<T: 'static + Send>(&self) -> usize {
        let resource_type = TypeId::of::<T>();
        self.groups
            .iter()
            .filter_map(|slot| slot.available.as_ref())
            .filter(|group| group.capabilities.contains_key(&resource_type))
            .count()
    }

    pub fn has<T: 'static + Send>(&self) -> bool {
        self.has_at_least::<T>(1)
    }

    pub fn has_at_least<T: 'static + Send>(&self, count: usize) -> bool {
        self.resource_count::<T>() >= count
    }

    /// Returns whether all resource types in `S` can be leased from different
    /// available physical groups.
    pub fn has_resource_set<S: ResourceSet>(&self) -> bool {
        S::is_available(self)
    }

    /// Leases one available physical group through capability `T`.
    pub fn take_resource<T: 'static + Send>(&mut self) -> Option<ResourceLease<T>> {
        let resource_type = TypeId::of::<T>();
        let index = self.groups.iter().rposition(|slot| {
            slot.available
                .as_ref()
                .is_some_and(|group| group.capabilities.contains_key(&resource_type))
        })?;
        Some(self.take_group_capability(index))
    }

    /// Atomically leases `count` different groups through capability `T`.
    pub fn take_resources<T: 'static + Send>(
        &mut self,
        count: usize,
    ) -> Option<Vec<ResourceLease<T>>> {
        if count == 0 {
            return Some(Vec::new());
        }

        let mut ids = self.resource_ids::<T>();
        if ids.len() < count {
            return None;
        }
        ids.truncate(count);

        Some(
            ids.into_iter()
                .map(|id| self.take_resource_with_id(id))
                .collect(),
        )
    }

    /// Atomically leases one resource of every type in `S` from different
    /// physical groups.
    pub fn take_resource_set<S: ResourceSet>(&mut self) -> Option<S::Leases> {
        S::take(self)
    }

    /// Returns a lease and makes its complete physical group available again.
    pub fn return_resource<T: 'static + Send>(&mut self, lease: ResourceLease<T>) {
        let ResourceLease {
            registry_id,
            group,
            resource_type: _,
        } = lease;
        assert_eq!(
            registry_id, self.id,
            "resource lease returned to a different registry"
        );
        let id = group.metadata.id();
        let slot = self
            .groups
            .iter_mut()
            .find(|slot| slot.id == id)
            .expect("leased resource group remains registered");
        assert!(
            slot.available.is_none(),
            "resource group cannot be returned while already available"
        );
        slot.available = Some(group);
    }

    fn insert_group(
        &mut self,
        metadata: ResourceMetadata,
        capabilities: CapabilityMap,
    ) -> Result<(), RegistryError> {
        let id = metadata.id();
        if self.groups.iter().any(|group| group.id == id) {
            return Err(RegistryError::DuplicateResourceId);
        }
        self.groups.push(ResourceGroupSlot {
            id,
            available: Some(ResourceGroup {
                metadata,
                capabilities,
            }),
        });
        Ok(())
    }

    fn next_registry_allocated_id(&mut self) -> u32 {
        let id = self.next_registry_allocated_id;
        self.next_registry_allocated_id = self
            .next_registry_allocated_id
            .checked_add(1)
            .expect("registry-allocated resource id counter overflowed");
        id
    }

    fn resource_ids<T: 'static + Send>(&self) -> Vec<ResourceId> {
        let resource_type = TypeId::of::<T>();
        self.groups
            .iter()
            .rev()
            .filter_map(|slot| slot.available.as_ref())
            .filter(|group| group.capabilities.contains_key(&resource_type))
            .map(|group| group.metadata.id())
            .collect()
    }

    fn take_resource_with_id<T: 'static + Send>(&mut self, id: ResourceId) -> ResourceLease<T> {
        let resource_type = TypeId::of::<T>();
        let index = self
            .groups
            .iter()
            .position(|slot| {
                slot.id == id
                    && slot
                        .available
                        .as_ref()
                        .is_some_and(|group| group.capabilities.contains_key(&resource_type))
            })
            .expect("resource assignment references an available capability");
        self.take_group_capability(index)
    }

    fn take_group_capability<T: 'static + Send>(&mut self, index: usize) -> ResourceLease<T> {
        let group = self.groups[index]
            .available
            .take()
            .expect("selected resource group is available");
        assert!(
            group
                .capabilities
                .get(&TypeId::of::<T>())
                .is_some_and(|resource| resource.is::<T>()),
            "selected resource group contains the requested capability"
        );

        ResourceLease {
            registry_id: self.id,
            group,
            resource_type: PhantomData,
        }
    }
}

fn single_capability<T: 'static + Send>(resource: T) -> CapabilityMap {
    let mut capabilities = BTreeMap::new();
    capabilities.insert(TypeId::of::<T>(), Box::new(resource) as Box<dyn Any + Send>);
    capabilities
}

fn resource_assignment(candidates: &[Vec<ResourceId>]) -> Option<Vec<ResourceId>> {
    fn assign(
        candidates: &[Vec<ResourceId>],
        candidate_index: usize,
        used: &mut Vec<ResourceId>,
    ) -> bool {
        if candidate_index == candidates.len() {
            return true;
        }

        for &id in &candidates[candidate_index] {
            if used.contains(&id) {
                continue;
            }

            used.push(id);
            if assign(candidates, candidate_index + 1, used) {
                return true;
            }
            used.pop();
        }

        false
    }

    let mut assignment = Vec::new();
    assign(candidates, 0, &mut assignment).then_some(assignment)
}

fn types_are_distinct(types: &[TypeId]) -> bool {
    types
        .iter()
        .enumerate()
        .all(|(index, resource_type)| !types[..index].contains(resource_type))
}

macro_rules! impl_group_capabilities {
    ($(($resource:ident, $value:ident)),+) => {
        impl<$($resource: 'static + Send),+> private::GroupCapabilitiesSealed
            for ($($resource,)+)
        {
        }

        impl<$($resource: 'static + Send),+> ResourceGroupCapabilities for ($($resource,)+) {
            fn into_capabilities(self) -> Result<CapabilityMap, RegistryError> {
                if !types_are_distinct(&[$(TypeId::of::<$resource>()),+]) {
                    return Err(RegistryError::DuplicateCapability);
                }

                let ($($value,)+) = self;
                let mut capabilities = BTreeMap::new();
                $(
                    capabilities.insert(
                        TypeId::of::<$resource>(),
                        Box::new($value) as Box<dyn Any + Send>,
                    );
                )+
                Ok(capabilities)
            }
        }
    };
}

impl_group_capabilities!((T1, t1), (T2, t2));
impl_group_capabilities!((T1, t1), (T2, t2), (T3, t3));
impl_group_capabilities!((T1, t1), (T2, t2), (T3, t3), (T4, t4));
impl_group_capabilities!((T1, t1), (T2, t2), (T3, t3), (T4, t4), (T5, t5));
impl_group_capabilities!((T1, t1), (T2, t2), (T3, t3), (T4, t4), (T5, t5), (T6, t6));
impl_group_capabilities!(
    (T1, t1),
    (T2, t2),
    (T3, t3),
    (T4, t4),
    (T5, t5),
    (T6, t6),
    (T7, t7)
);
impl_group_capabilities!(
    (T1, t1),
    (T2, t2),
    (T3, t3),
    (T4, t4),
    (T5, t5),
    (T6, t6),
    (T7, t7),
    (T8, t8)
);

macro_rules! impl_resource_groups {
    ($(($group:ident, $value:ident)),+) => {
        impl<$($group: ResourceGroupCapabilities),+> private::ResourceGroupsSealed
            for ($($group,)+)
        {
        }

        impl<$($group: ResourceGroupCapabilities),+> ResourceGroups for ($($group,)+) {
            fn into_groups(self) -> Result<Vec<CapabilityMap>, RegistryError> {
                let ($($value,)+) = self;
                Ok(vec![$(($value.into_capabilities()?),)+])
            }
        }
    };
}

impl_resource_groups!((G1, g1), (G2, g2));
impl_resource_groups!((G1, g1), (G2, g2), (G3, g3));
impl_resource_groups!((G1, g1), (G2, g2), (G3, g3), (G4, g4));
impl_resource_groups!((G1, g1), (G2, g2), (G3, g3), (G4, g4), (G5, g5));
impl_resource_groups!((G1, g1), (G2, g2), (G3, g3), (G4, g4), (G5, g5), (G6, g6));
impl_resource_groups!(
    (G1, g1),
    (G2, g2),
    (G3, g3),
    (G4, g4),
    (G5, g5),
    (G6, g6),
    (G7, g7)
);
impl_resource_groups!(
    (G1, g1),
    (G2, g2),
    (G3, g3),
    (G4, g4),
    (G5, g5),
    (G6, g6),
    (G7, g7),
    (G8, g8)
);

macro_rules! impl_resource_set {
    ($($resource:ident),+) => {
        impl<$($resource: 'static + Send),+> private::ResourceSetSealed for ($($resource,)+) {}

        impl<$($resource: 'static + Send),+> ResourceSet for ($($resource,)+) {
            type Leases = ($(ResourceLease<$resource>,)+);

            fn is_available(registry: &Registry) -> bool {
                if !types_are_distinct(&[$(TypeId::of::<$resource>()),+]) {
                    return false;
                }

                let candidates = [$(registry.resource_ids::<$resource>()),+];
                resource_assignment(&candidates).is_some()
            }

            fn take(registry: &mut Registry) -> Option<Self::Leases> {
                if !types_are_distinct(&[$(TypeId::of::<$resource>()),+]) {
                    return None;
                }

                let candidates = [$(registry.resource_ids::<$resource>()),+];
                let assignment = resource_assignment(&candidates)?;
                let mut ids = assignment.into_iter();
                Some(($(
                    registry.take_resource_with_id::<$resource>(
                        ids.next().expect("resource assignment contains every requested type"),
                    ),
                )+))
            }
        }
    };
}

impl_resource_set!(T1, T2);
impl_resource_set!(T1, T2, T3);
impl_resource_set!(T1, T2, T3, T4);
impl_resource_set!(T1, T2, T3, T4, T5);
impl_resource_set!(T1, T2, T3, T4, T5, T6);
impl_resource_set!(T1, T2, T3, T4, T5, T6, T7);
impl_resource_set!(T1, T2, T3, T4, T5, T6, T7, T8);
