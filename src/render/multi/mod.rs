//  MOD.rs
//    by Lut99
// 
//  Created:
//    19 May 2023, 11:31:15
//  Last edited:
//    19 May 2023, 12:48:29
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines a multi-threaded CPU renderer backend.
// 

// Declare submodules
pub mod renderer;

// Bring some of it into this namespace
pub use renderer::{Error, MultiThreadRenderer, MultiThreadRendererConfig};
