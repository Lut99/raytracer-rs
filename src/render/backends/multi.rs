//  RENDERER.rs
//    by Lut99
//
//  Created:
//    19 May 2023, 11:57:54
//  Last edited:
//    19 May 2023, 12:51:04
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements a multi-threaded renderer that re-uses the
//!   single-threaded renderer.
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::num::NonZeroUsize;
use std::thread::ScopedJoinHandle;
use std::time::Instant;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::info;
use serde::{Deserialize, Serialize};

use super::super::image::Image;
use super::super::iter::Coords;
use super::super::iter::prelude::*;
use super::super::spec::RayRenderer;
use super::cpu::ray_colour;
use crate::common::file::impl_file;
use crate::hitlist::HitList;
use crate::math::{Camera, Colour};
use crate::render::iter::Samples;
use crate::specifications::features::Features;


/***** ERRORS *****/
/// Defines errors that may occur when rendering multi-threaded.
#[derive(Debug)]
pub enum Error {
    /// Failed to get the number of available threads.
    AvailableThreads { err: std::io::Error },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            AvailableThreads { .. } => write!(f, "Failed to get available number of hardware threads"),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            AvailableThreads { err } => Some(err),
        }
    }
}





/***** AUXILLARY *****/
/// Defines the configuration options for the multi-threaded renderer.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MultiThreadRendererConfig {
    /// Defines the number of threads to spawn. If omitted, uses the number reported by `std::thread::available_parallelism()`.
    n_threads: Option<NonZeroUsize>,
}

impl Default for MultiThreadRendererConfig {
    #[inline]
    fn default() -> Self { Self { n_threads: None } }
}
impl_file!(MultiThreadRendererConfig, serde_yaml);





/***** LIBRARY *****/
/// The SingleThreadRenderer renders rays on multiple threads at once.
#[derive(Debug)]
pub struct MultiThreadRenderer {
    /// The dimensions of the output images.
    dims:      (u32, u32),
    /// The renderer features to enable/disable.
    features:  Features,
    /// Whether to enable or disable the progress bar.
    show_prgs: bool,

    /// The number of threads to render with.
    n_threads: usize,
}

impl MultiThreadRenderer {
    /// Constructor for the MultiThreadRenderer.
    ///
    /// # Arguments
    /// - `dims`: The dimensions of the output images of this renderer.
    /// - `features`: The features to enable in this renderer.
    /// - `show_prgs`: Whether or not to show the progress as we're rendering.
    /// - `config`: Any MultiThreadRenderer-specific config.
    ///
    /// # Returns
    /// A new MultiThreadRenderer instance.
    ///
    /// # Errors
    /// This function may error if the user left the number of threads unspecified and we failed to query the number ourselves.
    #[inline]
    pub fn new(
        dims: (impl Into<u32>, impl Into<u32>),
        features: impl Into<Features>,
        show_prgs: bool,
        config: impl Into<MultiThreadRendererConfig>,
    ) -> Result<Self, Error> {
        // Resolve the number of threads first
        let n_threads: usize = match config.into().n_threads {
            Some(n_threads) => n_threads.into(),
            None => match std::thread::available_parallelism() {
                Ok(n_threads) => n_threads.into(),
                Err(err) => {
                    return Err(Error::AvailableThreads { err });
                },
            },
        };

        // Done
        Ok(Self { dims: (dims.0.into(), dims.1.into()), features: features.into(), show_prgs, n_threads })
    }
}
impl RayRenderer for MultiThreadRenderer {
    type Error = std::convert::Infallible;

