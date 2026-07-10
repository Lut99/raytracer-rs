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

use super::super::RayRenderer;
use super::super::image::Image;
use super::cpu::ray_colour;
use crate::hittree::HitTree;
use crate::math::camera::Camera;
use crate::math::colour::Colour;
use crate::specifications::features::Features;
use crate::specifications::scene::Environment;


/***** LIBRARY *****/
/// The SingleThreadRenderer renders rays straightforwardly on a single thread, no fuss.
#[derive(Debug)]
pub struct SingleThreadRenderer {
    /// The renderer features to enable/disable.
    features:  Features,
    /// Whether to enable or disable the progress bar.
    show_prgs: bool,
}

impl SingleThreadRenderer {
    /// Constructor for the SingleThreadRenderer.
    ///
    /// # Arguments
    /// - `features`: The features to enable in this renderer.
    /// - `show_prgs`: Whether or not to show the progress as we're rendering.
    ///
    /// # Returns
    /// A new SingleThreadRenderer instance.
    #[inline]
    pub fn new(features: impl Into<Features>, show_prgs: bool) -> Self { Self { features: features.into(), show_prgs } }
}
impl RayRenderer for SingleThreadRenderer {
    type Error = std::convert::Infallible;

    fn render_frame(&self, world: &HitTree, cam: &Camera, env: &Environment) -> Result<crate::render::image::Image, Self::Error> {
        info!("Rendering scene ({} objects)...", world.len());

        // Create the image to render
        let dims: (u32, u32) = cam.dims();
        let mut image: Image = Image::new(dims);

        // Prepare the progressbar if desired
        let mut prgs: Option<(Instant, ProgressBar)> = if self.show_prgs {
            Some((
                Instant::now(),
                ProgressBar::new(dims.0 as u64 * dims.1 as u64 * cam.n_samples()).with_style(
                    ProgressStyle::with_template(" Ray {human_pos}/{human_len} [{wide_bar}] {percent}% (ETA {eta}) ")
                        .unwrap_or_else(|err| panic!("Invalid template given to progress bar: {err}"))
                        .progress_chars("=> "),
                ),
            ))
        } else {
            None
        };

        // Let us fire all the rays (we go top-to-bottom)
        let start: Instant = Instant::now();
        for (i, (_, x, y, ray)) in cam.rays(0).enumerate() {
            // Compute the colour of the Ray
            let colour: Colour = ray_colour(ray, world, self.features.max_depth, env);

            // Add the colour to the image.
            image[(x, y)] += colour;

            // Computed a ray!
            if let Some(prgs) = &mut prgs {
                if prgs.0.elapsed().as_millis() >= 500 {
                    prgs.1.update(|state| state.set_pos(i as u64));
                    prgs.0 += std::time::Duration::from_millis(500);
                }
            }
        }

        // Fix the final pixel values
        let scale: f64 = 1.0 / cam.n_samples() as f64;
        for colour in image.iter_mut() {
            *colour *= scale;
            if self.features.gamma_correction {
                *colour = colour.gamma();
            }
            *colour = colour.opaque().clamp();
        }

        if let Some(prgs) = prgs {
            prgs.1.finish_with_message(format!(
                "Done (averaged {:.2} rays/s)",
                (dims.0 as u64 * dims.1 as u64 * cam.n_samples()) as f64 / start.elapsed().as_secs() as f64
            ));
        }

        // Done
        Ok(image)
    }
}
