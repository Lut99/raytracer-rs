//  GENERATE.rs
//    by Lut99
// 
//  Created:
//    27 Apr 2023, 12:49:46
//  Last edited:
//    27 Apr 2023, 13:24:59
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements various subcommands in the `generate` subcommand.
// 

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::fs;
use std::path::{Path, PathBuf};

use console::style;
use image::{ColorType, EncodableLayout as _, RgbaImage};
use log::{debug, info};

use crate::common::errors::DirError;


/***** ERRORS *****/
/// Defines the errors for this crate.
#[derive(Debug)]
pub enum Error {
    /// Failed to fix missing directories.
    FixDirectories{ err: DirError },
    /// Parent directory not found, and '--fix-dirs' not given.
    MissingDirectories{ path: PathBuf },
    /// Failed to save an image.
    ImageSaveFailed{ path: PathBuf, err: image::ImageError },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            FixDirectories{ .. }        => write!(f, "Failed to create missing directories"),
            MissingDirectories{ path }  => write!(f, "Output directory '{}' not found (re-run with `--fix-dirs` to create it)", path.display()),
            ImageSaveFailed{ path, .. } => write!(f, "Failed to save generated image to '{}'", path.display()),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            FixDirectories{ err, .. }  => Some(err),
            MissingDirectories{ .. }   => None,
            ImageSaveFailed{ err, .. } => Some(err),
        }
    }
}





/***** LIBRARY *****/
/// Generates the test gradient image from the tutorial book.
/// 
/// From: <https://raytracing.github.io/books/RayTracingInOneWeekend.html>.
/// 
/// # Arguments
/// - `path`: The path to generate the image to.
/// - `dims`: The dimensions of the image.
/// - `fix_dirs`: Whether to fix missing directories or chicken out.
/// 
/// # Errors
/// This function may error if we failed to write the image or fix the missing directories.
pub fn gradient(path: impl AsRef<Path>, dims: (u32, u32), fix_dirs: bool) -> Result<(), Error> {
    let path: &Path = path.as_ref();
    info!("Generating gradient image to '{}' (fixing directories? {})...", path.display(), if fix_dirs { "yes" } else { "no" });

    // Create the image
    debug!("Generating image of {}x{} pixels...", dims.0, dims.1);
    let mut image: Vec<u8> = Vec::with_capacity(4 * (dims.0 * dims.1) as usize);
    for y in 0..dims.1 {
        let y: u32 = dims.1 - 1 - y;
        for x in 0..dims.0 {
            // Simply write the pixel values
            image.push((255.0 * (x as f64 / (dims.0 - 1) as f64)).round() as u8);
            image.push((255.0 * (y as f64 / (dims.1 - 1) as f64)).round() as u8);
            image.push((255.0 as f64 * 0.25).round() as u8);
            image.push(255);
        }
    }
    let image: RgbaImage = RgbaImage::from_vec(dims.0, dims.1, image).unwrap();

    // Fix the directory, if asked
    if let Some(parent) = path.parent() {
        debug!("Checking existance of directory '{}'", parent.display());
        if !parent.exists() {
            // Either crash or no
            if fix_dirs {
                if let Err(err) = fs::create_dir_all(parent) { return Err(Error::FixDirectories { err: DirError::Create { path: parent.into(), err } }); }
            } else {
                return Err(Error::MissingDirectories{ path: parent.into() });
            }
        }
    }

    // Now write it to file
    if let Err(err) = image::save_buffer(path, image.as_bytes(), dims.0, dims.1, ColorType::Rgba8) {
        return Err(Error::ImageSaveFailed{ path: path.into(), err });
    }

    // Done
    println!("Successfully {} image to {}", style("gradient image").bold().green(), style(path.display()).bold());
    Ok(())
}