    fn render_frame(&self, list: &HitList) -> Result<crate::render::image::Image, Self::Error> {
        // // Compute the (approximate) share for each thread
        // let rows_per_thread: u32 = self.dims.1 / self.n_threads as u32;

        // // Enter a thread scope to share the HitList
        // let mut result: Image = Image::new(self.dims);
        // thread::scope(|s| {
        //     // Spawn the required number of threads
        //     let mut handles: Vec<ScopedJoinHandle<Image>> = Vec::with_capacity(self.n_threads.into());
        //     for i in 0..self.n_threads {
        //         // Compute this thread's share
        //         let height: u32 = rows_per_thread + (i == self.n_threads - 1) as u32 * (self.dims.1 % self.n_threads as u32);

        //         // Spawn the thread
        //         let width: u32 = self.dims.0;
        //         let features: Features = self.features.clone();
        //         handles.push(s.spawn(move || {
        //             // Create a single-threaded renderer for this number of images
        //             let renderer: SingleThreadRenderer = SingleThreadRenderer::new((width, height), features, false);
        //             renderer.render_frame(list).unwrap()
        //         }));
        //     }

        //     // Now wait for the other threads to join, showing the progress bars in the meantime
        //     let mut done: usize = 0;
        //     while done < self.n_threads {
        //         // Poll the threads to see if they are ready
        //         done = 0;
        //         for handle in &handles {
        //             done += handle.is_finished() as usize;
        //         }

        //         // Do sommat progressbar-y in the meantime
        //         /* TODO */
        //     }

        //     // Join all the threads
        //     for (i, handle) in handles.into_iter().enumerate() {
        //         // Get the result
        //         let image: Image = match handle.join() {
        //             Ok(image) => image,
        //             Err(_) => {
        //                 panic!("Thread {i} panicked");
        //             },
        //         };

        //         // Move it into its location in the main image
        //         result.move_into(image, (0, i as u32 * rows_per_thread));
        //     }
        // });

        // // Done, return the image!
        // Ok(result)

        info!("Rendering scene ({} objects)...", list.len());

        // Let us define the camera (static, for now)
        let dims: (u32, u32) = self.dims;
        let camera: Camera = Camera::new(((dims.0 as f64 / dims.1 as f64) * 2.0, 2.0), 1.0);

        // Prepare the progressbar if desired
        let prgs: Option<(Instant, MultiProgress)> = if self.show_prgs { Some((Instant::now(), MultiProgress::new())) } else { None };

        // Split the rays, do each generator on a separate thread
        let mut image = Image::new(dims);
        let start: Instant = Instant::now();
        std::thread::scope(|s| {
            // Get an image and split it into many parts
            let parts: Vec<&mut [Colour]> = image.distribute(self.n_threads);

            // Split one set of rays for every thread
            let handles: Vec<ScopedJoinHandle<()>> = Coords::new(dims)
                .distribute(self.n_threads)
                .zip(parts)
                .map(|(iter, part)| {
                    // Prepare a progress bar
                    let llen: usize = iter.len();
                    let mut prgs: Option<(Instant, ProgressBar)> = prgs.as_ref().map(|(inst, prgs)| {
                        (
                            *inst,
                            prgs.add(
                                ProgressBar::new(llen as u64 * self.features.n_samples as u64).with_style(
                                    ProgressStyle::with_template(" Ray {human_pos}/{human_len} [{wide_bar}] {percent}% (ETA {eta}) ")
                                        .unwrap_or_else(|err| panic!("Invalid template given to progress bar: {err}"))
                                        .progress_chars("=> "),
                                ),
                            ),
                        )
                    });

                    // Spawn a thread that does this iterator
                    s.spawn(move || {
                        // Iterate
                        for (li, coord) in iter.enumerate() {
                            // Run through the samples
                            for (s, ray) in Samples::new(self.features.n_samples, [coord].into_iter()).cast(camera, dims).enumerate() {
                                // Compute the colour of the Ray
                                let colour: Colour = ray_colour(ray, list, self.features.max_depth);

                                // Add the colour to the image.
                                part[li] += colour;

                                // Computed a ray!
                                if let Some(prgs) = &mut prgs {
                                    if prgs.0.elapsed().as_millis() >= 500 {
                                        prgs.1.update(|state| state.set_pos((li * self.features.n_samples + s) as u64));
                                        prgs.0 += std::time::Duration::from_millis(500);
                                    }
                                }
                            }

                            // Scale the colour back
                            let scale: f64 = 1.0 / self.features.n_samples as f64;
                            if self.features.gamma_correction {
                                part[li] = (part[li] * scale).gamma().opaque().clamp();
                            } else {
                                part[li] = (part[li] * scale).opaque().clamp();
                            }
                        }
                        if let Some(prgs) = prgs {
                            prgs.1.finish_with_message(format!(
                                "Done (averaged {:.2} rays/s)",
                                (dims.0 as u64 * dims.1 as u64 * self.features.n_samples as u64) as f64 / start.elapsed().as_secs() as f64
                            ));
                        }
                    })
                })
                .collect();

            // Await them
            for (i, handle) in handles.into_iter().enumerate() {
                if handle.join().is_err() {
                    panic!("Thread {i} panicked");
                }
            }
        });

        // Done
        Ok(image)
    }
}
