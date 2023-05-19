//  RENDERER.rs
//    by Lut99
// 
//  Created:
//    19 May 2023, 11:35:51
//  Last edited:
//    19 May 2023, 12:12:22
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the single-threaded renderer.
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

use super::super::spec::RayRenderer;
use super::super::image::Image;
use super::super::generator::RayGenerator;


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
/// The SingleThreadRenderer renders rays straightforwardly on a single thread, no fuss.
#[derive(Debug)]
pub struct SingleThreadRenderer {
    /// The dimensions of the output images.
    dims      : (u32, u32),
    /// The renderer features to enable/disable.
    features  : Features,
    /// Whether to enable or disable the progress bar.
    show_prgs : bool,
}

impl SingleThreadRenderer {
    /// Constructor for the SingleThreadRenderer.
    /// 
    /// # Arguments
    /// - `dims`: The dimensions of the output images of this renderer.
    /// - `features`: The features to enable in this renderer.
    /// - `show_prgs`: Whether or not to show the progress as we're rendering.
    /// 
    /// # Returns
    /// A new SingleThreadRenderer instance.
    #[inline]
    pub fn new(dims: (impl Into<u32>, impl Into<u32>), features: impl Into<Features>, show_prgs: bool) -> Self {
        Self {
            dims     : (dims.0.into(), dims.1.into()),
            features : features.into(),
            show_prgs,
        }
    }
}
impl RayRenderer for SingleThreadRenderer {
    type Error = std::convert::Infallible;

    fn render_frame(&self, list: &HitList) -> Result<crate::render::image::Image, Self::Error> {
        info!("Rendering scene ({} objects)...", list.len());

        // Create the image to render
        let mut image: Image = Image::new(self.dims);

        // Let us define the camera (static, for now)
        let camera: Camera = Camera::new(((image.width() as f64 / image.height() as f64) * 2.0, 2.0), 1.0);

        // Prepare the progressbar if desired
        let mut prgs: Option<(Instant, ProgressBar)> = if self.show_prgs {
            Some((Instant::now(), ProgressBar::new(image.dims().0 as u64 * image.dims().1 as u64 * self.features.n_samples as u64).with_style(ProgressStyle::with_template(" Ray {human_pos}/{human_len} [{wide_bar}] {percent}% (ETA {eta}) ").unwrap_or_else(|err| panic!("Invalid template given to progress bar: {err}")).progress_chars("=> "))))
        } else {
            None
        };

        // Let us fire all the rays (we go top-to-bottom)
        let start: Instant = Instant::now();
        for ((s, x, y), ray) in RayGenerator::new(camera, image.dims(), self.features.n_samples).coords() {
            // Compute the colour of the Ray
            let colour : Colour = ray_colour(ray, list, self.features.max_depth);
            // println!("{colour}");

            // Add the colour to the image.
            image[(x, y)] += colour;

            // Scale the colour back if we're at the end of this pixel
            if s == self.features.n_samples - 1 {
                let scale: f64 = 1.0 / self.features.n_samples as f64;
                if self.features.gamma_correction {
                    image[(x, y)] = (image[(x, y)] * scale).gamma().opaque().clamp();
                } else {
                    image[(x, y)] = (image[(x, y)] * scale).opaque().clamp();
                }
            }

            // Computed a ray!
            if let Some(prgs) = &mut prgs {
                if prgs.0.elapsed().as_millis() >= 500 {
                    prgs.1.update(|state| state.set_pos(s as u64 + x as u64 * self.features.n_samples as u64 + y as u64 * self.features.n_samples as u64 * image.dims().0 as u64));
                    prgs.0 += std::time::Duration::from_millis(500);
                }
            }
        }
        if let Some(prgs) = prgs { prgs.1.finish_with_message(format!("Done (averaged {:.2} rays/s)", (image.dims().0 as u64 * image.dims().1 as u64 * self.features.n_samples as u64) as f64 / start.elapsed().as_secs() as f64)); }

        // Done
        Ok(image)
    }
}
