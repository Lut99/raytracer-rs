//  GRADIENT.rs
//    by Lut99
//
//  Description:
//!   Defines texture that implement a texture-coordinate based gradient.
//

use std::convert::Infallible;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::Textured;
use crate::math::{Colour, Vec3};


/***** LIBRARY *****/
/// A gradient that goes from one colour to the other.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Gradient {
    /// The one colour.
    pub colour1: Colour,
    /// The other colour.
    pub colour2: Colour,
}

// Interfaces
impl Loadable for Gradient {
    type Error = Infallible;

    #[inline]
    fn load(&mut self, _dir: &Path) -> Result<(), Self::Error> { Ok(()) }
}
impl Textured for Gradient {
    #[inline]
    fn value(&self, uv: (f64, f64), _p: Vec3) -> Colour {
        // Compute the average gradient
        let f: f64 = 0.5 * (uv.0 + uv.1);
        (1.0 - f) * self.colour1 + f * self.colour2
    }
}
