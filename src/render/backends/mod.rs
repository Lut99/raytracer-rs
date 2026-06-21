//  FLAVORS.rs
//    by Lut99
//
//  Description:
//!   Implements the different backends for rendering rays.
//

// Modules
mod cpu;
pub mod multi;
pub mod single;

// Module-wide imports
pub use multi::MultiThreadRenderer;
pub use single::SingleThreadRenderer;
