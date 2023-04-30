//  MOD.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2023, 13:27:16
//  Last edited:
//    30 Apr 2023, 12:04:35
//  Auto updated?
//    Yes
// 
//  Description:
//!   The `math` module implements the math needed for the RayTracer.
// 

// Declare the submodules
pub mod utils;
pub mod colour;
pub mod vec3;
pub mod ray;
pub mod aabb;
pub mod camera;

// Bring some stuff into the global namespace for convenience
pub use colour::Colour;
pub use vec3::{Vec3, Vector};
pub use ray::Ray;
pub use aabb::AABB;
pub use camera::Camera;
