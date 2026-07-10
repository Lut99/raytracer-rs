//  MOD.rs
//    by Lut99
//
//  Created:
//    29 Apr 2023, 09:36:21
//  Last edited:
//    19 May 2023, 11:32:02
//  Auto updated?
//    Yes
//
//  Description:
//!   The `render` module implements everything dealing with rendering.
//

// Declare submodules
pub mod backends;
pub mod image;

// Imports
use std::error::Error;
use std::fmt::Debug;

use clap::ValueEnum;

use crate::hittree::HitTree;
use crate::math::Camera;
use crate::render::image::Image;
use crate::specifications::scene::Environment;


/***** INTERFACE *****/
/// Defines the main trait for rendering backends.
pub trait RayRenderer: Debug {
    /// Defines the errors that this renderer may throw.
    type Error: Error;

    /// Renders a single frame of the given dimensions.
    ///
    /// # Arguments
    /// - `world`: The [`HitTree`] that contains the scene to render.
    ///
    /// # Returns
    /// A new [`Image`] struct that contains the rendered frame.
    ///
    /// # Errors
    /// This function may error. This will typically be an error relating to the backend of the renderer, since the rendering process, mathmatically, does not error.
    fn render_frame(&self, world: &HitTree, cam: &Camera, env: &Environment) -> Result<Image, Self::Error>;
}





/***** LIBRARY *****/
/// Defines the collection of all our renderers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ValueEnum)]
pub enum RenderBackend {
    /// Renders rays single-threaded.
    #[clap(name = "single", alias = "single_threaded", alias = "single-threaded")]
    SingleThreaded,
    /// Renders rays multi-threaded.
    #[clap(name = "multi", alias = "multi_threaded", alias = "multi-threaded")]
    MultiThreaded,
}
