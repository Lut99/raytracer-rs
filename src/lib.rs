//  LIB.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 11:41:15
//  Last edited:
//    30 Apr 2023, 11:38:34
//  Auto updated?
//    Yes
// 
//  Description:
//!   A new attempt at writing a simple raytracer. This time, no fumbling
//!   about with real-time stuff, but instead creating a good-old, offline
//!   renderer. Based on
//!   <https://raytracing.github.io/books/RayTracingInOneWeekend.html>.
// 

// Declare the library modules
pub mod common;
pub mod math;
pub mod specifications;
pub mod hitlist;

// Declare the subcommand modules
pub mod generate;
pub mod render;
