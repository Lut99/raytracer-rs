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
use std::iter::Enumerate;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::thread::ScopedJoinHandle;
use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use super::super::image::Image;
use super::super::iter::prelude::*;
use super::super::iter::{Coords, Samples};
use super::super::spec::RayRenderer;
use super::cpu::ray_colour;
use crate::common::file::{impl_toml_from_path, impl_toml_from_string, impl_toml_to_path, impl_toml_to_string};
use crate::hitlist::HitList;
use crate::math::{Camera, Colour};
use crate::specifications::features::Features;
use crate::specifications::scene::Environment;


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
    /// Defines the workload size to send to each thread everytime they finished rendering the previous one.
    work_size: usize,
}

impl Default for MultiThreadRendererConfig {
    #[inline]
    fn default() -> Self { Self { n_threads: None, work_size: 64 } }
}
impl MultiThreadRendererConfig {
    impl_toml_from_string!();
    impl_toml_to_string!();
    impl_toml_from_path!();
    impl_toml_to_path!();
}





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
    /// The number of rays to send to each thread every time they need work.
    work_size: usize,
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
        let config = config.into();
        let n_threads: usize = match config.n_threads {
            Some(n_threads) => n_threads.into(),
            None => match std::thread::available_parallelism() {
                Ok(n_threads) => n_threads.into(),
                Err(err) => {
                    return Err(Error::AvailableThreads { err });
                },
            },
        };

        // Done
        Ok(Self { dims: (dims.0.into(), dims.1.into()), features: features.into(), show_prgs, n_threads, work_size: config.work_size })
    }
}
impl RayRenderer for MultiThreadRenderer {
    type Error = std::convert::Infallible;

    fn render_frame(&self, list: &HitList, env: &Environment) -> Result<crate::render::image::Image, Self::Error> {
        info!("Rendering scene ({} objects)...", list.len());

        // Let us define the camera (static, for now)
        let dims: (u32, u32) = self.dims;
        let camera: Camera = Camera::new(((dims.0 as f64 / dims.1 as f64) * 2.0, 2.0), 1.0);
        let scale: f64 = 1.0 / self.features.n_samples as f64;

        // Now have the threads each do chunk of rays, popping them off the main queue
        std::thread::scope(|s| {
            let start: Instant = Instant::now();

            // Define the main queue of rays & the progress bar
            let queue: Arc<Mutex<(Enumerate<Coords>, Option<(Instant, ProgressBar)>)>> = Arc::new(Mutex::new((
                Coords::new(dims).enumerate(),
                if self.show_prgs {
                    Some((
                        Instant::now(),
                        ProgressBar::new(dims.0 as u64 * dims.1 as u64 * self.features.n_samples as u64).with_style(
                            ProgressStyle::with_template(" Ray {human_pos}/{human_len} [{wide_bar}] {percent}% (ETA {eta}) ")
                                .unwrap_or_else(|err| panic!("Invalid template given to progress bar: {err}"))
                                .progress_chars("=> "),
                        ),
                    ))
                } else {
                    None
                },
            )));

            // Split one set of rays for every thread
            let handles: Vec<ScopedJoinHandle<Image>> = (0..self.n_threads)
                .map(|_| {
                    // Spawn a thread that does this iterator
                    let queue = queue.clone();
                    s.spawn(move || {
                        // Prepare this local thread's frame to render to
                        let mut count: u64 = 0;
                        let mut image = Image::new(dims);
                        let mut buf: Vec<(usize, (f64, f64))> = Vec::with_capacity(self.work_size);

                        // Keep popping work until all pixels are computed
                        loop {
                            // Pop a chunk of rays to render
                            {
                                let mut lock = queue.lock();
                                buf.extend((&mut lock.0).take(self.work_size));
                                if buf.is_empty() {
                                    // Done, nothing to render anymore
                                    break image;
                                }

                                // Update the progress bar with what we're going to do
                                if let Some(prgs) = &mut lock.1 {
                                    if prgs.0.elapsed().as_millis() >= 500 {
                                        prgs.1.inc(count);
                                        prgs.0 += std::time::Duration::from_millis(500);
                                        count = 0;
                                    }
                                }
                            }

                            // Iterate over the allocated rays to compute them
                            for (i, coord) in buf.drain(..) {
                                for ray in Samples::new(self.features.n_samples, [coord].into_iter()).cast(camera, dims) {
                                    // Compute the colour of the Ray
                                    let colour: Colour = ray_colour(ray, list, self.features.max_depth, env);

                                    // Add the colour to the image.
                                    *image.at_mut(i) += colour;

                                    // Done this ray
                                    count += 1;
                                }

                                // Scale the colour back
                                if self.features.gamma_correction {
                                    *image.at_mut(i) = (*image.at(i) * scale).gamma().opaque().clamp();
                                } else {
                                    *image.at_mut(i) = (*image.at(i) * scale).opaque().clamp();
                                }
                            }
                        }
                    })
                })
                .collect();

            // Await them
            let mut res: Option<Image> = None;
            for (i, handle) in handles.into_iter().enumerate() {
                let image: Image = match handle.join() {
                    Ok(image) => image,
                    Err(_) => panic!("Thread {i} panicked"),
                };
                match &mut res {
                    Some(res) => *res += image,
                    None => res = Some(image),
                }
            }

            // Complete the progress bar
            if let Some(prgs) = &queue.lock().1 {
                prgs.1.finish_with_message(format!(
                    "Done (averaged {:.2} rays/s)",
                    (dims.0 as u64 * dims.1 as u64 * self.features.n_samples as u64) as f64 / start.elapsed().as_secs() as f64
                ));
            }

            // Done
            match res {
                Some(image) => Ok(image),
                None => panic!("No thread completed computation"),
            }
        })
    }
}
