//  MATH.rs
//    by Lut99
//
//  Description:
//!   Defines the math we need for the inspector.
//

// Modules
pub mod camera;
pub mod mat4;

// (Re-)Exports
pub use camera::Camera;
pub use mat4::Mat4;
pub use raytracer::math::aabb::Interval;
pub use raytracer::math::{Vec3, vec3};
