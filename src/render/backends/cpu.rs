//  COMMON.rs
//    by Lut99
//
//  Description:
//!   Contains some common functions across CPU-based renderers.
//

use crate::hitlist::HitList;
use crate::math::{Colour, Ray, Vec3};
use crate::specifications::objects::pdf::PDF;
use crate::specifications::scene::{Background, Environment};


/***** LIBRARY *****/
/// Computes an Rgba quadruplet based on what the Ray hits.
///
/// # Arguments
/// - `ray`: The [`Ray`] who's colour to compute.
/// - `world`: A [`HitTree`] that describes what to render.
/// - `depth`: The maximum number of times we bounce.
/// - `env`: An [`Environment`]-struct relating properties about the environment.
///
/// # Returns
/// A new [`Rgba`] struct that contains the matched colour.
pub fn ray_colour(ray: Ray, world: &HitList, depth: usize, env: &Environment) -> Colour {
    // We stop if there is no more to bounce
    if depth == 0 {
        return Colour::BLACK;
    }

    // Try to find the object that hits closest
    match world.hit_full(ray, 0.001, f64::INFINITY, env) {
        Some(record) => {
            // Compute if the material emits anything
            let colour_from_emission = record.emitted();

            // Scatter the ray now we've found it
            match record.scatter(ray, env) {
                // Return the recursive bounce of the returned ray + whatever we ourselves emit
                (Some(mut scatter), attenuation, pdf_value) => {
                    // Update: _if_ we hit, then compensate the found colour value for the bias
                    // introduced by the scattering PDF

                    // Compute the PDF value, which averages the found one with the lights one
                    let lights_pdf = world.light_pdf();
                    let pdf_value = 0.5 * pdf_value + 0.5 * lights_pdf.value(scatter, env);

                    // Compute the new scatter, which is one of the two rays, randomly
                    if fastrand::f64() < 0.5 {
                        scatter = Ray::with_time(record.data.hit, lights_pdf.sample(ray.time, record.data.hit).unit(), ray.time);
                    }
                    let scattering_pdf = record.pdf(ray, env, scatter);

                    // Finally, use them in the result
                    let sample_colour = ray_colour(scatter, world, depth - 1, env);
                    println!("{:?}", colour_from_emission + ((attenuation * scattering_pdf * sample_colour) / pdf_value));
                    colour_from_emission + ((attenuation * scattering_pdf * sample_colour) / pdf_value)
                },

                // We can simply return the emitted colour
                (None, colour, _) => colour_from_emission + colour,
            }
        },

        // Otherwise, return the background colour
        None => match env.background {
            Background::IlluminatedSky => {
                // Skybox of old
                let udir: Vec3 = ray.direct.unit();
                let t: f64 = 0.5 * (udir.y + 1.0);
                ((1.0 - t) * Colour::new(1.0, 1.0, 1.0, 0.0) + t * Colour::new(0.5, 0.7, 1.0, 0.0)).opaque()
            },

            Background::Colour(colour) => colour,
            Background::None => Colour::BLACK,
        },
    }
}
