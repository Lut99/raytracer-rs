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
use std::thread::{self, ScopedJoinHandle};

use serde::{Deserialize, Serialize};

use crate::common::file::impl_file;
use crate::hitlist::HitList;
use crate::specifications::features::Features;

use super::super::spec::RayRenderer;
use super::super::image::Image;
use super::super::single::SingleThreadRenderer;


/***** ERRORS *****/
/// Defines errors that may occur when rendering multi-threaded.
#[derive(Debug)]
pub enum Error {
    /// Failed to get the number of available threads.
    AvailableThreads{ err: std::io::Error },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            AvailableThreads{ .. } => write!(f, "Failed to get available number of hardware threads"),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            AvailableThreads{ err } => Some(err),
        }
    }
}





/***** AUXILLARY *****/
/// Defines the configuration options for the multi-threaded renderer.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MultiThreadRendererConfig {
    /// Defines the number of threads to spawn. If omitted, uses the number reported by `std::thread::available_parallelism()`.
    n_threads : Option<NonZeroUsize>,
}

impl Default for MultiThreadRendererConfig {
    #[inline]
    fn default() -> Self {
        Self {
            n_threads : None,
        }
    }
}
impl_file!(MultiThreadRendererConfig, serde_yaml);





/***** LIBRARY *****/
/// The SingleThreadRenderer renders rays on multiple threads at once.
#[derive(Debug)]
pub struct MultiThreadRenderer {
    /// The dimensions of the output images.
    dims     : (u32, u32),
    /// The renderer features to enable/disable.
    features : Features,

    /// The number of threads to render with.
    n_threads : usize,
}

impl MultiThreadRenderer {
    /// Constructor for the MultiThreadRenderer.
    /// 
    /// # Arguments
    /// - `dims`: The dimensions of the output images of this renderer.
    /// - `features`: The features to enable in this renderer.
    /// - `config`: Any MultiThreadRenderer-specific config.
    /// 
    /// # Returns
    /// A new MultiThreadRenderer instance.
    /// 
    /// # Errors
    /// This function may error if the user left the number of threads unspecified and we failed to query the number ourselves.
    #[inline]
    pub fn new(dims: (impl Into<u32>, impl Into<u32>), features: impl Into<Features>, config: impl Into<MultiThreadRendererConfig>) -> Result<Self, Error> {
        // Resolve the number of threads first
        let n_threads: usize = match config.into().n_threads {
            Some(n_threads) => n_threads.into(),
            None => match std::thread::available_parallelism() {
                Ok(n_threads) => n_threads.into(),
                Err(err)      => { return Err(Error::AvailableThreads { err }); },
            },
        };

        // Done
        Ok(Self {
            dims     : (dims.0.into(), dims.1.into()),
            features : features.into(),
            n_threads,
        })
    }
}
impl RayRenderer for MultiThreadRenderer {
    type Error = std::convert::Infallible;

    fn render_frame(&self, list: &HitList) -> Result<crate::render::image::Image, Self::Error> {
        // Compute the (approximate) share for each thread
        let rows_per_thread: u32 = self.dims.1 / self.n_threads as u32;

        // Enter a thread scope to share the HitList
        let mut result: Image = Image::new(self.dims);
        thread::scope(|s| {
            // Spawn the required number of threads
            let mut handles: Vec<ScopedJoinHandle<Image>> = Vec::with_capacity(self.n_threads.into());
            for i in 0..self.n_threads {
                // Compute this thread's share
                let height: u32 = rows_per_thread + (i == self.n_threads - 1) as u32 * (self.dims.1 % self.n_threads as u32);

                // Spawn the thread
                let width    : u32      = self.dims.0;
                let features : Features = self.features.clone();
                handles.push(s.spawn(move || {
                    // Create a single-threaded renderer for this number of images
                    let renderer: SingleThreadRenderer = SingleThreadRenderer::new((width, height), features, false);
                    renderer.render_frame(list).unwrap()
                }));
            }

            // Now wait for the other threads to join, showing the progress bars in the meantime
            let mut done: usize = 0;
            while done < self.n_threads {
                // Poll the threads to see if they are ready
                done = 0;
                for handle in &handles {
                    done += handle.is_finished() as usize;
                }

                // Do sommat progressbar-y in the meantime
                /* TODO */
            }

            // Join all the threads
            for (i, handle) in handles.into_iter().enumerate() {
                // Get the result
                let image: Image = match handle.join() {
                    Ok(image) => image,
                    Err(_)    => { panic!("Thread {i} panicked"); },
                };

                // Move it into its location in the main image
                result.move_into(image, (0, i as u32 * rows_per_thread));
            }
        });

        // Done, return the image!
        Ok(result)
    }
}
