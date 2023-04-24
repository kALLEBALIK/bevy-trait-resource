//! # bevy-trait-resource
//! A way to get resources that implements a specific trait.
//! 
//! ```rust
//! use bevy_trait_resource::*;
//! use bevy::prelude::*;
//! use bevy_trait_resource::{trait_resource, TraitResourceExt};
//! 
//! #[trait_resource]
//! pub trait IncrementTrait {
//!     fn value(&self) -> i32;
//!     fn increment(&mut self);
//! }
//! 
//! #[derive(Resource, Default)]
//! struct NumberValueResource {
//!     value: i32,
//! }
//! 
//! impl IncrementTrait for NumberValueResource {
//!#     fn value(&self) -> i32 {
//!#         self.value
//!#     }
//!#     fn increment(&mut self) {
//!#         self.value += 1;
//!#     }
//!     // Trait implementation...
//! }
//! 
//! pub fn increment_value_system(world: &mut World) {
//!     for res_opt in world.get_resources_trait_mut::<dyn IncrementTrait>() {
//!         if let Some(res) = res_opt {
//!             res.increment();
//!         }
//!     }
//! } 
//! 
//! struct SomePlugin;
//! 
//! impl Plugin for SomePlugin {
//!     fn build(&self, app: &mut App) {
//!         app.init_resource_as::<dyn IncrementTrait, NumberValueResource>();
//!         app.add_system(increment_value_system);
//!     }
//! }
//!# 
//!# fn main() {
//!#    App::new()
//!#        .add_plugin(SomePlugin)
//!#        .run();
//!# }
//! ```
//! You can also register a resource to multiple traits.
//! ```rust
//!# use bevy_trait_resource::*;
//!# use bevy::prelude::*;
//!# use bevy_trait_resource::{trait_resource, TraitResourceExt};
//!# 
//!# #[trait_resource]
//!# pub trait IncrementTrait {
//!#     fn value(&self) -> i32;
//!#     fn increment(&mut self);
//!# }
//!# 
//!# #[derive(Resource, Default)]
//!# struct NumberValueResource {
//!#     value: i32,
//!# }
//!# 
//!# impl IncrementTrait for NumberValueResource {
//!#     fn value(&self) -> i32 {
//!#         self.value
//!#     }
//!#     fn increment(&mut self) {
//!#         self.value += 1;
//!#     }
//!# }
//!# 
//!# #[trait_resource]
//!# pub trait SomeOtherTrait  {
//!#     fn some_other_value(&self) -> i32;
//!# }
//!# 
//!# impl SomeOtherTrait for NumberValueResource {
//!#     fn some_other_value(&self) -> i32 {
//!#         self.value
//!#     }
//!# }
//!# 
//!# pub fn increment_value_system(world: &mut World) {
//!#     for res_opt in world.get_resources_trait_mut::<dyn IncrementTrait>() {
//!#         if let Some(res) = res_opt {
//!#             res.increment();
//!#         }
//!#     }
//!# } 
//!# 
//!# struct SomePlugin;
//!# 
//! impl Plugin for SomePlugin {
//!     fn build(&self, app: &mut App) {
//!         app.init_resource::<NumberValueResource>();
//! 
//!         app.register_resource_as::<dyn IncrementTrait, NumberValueResource>()
//!            .register_resource_as::<dyn SomeOtherTrait, NumberValueResource>();
//!     }
//! }
//! 
//!# fn main() {
//!#    App::new()
//!#        .add_plugin(SomePlugin)
//!#        .run();
//!# }
//! ```
//! Unregistering.
//! ```rust
//!# use bevy_trait_resource::*;
//!# use bevy::prelude::*;
//!# use bevy_trait_resource::{trait_resource, TraitResourceExt};
//!# 
//!# #[trait_resource]
//!# pub trait IncrementTrait {
//!#     fn value(&self) -> i32;
//!#     fn increment(&mut self);
//!# }
//!# 
//!# #[derive(Resource, Default)]
//!# struct NumberValueResource {
//!#     value: i32,
//!# }
//!# 
//!# impl IncrementTrait for NumberValueResource {
//!#     fn value(&self) -> i32 {
//!#         self.value
//!#     }
//!#     fn increment(&mut self) {
//!#         self.value += 1;
//!#     }
//!# }
//!# 
//!# struct SomePlugin; 
//!#
//!# impl Plugin for SomePlugin {
//!#    fn build(&self, app: &mut App) {
//! app.unregister_resource_from_trait::<dyn IncrementTrait, NumberValueResource>();
//!#    }
//!# }
//!#
//!# fn main() {
//!#    App::new()
//!#        .add_plugin(SomePlugin)
//!#        .run();
//!# }
//! ```

