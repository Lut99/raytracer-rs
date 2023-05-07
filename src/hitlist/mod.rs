//  MOD.rs
//    by Lut99
// 
//  Created:
//    29 Apr 2023, 10:50:46
//  Last edited:
//    07 May 2023, 12:17:27
//  Auto updated?
//    Yes
// 
//  Description:
//!   The so-manieth implementation of an Entity Component System (ECS).
//!   This one is tuned for usage with our raytracer, and thus called a
//!   [`HitList`] instead of an ECS.
// 

// Declare submodules
pub mod hitlist;

// Get some stuff into the module namespace
pub use hitlist::{HitIndex, HitList};
