//  SPEC.rs
//    by Lut99
// 
//  Created:
//    19 May 2023, 11:31:46
//  Last edited:
//    19 May 2023, 12:47:14
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines a few specifications of the various compute backends.
// 

use std::error::Error;
use std::fmt::Debug;

use clap::ValueEnum;
use enum_debug::EnumDebug;

use crate::hitlist::HitList;

use super::image::Image;


/***** LIBRARY *****/
/// Defines the main trait for rendering backends.
pub trait RayRenderer: Debug {
    /// Defines the errors that this renderer may throw.
    type Error: Error;

    /// Renders a single frame of the given dimensions.
    /// 
    /// # Arguments
    /// - `list`: The [`HitList`] that contains the scene to render.
    /// 
    /// # Returns
    /// A new [`Image`] struct that contains the rendered frame.
    /// 
    /// # Errors
    /// This function may error. This will typically be an error relating to the backend of the renderer, since the rendering process, mathmatically, does not error.
    fn render_frame(&self, list: &HitList) -> Result<Image, Self::Error>;
}



/// Defines the collection of all our renderers.
#[derive(Clone, Copy, Debug, EnumDebug, Eq, Hash, PartialEq, ValueEnum)]
pub enum RenderBackend {
    /// Renders rays single-threaded.
    #[clap(name = "single", alias = "single_threaded", alias = "single-threaded")]
    SingleThreaded,
    /// Renders rays multi-threaded.
    #[clap(name = "multi", alias = "multi_threaded", alias = "multi-threaded")]
    MultiThreaded,
}