use std::marker::PhantomData;
use bevy::{
    ecs::component::ComponentId,
    prelude::*,
    ptr::{Ptr, PtrMut}
};

#[cfg(test)]
mod tests;

pub use bevy_trait_resource_macro::trait_resource;

#[doc(hidden)]
pub mod imports {
    pub use bevy::ecs::{
            world::World,
            system::Resource,
    };
}

pub trait TraitResource: 'static {}

#[doc(hidden)]
pub trait TraitResourceMarker<Trait: ?Sized + TraitResource> {
    type Covered: Resource;
    fn cast(_: *mut u8) -> *mut Trait;
}

/// Turns an untyped pointer into a trait object pointer,
/// for a specific erased concrete type.
struct DynCtor<Trait: ?Sized> {
    cast: unsafe fn(*mut u8) -> *mut Trait,
}

impl<T: ?Sized> Copy for DynCtor<T> {}
impl<T: ?Sized> Clone for DynCtor<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Trait: ?Sized> DynCtor<Trait> {
    #[inline]
    unsafe fn cast(self, ptr: Ptr) -> &Trait {
        &*(self.cast)(ptr.as_ptr())
    }
    #[inline]
    unsafe fn cast_mut(self, ptr: PtrMut) -> &mut Trait {
        &mut *(self.cast)(ptr.as_ptr())
    }
}

struct TraitData<Trait: ?Sized> {
    resource_component_id: ComponentId,
    trait_ptr: DynCtor<Trait>,
}

impl<T: ?Sized> Copy for TraitData<T> {}
impl<T: ?Sized> Clone for TraitData<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Resource)]
struct TraitResourceRegistry<Trait: ?Sized> {
    trait_data: Vec<TraitData<Trait>>,
}

impl<Trait: ?Sized> TraitResourceRegistry<Trait> {
    fn empty() -> Self {
        Self { trait_data: vec![] }
    }
}

impl<Trait: ?Sized> Default for TraitResourceRegistry<Trait> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<Trait: ?Sized + TraitResource> TraitResourceRegistry<Trait> {
    /// Registers data for resource trait.
    /// Overrides data with same component id.
    fn register(&mut self, trait_data: TraitData<Trait>) {
        let exists_in_index = self
            .trait_data
            .iter()
            .position(|data| data.resource_component_id == trait_data.resource_component_id);

        if let Some(index) = exists_in_index {
            self.trait_data[index] = trait_data;
        } else {
            self.trait_data.push(trait_data);
        }
    }

    /// Unregister the component from the registry.
    /// Returns the new length of the registry.
    fn unregister(&mut self, resource_component_id: ComponentId) -> usize {
        self.trait_data.retain(|data| data.resource_component_id != resource_component_id);
        self.trait_data.len()
    }
}

/// An [`Iterator`] over resources as mutables that implements a trait.
pub struct TraitResourceIteratorMut<'w, Trait: ?Sized + TraitResource> {
    world: &'w mut World,
    cursor: usize,
    _marker: PhantomData<Trait>
}

impl<'w, Trait: ?Sized + TraitResource> TraitResourceIteratorMut<'w, Trait> {
    fn new(world: &'w mut World) -> Self {
        Self {
            world,
            cursor: 0,
            _marker: PhantomData
        }
    }
}

