//  FRAME.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2023, 14:40:55
//  Last edited:
//    07 May 2023, 12:47:02
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the functionality to render a single frame based on a
//!   [`SceneFile`].
// 

use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};
use log::info;

use crate::math::colour::Colour;
use crate::math::vec3::{Vec3, Vector as _};
use crate::math::ray::Ray;
use crate::math::camera::Camera;
use crate::specifications::features::Features;
use crate::hitlist::HitList;

use super::image::Image;
use super::generator::RayGenerator;


/***** HELPER MACROS *****/
/***** HELPER FUNCTIONS *****/
/// Computes an Rgba quadruplet based on what the Ray hits.
/// 
/// # Arguments
/// - `ray`: The [`Ray`] who's colour to compute.
/// - `list`: A [`HitList`] that describes what to render.
/// - `depth`: The maximum number of times we bounce.
/// 
/// # Returns
/// A new [`Rgba`] struct that contains the matched colour.
fn ray_colour(ray: Ray, list: &HitList, depth: usize) -> Colour {
    // We stop if there is no more to bounce
    if depth == 0 { return Colour::new(0.0, 0.0, 0.0, 1.0); }

    // Try to find the object that hits closest
    match list.hit(ray, 0.0, f64::INFINITY) {
        Some((index, record)) => {
            // Scatter the ray now we've found it
            match list.scatter(ray, index, record) {
                // Return the recursive bounce of the returned ray
                (Some(scatter), attenuation) => attenuation * ray_colour(scatter, list, depth - 1),

                // We can simply return this colour
                (None, colour) => colour,
            }
        },

        None => {
            // Otherwise, return the sky colour
            let udir: Vec3 = ray.direct.unit();
            let t: f64 = 0.5 * (udir.y + 1.0);
            ((1.0 - t) * Colour::new(1.0, 1.0, 1.0, 0.0) + t * Colour::new(0.5, 0.7, 1.0, 0.0)).opaque()
        }
    }
}





/***** LIBRARY *****/
/// Implements the main rendering functionality.
/// 
/// # Arguments
/// - `image`: The [`Image`] to which we will render the scene.
/// - `list`: A [`HitList`] that describes what to render.
/// - `features`: A [`Features`] defining any extra features to enable/disable during rendering.
/// 
/// # Returns
/// A newly rendered image based on the given scene file.
pub fn render(image: &mut Image, list: &HitList, features: &Features) {
    info!("Rendering scene ({} objects)...", list.len());

    // Let us define the camera (static, for now)
    let camera: Camera = Camera::new(((image.width() as f64 / image.height() as f64) * 2.0, 2.0), 1.0);

    // Let us fire all the rays (we go top-to-bottom)
    let mut last_prgs_update: Instant = Instant::now();
    let start: Instant = Instant::now();
    let prgs: ProgressBar = ProgressBar::new(image.dims().0 as u64 * image.dims().1 as u64 * features.n_samples as u64).with_style(ProgressStyle::with_template(" Ray {human_pos}/{human_len} [{wide_bar}] {percent}% (ETA {eta}) ").unwrap_or_else(|err| panic!("Invalid template given to progress bar: {err}")).progress_chars("=> "));
    for ((s, x, y), ray) in RayGenerator::new(camera, image.dims(), features.n_samples).coords() {
        // Compute the colour of the Ray
        let colour : Colour = ray_colour(ray, list, features.max_depth);
        // println!("{colour}");

        // Add the colour to the image.
        image[(x, y)] += colour;

        // Scale the colour back if we're at the end of this pixel
        if s == features.n_samples - 1 {
            let scale: f64 = 1.0 / features.n_samples as f64;
            if features.gamma_correction {
                image[(x, y)] = (image[(x, y)] * scale).gamma().opaque().clamp();
            } else {
                image[(x, y)] = (image[(x, y)] * scale).opaque().clamp();
            }
        }

        // Computed a ray!
        if last_prgs_update.elapsed().as_millis() >= 500 {
            prgs.update(|state| state.set_pos(s as u64 + x as u64 * features.n_samples as u64 + y as u64 * features.n_samples as u64 * image.dims().0 as u64));
            last_prgs_update += std::time::Duration::from_millis(500);
        }
    }
    prgs.finish_with_message(format!("Done (averaged {:.2} rays/s)", (image.dims().0 as u64 * image.dims().1 as u64 * features.n_samples as u64) as f64 / start.elapsed().as_secs() as f64));

    // Done
}
