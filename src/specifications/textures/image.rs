//  IMAGE.rs
//    by Lut99
//
//  Description:
//!   A texture based on Rust's [`image`]-library to load images.
//!
//!   In fact, we just extend the existing [`Image`] with the capability to
//!   be an image.
//

use super::Textured;
use crate::math::{Colour, Vec3};
pub use crate::render::image::Image;


/***** LIBRARY *****/
impl Textured for Image {
    #[inline]
    fn value(&self, uv: (f64, f64), _p: Vec3) -> Colour {
        // Scale the logical pixel coordinates to concrete coordinates
        // NOTE: Flip the Y-axis
        let (x, y): (u32, u32) = ((self.dims().0 as f64 * uv.0).round() as u32, (self.dims().1 as f64 * (1.0 - uv.1)).round() as u32);

        // Now sample that coordinate from ourselves
        self[(x, y)]
    }
}
