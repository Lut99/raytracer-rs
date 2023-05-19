//  MOD.rs
//    by Lut99
// 
//  Created:
//    19 May 2023, 11:30:47
//  Last edited:
//    19 May 2023, 11:43:07
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines a single-threaded backend renderer. Essentially the most
//!   straightforward renderer you can imagine.
// 

// Declare submodules
pub mod renderer;

// Bring some of it into this namespace
pub use renderer::SingleThreadRenderer;
