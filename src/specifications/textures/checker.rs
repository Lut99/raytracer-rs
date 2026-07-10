//  CHECKER.rs
//    by Lut99
//
//  Description:
//!   Implements a checker texture.
//

use serde::{Deserialize, Serialize};

use super::Textured;
use crate::math::{Colour, Vec3};


/***** LIBRARY *****/
/// A texture that renders a checkerboard pattern.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Checker {
    /// The size of the checkers in space.
    pub scale: f64,
    /// The colour of the "black" squares.
    pub black: Colour,
    /// The colour of  the "white" squares.
    pub white: Colour,
}

// Textures
impl Textured for Checker {
    #[inline]
    fn value(&self, uv: (f64, f64), _p: Vec3) -> Colour {
        // Let's implement it as a spatial texture; we will compute the pattern based on the hit's
        // location in space, essentially colouring space.
        let iscale: f64 = 1.0 / self.scale;
        // We get some binned version of the floating-point coordinate (the scale determines the
        // size of the bins)
        let ix: i32 = (iscale * uv.0).floor() as i32;
        let iy: i32 = (iscale * uv.1).floor() as i32;
        // If the bin is somehow even, it's a white checkerboard; else it's black.
        if (ix + iy) % 2 == 0 { self.white } else { self.black }
    }
}
