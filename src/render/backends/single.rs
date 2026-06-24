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

use super::super::image::Image;
use super::super::iter::Coords;
use super::super::iter::prelude::*;
use super::super::spec::RayRenderer;
use super::cpu::ray_colour;
use crate::hitlist::HitList;
use crate::math::camera::Camera;
use crate::math::colour::Colour;
use crate::render::iter::Samples;
use crate::specifications::features::Features;
use crate::specifications::scene::Environment;


/***** LIBRARY *****/
/// The SingleThreadRenderer renders rays straightforwardly on a single thread, no fuss.
#[derive(Debug)]
pub struct SingleThreadRenderer {
    /// The dimensions of the output images.
    dims:      (u32, u32),
    /// The renderer features to enable/disable.
    features:  Features,
    /// Whether to enable or disable the progress bar.
    show_prgs: bool,
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
        Self { dims: (dims.0.into(), dims.1.into()), features: features.into(), show_prgs }
    }
}
impl RayRenderer for SingleThreadRenderer {
    type Error = std::convert::Infallible;

    fn render_frame(&self, list: &HitList, env: &Environment) -> Result<crate::render::image::Image, Self::Error> {
        info!("Rendering scene ({} objects)...", list.len());

        // Create the image to render
        let mut image: Image = Image::new(self.dims);

        // Let us define the camera (static, for now)
        let camera: Camera = Camera::new(((image.width() as f64 / image.height() as f64) * 2.0, 2.0), 1.0);

        // Prepare the progressbar if desired
        let mut prgs: Option<(Instant, ProgressBar)> = if self.show_prgs {
            Some((
                Instant::now(),
                ProgressBar::new(image.dims().0 as u64 * image.dims().1 as u64 * self.features.n_samples as u64).with_style(
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
        for (i, coord) in Coords::new(self.dims).enumerate() {
            for (s, ray) in Samples::new(self.features.n_samples, [coord].into_iter()).cast(camera, self.dims).enumerate() {
                // Compute the colour of the Ray
                let colour: Colour = ray_colour(ray, list, self.features.max_depth, env);

                // Add the colour to the image.
                *image.at_mut(i) += colour;

                // Computed a ray!
                if let Some(prgs) = &mut prgs {
                    if prgs.0.elapsed().as_millis() >= 500 {
                        prgs.1.update(|state| state.set_pos((i * self.features.n_samples + s) as u64));
                        prgs.0 += std::time::Duration::from_millis(500);
                    }
                }
            }

            // Scale the colour back if we're at the end of this pixel
            let scale: f64 = 1.0 / self.features.n_samples as f64;
            if self.features.gamma_correction {
                *image.at_mut(i) = (*image.at(i) * scale).gamma().opaque().clamp();
            } else {
                *image.at_mut(i) = (*image.at(i) * scale).opaque().clamp();
            }
        }
        if let Some(prgs) = prgs {
            prgs.1.finish_with_message(format!(
                "Done (averaged {:.2} rays/s)",
                (image.dims().0 as u64 * image.dims().1 as u64 * self.features.n_samples as u64) as f64 / start.elapsed().as_secs() as f64
            ));
        }

        // Done
        Ok(image)
    }
}
