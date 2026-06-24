//  COMMON.rs
//    by Lut99
//
//  Description:
//!   Contains some common functions across CPU-based renderers.
//

use crate::hitlist::HitList;
use crate::math::{Colour, Ray, Vec3};
use crate::specifications::scene::Environment;


/***** LIBRARY *****/
/// Computes an Rgba quadruplet based on what the Ray hits.
///
/// # Arguments
/// - `ray`: The [`Ray`] who's colour to compute.
/// - `list`: A [`HitList`] that describes what to render.
/// - `depth`: The maximum number of times we bounce.
/// - `env`: An [`Environment`]-struct relating properties about the environment.
///
/// # Returns
/// A new [`Rgba`] struct that contains the matched colour.
pub fn ray_colour(ray: Ray, list: &HitList, depth: usize, env: &Environment) -> Colour {
    // We stop if there is no more to bounce
    if depth == 0 {
        return Colour::new(0.0, 0.0, 0.0, 1.0);
    }

    // Try to find the object that hits closest
    match list.hit(ray, 0.001, f64::INFINITY, env) {
        Some((index, record)) => {
            // Scatter the ray now we've found it
            match list.scatter(ray, index, record, env) {
                // Return the recursive bounce of the returned ray
                (Some(scatter), attenuation) => attenuation * ray_colour(scatter, list, depth - 1, env),

                // We can simply return this colour
                (None, colour) => colour,
            }
        },

        None => {
            // Otherwise, return the sky colour
            let udir: Vec3 = ray.direct.unit();
            let t: f64 = 0.5 * (udir.y + 1.0);
            ((1.0 - t) * Colour::new(1.0, 1.0, 1.0, 0.0) + t * Colour::new(0.5, 0.7, 1.0, 0.0)).opaque()
        },
    }
}
