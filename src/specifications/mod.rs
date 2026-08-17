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
use std::cell::RefMut;
use std::error::Error;
use std::path::Path;
use std::sync::{MutexGuard, RwLockWriteGuard};


/***** HELPER MACROS *****/
/// Implements pointer-like impls for [`Loadable`].
macro_rules! loadable_ptr_impl {
    ('a, $ty:ty) => {
        impl<'a, T: Loadable> Loadable for $ty {
            type Error = <T as Loadable>::Error;

            #[inline]
            fn load(&mut self, dir: &Path) -> Result<(), Self::Error> { <T as Loadable>::load(self, dir) }
        }
    };
    ($ty:ty) => {
        impl<T: Loadable> Loadable for $ty {
            type Error = <T as Loadable>::Error;

            #[inline]
            fn load(&mut self, dir: &Path) -> Result<(), Self::Error> { <T as Loadable>::load(self, dir) }
        }
    };
}





/***** LIBRARY *****/
/// Defines that something might be referenced externally and might need to be loaded yet.
pub trait Loadable {
    type Error: Error;

    /// Ensures that any external references in this instance are loaded.
    ///
    /// After execution, it should be valid for rendering.
    ///
    /// # Arguments
    /// - `dir`: The directory in which the file defining the something exists. This might be used
    ///   to resolve relative paths.
    ///
    /// # Errors
    /// This function can error if we failed to find -or load- the external file.
    fn load(&mut self, dir: &Path) -> Result<(), Self::Error>;
}

// Pointer-like impls
loadable_ptr_impl!('a, &'a mut T);
loadable_ptr_impl!(Box<T>);
loadable_ptr_impl!('a, RefMut<'a, T>);
loadable_ptr_impl!('a, RwLockWriteGuard<'a, T>);
loadable_ptr_impl!('a, MutexGuard<'a, T>);
loadable_ptr_impl!('a, parking_lot::RwLockWriteGuard<'a, T>);
loadable_ptr_impl!('a, parking_lot::MutexGuard<'a, T>);
