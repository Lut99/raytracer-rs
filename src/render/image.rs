//  IMAGE.rs
//    by Lut99
// 
//  Created:
//    29 Apr 2023, 09:39:10
//  Last edited:
//    29 Apr 2023, 10:12:25
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements the [`Image`] struct, which represents a single frame
//!   that we can render to.
// 

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::fs;
use std::ops::{Index, IndexMut};
use std::path::{Path, PathBuf};

use image::{ColorType, RgbaImage};

use crate::common::errors::DirError;
use crate::math::colour::Colour;


/***** ERRORS *****/
/// Defines the errors that may occur within the [`Image`] struct.
#[derive(Debug)]
pub enum Error {
    /// The parent directories did not exist.
    ParentNotFound{ path: PathBuf },
    /// Failed to fix the parent directories.
    FixDirs{ path: PathBuf, err: DirError },
    /// Failed to save an Image to disk.
    ToPath{ path: PathBuf, err: image::ImageError },
}
impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            ParentNotFound{ path } => write!(f, "Parent directory '{}' not found (re-run with '--fix-dirs' to create it)", path.display()),
            FixDirs{ path, .. }    => write!(f, "Failed to create parent directory for '{}'", path.display()),
            ToPath{ path, .. }     => write!(f, "Failed to write Image to '{}'", path.display()),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            ParentNotFound{ .. } => None,
            FixDirs{ err, .. }   => Some(err),
            ToPath{ err, .. }    => Some(err),
        }
    }
}





/***** LIBRARY *****/
/// The Image struct represents a single image buffer we can render to.
#[derive(Clone, Debug)]
pub struct Image {
    /// The actual image data.
    pixels : Vec<Colour>,
    /// The dimensions of the image.
    dims   : (usize, usize),
}

impl Image {
    /// Constructor for the Image that initializes it to be empty (all-zero).
    /// 
    /// # Arguments
    /// - `dims`: The dimensions for this image, as `(width, height)`.
    /// 
    /// # Returns
    /// A new instance of Self with only 0's in it.
    #[inline]
    pub fn new(dims: (impl Into<usize>, impl Into<usize>)) -> Self {
        let width  : usize = dims.0.into();
        let height : usize = dims.1.into();
        Self {
            pixels : vec![ Colour::zeroes(); width * height ],
            dims   : (width, height),
        }
    }



    /// Writes the Image to disk using the [`image`] library.
    /// 
    /// # Arguments
    /// - `path`: The path of the file to write to.
    /// - `fix_dirs`: Whether to fix missing directories when writing or not.
    /// 
    /// # Errors
    /// This function may error if we failed to create the file or if we failed to create directories (if `fix_dirs` is true).
    pub fn to_path(&self, path: impl AsRef<Path>, fix_dirs: bool) -> Result<(), Error> {
        let path: &Path = path.as_ref();

        // Fix the directories, if needed and told to
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if fix_dirs {
                    if let Err(err) = fs::create_dir_all(parent) { return Err(Error::FixDirs { path: path.into(), err: DirError::Create{ path: parent.into(), err } }); }
                } else{
                    return Err(Error::ParentNotFound { path: parent.into() });
                }
            }
        }

        // Cast our internal buffer to a [`Vec<u8>`]
        let mut buffer: RgbaImage = RgbaImage::new(self.dims.0 as u32, self.dims.1 as u32);
        for y in 0..self.dims.1 {
            for x in 0..self.dims.0 {
                buffer[(x as u32, (self.dims.1 - 1 - y) as u32)] = self.pixels[x + self.dims.0 * y].into();
            }
        }

        // Write it
        match image::save_buffer(path, &buffer, self.dims.0 as u32, self.dims.1 as u32, ColorType::Rgba8) {
            Ok(_)    => Ok(()),
            Err(err) => Err(Error::ToPath { path: path.into(), err }),
        }
    }



    /// Returns the number of pixels in this Image.
    #[inline]
    pub fn len(&self) -> usize { self.pixels.len() }
    /// Returns the dimensions of this Image.
    #[inline]
    pub fn dims(&self) -> (usize, usize) { self.dims }
    /// Returns the width of the image.
    #[inline]
    pub fn width(&self) -> usize { self.dims.0 }
    /// Returns the height of the image.
    #[inline]
    pub fn height(&self) -> usize { self.dims.1 }
}

impl Index<usize> for Image {
    type Output = [Colour];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        // Assert the index is within the number of rows before returning
        #[cfg(debug_assertions)]
        if index >= self.dims.1 { panic!("Row index {} is out-of-bounds for Image of size {}x{}", index, self.dims.0, self.dims.1); }
        &self.pixels[self.dims.0 * index..(self.dims.0 + 1) * index]
    }
}
impl IndexMut<usize> for Image {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        // Assert the index is within the number of rows before returning
        #[cfg(debug_assertions)]
        if index >= self.dims.1 { panic!("Row index {} is out-of-bounds for Image of size {}x{}", index, self.dims.0, self.dims.1); }
        &mut self.pixels[self.dims.0 * index..(self.dims.0 + 1) * index]
    }
}
impl Index<(usize, usize)> for Image {
    type Output = Colour;

    #[inline]
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        // Assert the index is within range before returning the individual pixel
        let index: usize = index.0 + self.dims.0 * index.1;
        #[cfg(debug_assertions)]
        if index >= self.pixels.len() { panic!("Index {} is out-of-bounds for Image of size {}x{}", index, self.dims.0, self.dims.1); }
        &self.pixels[index]
    }
}
impl IndexMut<(usize, usize)> for Image {
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        // Assert the index is within range before returning the individual pixel
        let index: usize = index.0 + self.dims.0 * index.1;
        #[cfg(debug_assertions)]
        if index >= self.pixels.len() { panic!("Index {} is out-of-bounds for Image of size {}x{}", index, self.dims.0, self.dims.1); }
        &mut self.pixels[index]
    }
}
