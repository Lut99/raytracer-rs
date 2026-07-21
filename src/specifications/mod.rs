//  MOD.rs
//    by Lut99
//
//  Created:
//    23 Apr 2023, 11:40:34
//  Last edited:
//    05 May 2023, 10:41:59
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the "outside world" specifications for the `raytracer`.
//!   Contains stuff like file layouts, network messages (if applicable).
//

// Declare the submodules
pub mod animations;
pub mod features;
pub mod materials;
pub mod objects;
pub mod scene;
pub mod textures;

// Imports
use std::error::Error;


/***** LIBRARY *****/
/// Defines that something might be referenced externally and might need to be loaded yet.
pub trait Loadable {
    type Error: Error;

    /// Ensures that any external references in this instance are loaded.
    ///
    /// After execution, it should be valid for rendering.
    ///
    /// # Errors
    /// This function can error if we failed to find -or load- the external file.
    fn load(&mut self) -> Result<(), Self::Error>;
}
