//  MOD.rs
//    by Lut99
// 
//  Created:
//    05 May 2023, 10:41:36
//  Last edited:
//    05 May 2023, 11:44:16
//  Auto updated?
//    Yes
// 
//  Description:
//!   The `materials` module defines the various materials we can render
//!   to. While it is structured object-oriented-like, we never call the
//!   material as a dynamic trait object. This way, we can get OOP design
//!   pros with functional speeds.
// 

// Declare submodules
pub mod spec;
pub mod simple;
pub mod diffuse;

// Put some of it into the module namespace
pub use spec::*;
pub use simple::NormalMap;
pub use diffuse::Diffuse;
