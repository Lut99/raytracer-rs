//  IMAGE.rs
//    by Lut99
//
//  Description:
//!   A texture based on Rust's [`image`]-library to load images.
//!
//!   In fact, we just extend the existing [`Image`] with the capability to
//!   be an image.
//

use std::path::PathBuf;

use image::ImageFormat;
use log::debug;
use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::Textured;
use super::checker::spatial_checker_value;
use crate::math::{Colour, Vec3};


/***** LIBRARY *****/
/// Wraps the render [`Image`](crate::render::image::Image) to make it a texture.
///
/// The difference is that the image can be loaded from disk, optionally.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Image {
    /// It's a loaded image.
    Loaded(crate::render::image::Image),
    /// It's a to-be-loaded image.
    ToLoad { path: PathBuf, format: Option<ImageFormat> },
}

// Interface
impl Loadable for Image {
    type Error = crate::render::image::Error;

    #[inline(always)]
    fn load(&mut self) -> Result<(), Self::Error> {
        // Get the image (or quit if we already loaded it)
        match self {
            Self::Loaded(_) => Ok(()),
            Self::ToLoad { path, format } => {
                let image = match format {
                    Some(fmt) => crate::render::image::Image::from_path(*fmt, &*path)?,
                    None => crate::render::image::Image::from_path_auto(&*path)?,
                };

                // Replace ourselves with the loaded image
                debug!("Succesfully loaded image {path:?}");
                *self = Self::Loaded(image);
                Ok(())
            },
        }
    }
}
impl Textured for Image {
    #[inline]
    fn value(&self, uv: (f64, f64), p: Vec3) -> Colour {
        match self {
            Self::Loaded(image) => {
                // Scale the logical pixel coordinates to concrete coordinates
                // NOTE: Flip the Y-axis
                let (x, y): (u32, u32) = ((image.dims().0 as f64 * uv.0).round() as u32, (image.dims().1 as f64 * (1.0 - uv.1)).round() as u32);

                // Now sample that coordinate from ourselves
                image[(x, y)]
            },

            // Else return the magenta colour pattern we know and love
            Self::ToLoad { .. } => spatial_checker_value(1.0, Colour::new(1.0, 0.0, 1.0, 1.0), Colour::new(0.0, 0.0, 0.0, 1.0), p),
        }
    }
}

// Conversion
impl From<crate::render::image::Image> for Image {
    #[inline]
    fn from(value: crate::render::image::Image) -> Self { Self::Loaded(value) }
}
impl TryFrom<Image> for crate::render::image::Image {
    type Error = crate::render::image::Error;

    #[inline]
    fn try_from(value: Image) -> Result<Self, Self::Error> {
        match value {
            Image::Loaded(image) => Ok(image),
            Image::ToLoad { path, format: Some(format) } => crate::render::image::Image::from_path(format, path),
            Image::ToLoad { path, format: None } => crate::render::image::Image::from_path_auto(path),
        }
    }
}
