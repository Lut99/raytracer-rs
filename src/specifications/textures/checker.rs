//  CHECKER.rs
//    by Lut99
//
//  Description:
//!   Implements a checker texture.
//

use std::convert::Infallible;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::Textured;
use crate::math::{Colour, Vec3};


/***** HELPER FUNCTIONS *****/
/// Computes a colour based on the spatial position of the ray hit.
#[inline]
pub fn spatial_checker_value(scale: f64, white: Colour, black: Colour, p: Vec3) -> Colour {
    // Let's implement it as a spatial texture; we will compute the pattern based on the hit's
    // location in space, essentially colouring space.
    let iscale: f64 = 1.0 / scale;
    // We get some binned version of the floating-point coordinate (the scale determines the
    // size of the bins)
    let ix: i32 = (iscale * p.x).floor() as i32;
    let iy: i32 = (iscale * p.y).floor() as i32;
    let iz: i32 = (iscale * p.z).floor() as i32;
    // If the bin is somehow even, it's a white checkerboard; else it's black.
    if (ix + iy + iz) % 2 == 0 { white } else { black }
}





/***** LIBRARY *****/
/// A texture that renders a checkerboard pattern using absolute coordinates.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct SpatialChecker {
    /// The size of the checkers in space.
    pub scale: f64,
    /// The colour of the "black" squares.
    pub black: Colour,
    /// The colour of  the "white" squares.
    pub white: Colour,
}

// Textures
impl Loadable for SpatialChecker {
    type Error = Infallible;

    #[inline(always)]
    fn load(&mut self) -> Result<(), Self::Error> {
        /* No op */
        Ok(())
    }
}
impl Textured for SpatialChecker {
    #[inline]
    fn value(&self, _uv: (f64, f64), p: Vec3) -> Colour { spatial_checker_value(self.scale, self.white, self.black, p) }
}



/// A texture that renders a checkerboard pattern using relative coordinates.
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
impl Loadable for Checker {
    type Error = Infallible;

    #[inline(always)]
    fn load(&mut self) -> Result<(), Self::Error> {
        /* No op */
        Ok(())
    }
}
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
