//  MOD.rs
//    by Lut99
// 
//  Created:
//    29 Apr 2023, 10:50:46
//  Last edited:
//    29 Apr 2023, 10:55:53
//  Auto updated?
//    Yes
// 
//  Description:
//!   The so-manieth implementation of an Entity Component System (ECS).
//!   This one is tuned for usage with our raytracer.
// 

// Declare submodules
pub mod spec;
pub mod system;

// Get some stuff into the module namespace
pub use spec::{Component, Entity};
pub use system::Ecs;