impl<'w, Trait: ?Sized + TraitResource> Iterator for TraitResourceIteratorMut<'w, Trait> {
    type Item = Option<&'w mut Trait>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Avoid getting resource every next call? (This is done because we cant have two mutable borrows of world).
        if let Some(registry) = self.world.get_resource_mut::<TraitResourceRegistry<Trait>>()  {
            if self.cursor >= registry.trait_data.len() {
                return None;
            }

            let data = registry.trait_data[self.cursor];
            self.cursor += 1;

            if let Some(mut ptr) = self.world.get_resource_mut_by_id(data.resource_component_id) {
                // SAFETY: We know that the pointer is valid because we got it from the world.
                // We are also guaranteed that there is no duplicates of resource_component_id and resources
                // and we can therefore "guarantee" that the iterator will only return one mutable
                // reference to a single resource. So this should be OK?
                let raw_ptr = unsafe { data.trait_ptr.cast_mut(ptr.as_mut()) as *mut Trait };
                Some(Some(unsafe { &mut *raw_ptr }))
            } else {
                Some(None)
            }
        } else {
            None
        }
    }
}

/// An [`Iterator`] over resources that implements a trait.
pub struct TraitResourceIterator<'w, Trait: ?Sized + TraitResource> {
    registry: Option<&'w TraitResourceRegistry<Trait>>,
    world: &'w World,
    cursor: usize,
}

impl<'w, Trait: ?Sized + TraitResource> TraitResourceIterator<'w, Trait> {
    fn new(world: &'w World) -> Self {
        Self {
            registry: world.get_resource::<TraitResourceRegistry<Trait>>(),
            world,
            cursor: 0,
        }
    }
}

impl<'w, Trait: ?Sized + TraitResource> Iterator for TraitResourceIterator<'w, Trait> {
    type Item = Option<&'w Trait>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(registry) = self.registry {
            if self.cursor >= registry.trait_data.len() {
                return None
            }

            let data = registry.trait_data[self.cursor];
            self.cursor += 1;

            if let Some(ptr) = self.world.get_resource_by_id(data.resource_component_id) {
                Some(Some(unsafe { data.trait_ptr.cast(ptr) }))
            } else {
                Some(None)
            }
        } else {
            None
        }
    }
}

pub trait TraitResourceExt {
    /// Inserts a resource into the world and registers its trait.
    /// if the resource already exists, it will be overridden.
    fn insert_resource_as<Trait: ?Sized + TraitResource, R: Resource>(&mut self, resource: R) -> &mut Self
    where (R,): TraitResourceMarker<Trait, Covered = R>;

    /// Initializes a resource into the world and registers its trait.
    /// if the resource already exists, it will be overridden.
    fn init_resource_as<Trait: ?Sized + TraitResource, R: Resource + Default>(&mut self) -> &mut Self
    where (R,): TraitResourceMarker<Trait, Covered = R>;

    // ?: Should this maybe warn instead of panic?
    /// Registers a resource as implementing a trait.
    /// If the resource is already registered nothing will happen.
    /// # Panics 
    /// Panics if the resource does not exist.
    fn register_resource_as<Trait: ?Sized + TraitResource, R: Resource>(&mut self) -> &mut Self
    where (R,): TraitResourceMarker<Trait, Covered = R>;

    /// Get [`TraitResourceIterator<Trait>`]
    fn get_resources_trait<Trait: ?Sized + TraitResource>(&self) -> TraitResourceIterator<Trait>;

    /// Get [`TraitResourceIteratorMut<Trait>`]
    fn get_resources_trait_mut<Trait: ?Sized + TraitResource>(&mut self) -> TraitResourceIteratorMut<Trait>;

    /// Unregister a resource from trait.
    fn unregister_resource_from_trait<Trait: ?Sized + TraitResource, R: Resource>(&mut self)
    where (R,): TraitResourceMarker<Trait, Covered = R>;
}

