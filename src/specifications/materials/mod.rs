//  MOD.rs
//    by Lut99
//
//  Created:
//    05 May 2023, 10:41:36
//  Last edited:
//    07 May 2023, 10:51:52
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
pub mod dielectric;
pub mod diffuse;
pub mod metal;
pub mod simple;

// Put some of it into the module namespace
pub use dielectric::{Dielectric, PartialDielectric};
pub use diffuse::{Diffuse, Lambertian};
pub use metal::Metal;
pub use simple::{NormalMap, StaticColour};

// Imports
use crate::math::{Colour, Ray};
use crate::specifications::objects::HitRecord;
use crate::specifications::scene::Environment;


/***** LIBRARY *****/
/// The Material trait implements any material that we can use to cover an object.
pub trait Material {
    /// Bounces (or reflects) a ray from this material.
    ///
    /// # Arguments
    /// - `ray`: The inbound [`Ray`] that we want to scatter.
    /// - `record`: The [`HitRecord`] that determines where the hit was and what the hit normal was
    ///   and such.
    /// - `env`: An [`Environment`] object relating properties about the scene's global
    ///   environment.
    ///
    /// # Returns
    /// A tuple that represents the bounced [`Ray`] and the attenuated colour from this bounce. If
    /// [`None`] is returned for the [`Ray`], then no more bounce is necessary.
    fn scatter(&self, ray: Ray, record: HitRecord, env: &Environment) -> (Option<Ray>, Colour);
}
