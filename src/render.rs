//  RENDER.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2023, 14:40:55
//  Last edited:
//    27 Apr 2023, 15:17:08
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the main render functionality.
// 

use image::{Rgba, RgbaImage};
use log::info;

use crate::specifications::scene::SceneFile;
use crate::math::colour::Colour;
use crate::math::vec3::{Vec3, Vector as _};
use crate::math::ray::Ray;


/***** HELPER FUNCTIONS *****/
/// Computes an Rgba quadruplet based on what the Ray hits.
/// 
/// # Arguments
/// - `ray`: The [`Ray`] who's colour to compute.
/// 
/// # Returns
/// A new [`Rgba`] struct that contains the matched colour.
fn ray_colour(ray: Ray) -> Colour {
    // Compute the colour using our math
    let udir: Vec3 = ray.direct.unit();
    let t: f64 = 0.5 * (udir.y + 1.0);
    (1.0 - t) * Colour::new(1.0, 1.0, 1.0, 0.0) + t * Colour::new(0.5, 0.7, 1.0, 0.0)
}





/***** LIBRARY *****/
/// Implements the main rendering functionality.
/// 
/// # Arguments
/// - `scene`: A [`SceneFile`] that describes what to render.
/// 
/// # Returns
/// A newly rendered image based on the given scene file.
pub fn handle(scene: SceneFile) -> RgbaImage {
    info!("Rendering scene...");

    

    // Done
    RgbaImage::new(0, 0)
}