impl TraitResourceExt for World {
    fn insert_resource_as<Trait: ?Sized + TraitResource, R: Resource>(
        &mut self,
        resource: R,
    ) -> &mut Self
    where
        (R,): TraitResourceMarker<Trait, Covered = R>,
    {
        self.insert_resource(resource);
        self.register_resource_as::<Trait, R>();
        self
    }

    fn init_resource_as<Trait: ?Sized + TraitResource, R: Resource + Default>(&mut self) -> &mut Self
    where
        (R,): TraitResourceMarker<Trait, Covered = R>
    {
        self.insert_resource_as::<Trait, R>(R::default());
        self
    }

    fn register_resource_as<Trait: ?Sized + TraitResource, R: Resource>(&mut self) -> &mut Self
    where
        (R,): TraitResourceMarker<Trait, Covered = R>,
    {
        let resource_id = self
            .components()
            .resource_id::<R>()
            .expect("Trying to register a nonexistent resource");

        let resource_registry = self
            .get_resource_or_insert_with::<TraitResourceRegistry<Trait>>(default)
            .into_inner();

        let trait_data = TraitData {
            resource_component_id: resource_id,
            trait_ptr: DynCtor { cast: <(R,)>::cast },
        };

        resource_registry.register(trait_data);
        self
    }

    fn get_resources_trait<Trait: ?Sized + TraitResource>(&self) -> TraitResourceIterator<Trait> {
        TraitResourceIterator::new(self)
    }

    fn get_resources_trait_mut<Trait: ?Sized + TraitResource>(&mut self) -> TraitResourceIteratorMut<Trait> {
        TraitResourceIteratorMut::new(self)
    }

    // ? Should this maybe warn if the resource does not exists, because we should 
    // ? unregister before removing the resource if we also want to remove the resource
    // Also removes the registry if it is empty.
    fn unregister_resource_from_trait<Trait: ?Sized + TraitResource, R: Resource>(&mut self)
    where
        (R,): TraitResourceMarker<Trait, Covered = R>
    {
        let resource_id_opt = self
            .components()
            .resource_id::<R>();

        if let Some(resource_id) = resource_id_opt {
            match self.get_resource_mut::<TraitResourceRegistry<Trait>>() {
                Some(mut registry) => {
                    let new_size = registry.unregister(resource_id);
                    if new_size == 0 {
                        self.remove_resource::<TraitResourceRegistry<Trait>>();
                    }
                },
                _ => {}
            }
        }
    }
}

impl TraitResourceExt for App {
    fn insert_resource_as<Trait: ?Sized + TraitResource, R: Resource>(
        &mut self,
        resource: R,
    ) -> &mut Self
    where
        (R,): TraitResourceMarker<Trait, Covered = R>,
    {
        self.world.insert_resource_as::<Trait, R>(resource);
        self
    }

    fn init_resource_as<Trait: ?Sized + TraitResource, R: Resource + Default>(&mut self) -> &mut Self
    where
        (R,): TraitResourceMarker<Trait, Covered = R>
    {
        self.world.init_resource_as::<Trait, R>();
        self
    }

    fn register_resource_as<Trait: ?Sized + TraitResource, R: Resource>(&mut self) -> &mut Self
    where
        (R,): TraitResourceMarker<Trait, Covered = R> 
    {
        self.world.register_resource_as::<Trait, R>();
        self
    }

    fn get_resources_trait<Trait: ?Sized + TraitResource>(&self) -> TraitResourceIterator<Trait> {
        self.world.get_resources_trait::<Trait>()
    }

    fn get_resources_trait_mut<Trait: ?Sized + TraitResource>(&mut self) -> TraitResourceIteratorMut<Trait> {
        self.world.get_resources_trait_mut::<Trait>()
    }

    fn unregister_resource_from_trait<Trait: ?Sized + TraitResource, R: Resource>(&mut self)
    where
        (R,): TraitResourceMarker<Trait, Covered = R>
    {
        self.world.unregister_resource_from_trait::<Trait, R>();
    }
}
