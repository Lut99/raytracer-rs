//  SYSTEM.rs
//    by Lut99
// 
//  Created:
//    29 Apr 2023, 10:51:41
//  Last edited:
//    29 Apr 2023, 11:13:14
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the [`Ecs`] itself.
// 

use std::any::TypeId;
use std::collections::HashMap;

use super::spec::{Component, ComponentSalt, Entity};


/***** LIBRARY *****/
/// The Entity Component System that we use to efficiently abstract over different entities we might want to render.
#[derive(Debug)]
pub struct Ecs {
    /// The first available new Entity ID.
    id         : u64,
    /// The list of ComponentLists.
    components : HashMap<(TypeId, Option<u64>), Vec<Box<dyn Component>>>,
}

impl Ecs {
    /// Constructor for the Ecs that initializes it with nothing in it.
    /// 
    /// # Returns
    /// A new instance of Self that can be used to register, then add entities and components.
    #[inline]
    pub fn new() -> Self {
        Self {
            id         : 0,
            components : HashMap::with_capacity(16),
        }
    }



    /// Registers a new Component with the system.
    /// 
    /// Will override whathever list of components was already there if the entity was already registered.
    /// 
    /// # Generic arguments
    /// - `C`: The [`Component`] to register.
    /// 
    /// # Returns
    /// Whether this component was already registered or not.
    #[inline]
    pub fn register<C: Component>(&mut self) -> bool {
        self.components.insert((TypeId::of::<C>(), None), Vec::with_capacity(16)).is_some()
    }

    /// Registers a new Component with the system in a way that allows them to be distinguished at runtime.
    /// 
    /// Will override whathever list of components was already there if the entity was already registered.
    /// 
    /// # Generic arguments
    /// - `C`: The [`Component`] to register.
    /// 
    /// # Arguments
    /// - `salt`: Some additional identifier that distinguishes it from equally typed other components.
    /// 
    /// # Returns
    /// Whether this component was already registered or not.
    #[inline]
    pub fn register_with_id<C: Component>(&mut self, salt: impl ComponentSalt) -> bool {
        self.components.insert((TypeId::of::<C>(), Some(salt.variant())), Vec::with_capacity(16)).is_some()
    }



    /// Creates a new entity within the system without any components to it.
    /// 
    /// # Returns
    /// The [`Entity`] identifier of the new entity. Will be unique among the currently existing entities.
    pub fn add(&mut self) -> Entity {
        let id: u64 = self.id;
        self.id += 1;
        Entity(id)
    }

    /// Removes the given entity from the system.
    /// 
    /// # Arguments
    /// - `entity`: The [`Entity`] to remove.
    /// 
    /// # Returns
    /// Whether the Entity was removed or not.
    pub fn remove(&mut self, entity: Entity) -> bool {
        // Iterate over all entities to find its components
        let mut removed: bool = false;
        for list in self.components.values_mut() {
            removed |= list.remove(entity).is_some();
        }
        removed
    }



    /// Returns a component of a given entity.
    /// 
    /// # Generic arguments
    /// - `C`: The [`Component`] to get.
    /// 
    /// # Arguments
    /// - `salt`: Some runtime-dependent identifier ([`ComponentSalt`]) of the [`Component`], if any.
    /// - `entity`: The [`Entity`] to return the component of.
    /// 
    /// # Returns
    /// A reference to the entity's component if it has one.
    pub fn component<C: Component>(&self, salt: Option<impl ComponentSalt>, entity: Entity) -> Option<&C> {
        None
    }
}
