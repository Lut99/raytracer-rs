//  MOD.rs
//    by Lut99
// 
//  Created:
//    01 May 2023, 18:54:46
//  Last edited:
//    05 May 2023, 11:17:42
//  Auto updated?
//    Yes
// 
//  Description:
//!   The `objects` module defines the objects to which we can render. It
//!   is structured object-oriented _like_, but because we use our
//!   ECS-like [`crate::hitlist::HitList`] and we never turn the objects
//!   into dynamic trait instances, we won't have the downsides of virtual
//!   function pointers.
// 

// Define the submodules
pub mod spec;
pub mod utils;
pub mod sphere;

// Bring some of this into this namespace
pub use spec::*;
pub use sphere::Sphere;
