//  SIMPLE.rs
//    by Lut99
//
//  Created:
//    05 May 2023, 11:41:04
//  Last edited:
//    07 May 2023, 10:51:40
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines super-simple materials, mostly used for debugging or to
//!   represent earlier parts of the tutorial.
//

use std::convert::Infallible;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::super::objects::HitData;
use super::Scattering;
use crate::math::Colour;


/***** LIBRARY *****/
/// Implements a non-bouncing, static-colour material.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct StaticColour {
    /// The colour to show.
    pub colour: Colour,
}
impl Loadable for StaticColour {
    type Error = Infallible;

    #[inline]
    fn load(&mut self, _dir: &Path) -> Result<(), Self::Error> { Ok(()) }
}
impl Scattering for StaticColour {
    type PDF = Infallible;

    #[inline]
    fn emitted(&self, _rec: &HitData) -> Colour { self.colour }
}



/// Implements a non-bouncing, just-normal-map kind of material. Mostly created for the scene in the [tutorial](https://raytracing.github.io/books/RayTracingInOneWeekend.html#surfacenormalsandmultipleobjects/commonconstantsandutilityfunctions).
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct NormalMap;
impl Loadable for NormalMap {
    type Error = Infallible;

    #[inline]
    fn load(&mut self, _dir: &Path) -> Result<(), Self::Error> { Ok(()) }
}
impl Scattering for NormalMap {
    type PDF = Infallible;

    #[inline]
    fn emitted(&self, rec: &HitData) -> Colour { 0.5 * Colour::new(rec.normal.x + 1.0, rec.normal.y + 1.0, rec.normal.z + 1.0, 2.0) }
}
