//  IMAGE.rs
//    by Lut99
//
//  Created:
//    29 Apr 2023, 09:39:10
//  Last edited:
//    19 May 2023, 12:28:52
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the [`Image`] struct, which represents a single frame
//!   that we can render to.
//

use std::fs;
use std::io::{BufRead, BufReader, Cursor, Seek};
use std::ops::{AddAssign, Index, IndexMut};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use image::codecs::png::PngEncoder;
use image::{ColorType, DynamicImage, GenericImageView, ImageFormat, Pixel, RgbaImage};
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::math::colour::Colour;


/***** ERRORS *****/
/// Defines the errors that may occur within the [`Image`] struct.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Parent directory {path:?} not found (re-run with '--fix-dirs' to create it)")]
    ParentNotFound { path: PathBuf },
    #[error("Failed to create parent directory for {path:?}")]
    FixDirs {
        path: PathBuf,
        #[source]
        err:  std::io::Error,
    },
    #[error("Failed to write Image to {err:?}")]
    ToPath {
        path: PathBuf,
        #[source]
        err:  image::ImageError,
    },
    #[error("Failed to read the Image from a buffer as {fmt:?}.")]
    Reader {
        fmt: ImageFormat,
        #[source]
        err: image::ImageError,
    },
    #[error("Failed to read from reader")]
    ReaderRead(#[source] std::io::Error),
    #[error("Failed to guess format of reader bytes")]
    GuessFormat(#[source] image::ImageError),
    #[error("Failed to open file {path:?}")]
    FileOpen {
        path: PathBuf,
        #[source]
        err:  std::io::Error,
    },
}





/***** HELPERS *****/
/// A struct that we actually serialize/deserialize
#[derive(Deserialize, Serialize)]
enum ImageInfo {
    Path(ImageInfoPath),
    Raw(ImageInfoRaw),
}

#[derive(Deserialize, Serialize)]
struct ImageInfoPath {
    /// The path where the file resides.
    path: PathBuf,
    /// Optional image format.
    #[serde(alias = "format")]
    fmt:  Option<ImageFormat>,
}

#[derive(Deserialize, Serialize)]
struct ImageInfoRaw {
    /// The raw data we're reading, encoded as Base64
    data: String,
}





/***** LIBRARY *****/
/// The Image struct represents a single image buffer we can render to.
#[derive(Clone, Debug)]
pub struct Image {
    /// The actual image data.
    pixels: Vec<Colour>,
    /// The dimensions of the image.
    dims:   (u32, u32),
}

// Constructors
impl Image {
    /// Constructor for the Image that initializes it to be empty (all-zero).
    ///
    /// # Arguments
    /// - `dims`: The dimensions for this image, as `(width, height)`.
    ///
    /// # Returns
    /// A new instance of Self with only 0's in it.
    #[inline]
    pub fn new(dims: (impl Into<u32>, impl Into<u32>)) -> Self {
        let width: u32 = dims.0.into();
        let height: u32 = dims.1.into();
        Self { pixels: vec![Colour::zeroes(); (width * height) as usize], dims: (width, height) }
    }

    /// Loads the image from a set of encoded bytes with the [`image`]-library.
    ///
    /// # Arguments
    /// - `fmt`: An [`ImageFormat`] declaring how to read the...
    /// - `bytes`: Raw bytes to load the image from.
    ///
    /// # Returns
    /// A new Image.
    ///
    /// # Errors
    /// This function errors if we couldn't parse the bytes as the given format.
    pub fn from_reader<R: BufRead + Seek>(fmt: ImageFormat, reader: R) -> Result<Self, Error> {
        // Read the bytes as an rgba image
        let image: DynamicImage = image::load(reader, fmt).map_err(|err| Error::Reader { fmt, err })?;

        // Convert to ourselves
        let dims: (u32, u32) = image.dimensions();
        let mut pixels = Vec::with_capacity((dims.0 * dims.1) as usize);
        for (_, _, pixel) in image.pixels() {
            let rgba = pixel.to_rgba();
            pixels.push(Colour::new(rgba[0] as f64 / 255.0, rgba[1] as f64 / 255.0, rgba[2] as f64 / 255.0, rgba[3] as f64 / 255.0));
        }
        Ok(Self { pixels, dims })
    }

    /// Loads the image from a set of encoded bytes with the [`image`]-library, attempting to guess
    /// the format automatically.
    ///
    /// # Arguments
    /// - `bytes`: Raw bytes to load the image from.
    ///
    /// # Returns
    /// A new Image.
    ///
    /// # Errors
    /// This function errors if we couldn't guess the format or if we couldn't parse the bytes as
    /// that format.
    pub fn from_reader_auto<R: BufRead + Seek>(mut reader: R) -> Result<Self, Error> {
        // Attempt to guess the format
        let mut buffer: [u8; 8192] = [0; 8192];
        let buffer_len: usize = reader.read(&mut buffer).map_err(Error::ReaderRead)?;
        let fmt: ImageFormat = image::guess_format(&buffer[..buffer_len]).map_err(Error::GuessFormat)?;
        Self::from_reader(fmt, reader)
    }

    /// Loads the image from a file referred to by a path.
    ///
    /// # Arguments
    /// - `fmt`: An [`ImageFormat`] declaring how to read the...
    /// - `path`: A file pointed to by a path-like.
    ///
    /// # Returns
    /// A new Image.
    ///
    /// # Errors
    /// This function errors if we couldn't read the file or parse the bytes as the given format.
    pub fn from_path(fmt: ImageFormat, path: impl AsRef<Path>) -> Result<Self, Error> {
        // Open the file
        let path: &Path = path.as_ref();
        let handle = fs::File::open(path).map_err(|err| Error::FileOpen { path: path.into(), err })?;
        Self::from_reader(fmt, BufReader::new(handle))
    }

    /// Loads the image from a file referred to by a path, guessing the file format from the bytes.
    ///
    /// # Arguments
    /// - `path`: A file pointed to by a path-like.
    ///
    /// # Returns
    /// A new Image.
    ///
    /// # Errors
    /// This function errors if we couldn't read the file, guess the format or parse the bytes as
    /// the given format.
    pub fn from_path_auto(path: impl AsRef<Path>) -> Result<Self, Error> {
        // Open the file
        let path: &Path = path.as_ref();
        let handle = fs::File::open(path).map_err(|err| Error::FileOpen { path: path.into(), err })?;
        Self::from_reader_auto(BufReader::new(handle))
    }
}

// Collection
impl Image {
    /// Gets a read-only reference to a pixel by linear coordinate.
    ///
    /// To use XY-coordinates, see the various [`IndexMut`]-implementations.
    ///
    /// # Arguments
    /// - `i`: The index to set.
    ///
    /// # Returns
    /// A reference to the [`Colour`] on that location.
    ///
    /// # Panics
    /// This function panics if `i` is out-of-bounds.
    #[inline]
    #[track_caller]
    pub fn at(&self, i: usize) -> &Colour {
        let pixels_len: usize = self.pixels.len();
        if let Some(pixel) = self.pixels.get(i) { pixel } else { panic!("Index {i} is out-of-bounds for image of {pixels_len} pixels") }
    }

    /// Gets a mutable reference to a pixel by linear coordinate.
    ///
    /// To use XY-coordinates, see the various [`IndexMut`]-implementations.
    ///
    /// # Arguments
    /// - `i`: The index to set.
    ///
    /// # Returns
    /// A mutable reference to the [`Colour`] on that location.
    ///
    /// # Panics
    /// This function panics if `i` is out-of-bounds.
    #[inline]
    #[track_caller]
    pub fn at_mut(&mut self, i: usize) -> &mut Colour {
        let pixels_len: usize = self.pixels.len();
        if let Some(pixel) = self.pixels.get_mut(i) { pixel } else { panic!("Index {i} is out-of-bounds for image of {pixels_len} pixels") }
    }

    /// Copies another image into this one.
    ///
    /// # Arguments
    /// - `other`: The other image to paste into those image.
    /// - `position`: The position in this image, given as an `(x, y)` pair.
    ///
    /// # Panics
    /// This function panics if the given image was too large for the position it was places, i.e., `position.0 + other.width() > self.width()` or `position.1 + other.height() > self.height()`.
    #[track_caller]
    pub fn move_into(&mut self, other: Image, position: (u32, u32)) {
        // Assert the image fits
        if position.0 + other.dims.0 > self.dims.0 || position.1 + other.dims.1 > self.dims.1 {
            panic!(
                "Cannot move given image of size {}x{} into this image of size {}x{} at position {}x{} ({},{} + {}x{} > {}x{})",
                other.dims.0,
                other.dims.1,
                self.dims.0,
                self.dims.1,
                position.0,
                position.1,
                position.0,
                position.1,
                other.dims.0,
                other.dims.1,
                self.dims.0,
                self.dims.1,
            );
        }

        // Perform the copy
        let start: usize = position.1 as usize * self.dims.0 as usize + position.0 as usize;
        self.pixels[start..start + other.pixels.len()].copy_from_slice(&other.pixels);
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
                    if let Err(err) = fs::create_dir_all(parent) {
                        return Err(Error::FixDirs { path: path.into(), err });
                    }
                } else {
                    return Err(Error::ParentNotFound { path: parent.into() });
                }
            }
        }

        // Cast our internal buffer to a [`Vec<u8>`]
        let mut buffer: RgbaImage = RgbaImage::new(self.dims.0 as u32, self.dims.1 as u32);
        for y in 0..self.dims.1 {
            for x in 0..self.dims.0 {
                buffer[(x as u32, (self.dims.1 - 1 - y) as u32)] = self.pixels[(x + self.dims.0 * y) as usize].into();
            }
        }

        // Write it
        match image::save_buffer(path, &buffer, self.dims.0 as u32, self.dims.1 as u32, ColorType::Rgba8) {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::ToPath { path: path.into(), err }),
        }
    }
}

// Collection stats
impl Image {
    /// Returns the number of pixels in this Image.
    #[inline]
    pub fn len(&self) -> usize { self.pixels.len() }
    /// Returns the dimensions of this Image.
    #[inline]
    pub fn dims(&self) -> (u32, u32) { self.dims }
    /// Returns the width of the image.
    #[inline]
    pub fn width(&self) -> u32 { self.dims.0 }
    /// Returns the height of the image.
    #[inline]
    pub fn height(&self) -> u32 { self.dims.1 }
}

// Ops
impl Index<u32> for Image {
    type Output = [Colour];

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        // Assert the index is within the number of rows before returning
        #[cfg(debug_assertions)]
        if index >= self.dims.1 {
            panic!("Row index {} is out-of-bounds for Image of size {}x{}", index, self.dims.0, self.dims.1);
        }
        &self.pixels[(self.dims.0 * index) as usize..((self.dims.0 + 1) * index) as usize]
    }
}
impl IndexMut<u32> for Image {
    #[inline]
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        // Assert the index is within the number of rows before returning
        #[cfg(debug_assertions)]
        if index >= self.dims.1 {
            panic!("Row index {} is out-of-bounds for Image of size {}x{}", index, self.dims.0, self.dims.1);
        }
        &mut self.pixels[(self.dims.0 * index) as usize..((self.dims.0 + 1) * index) as usize]
    }
}
impl Index<(u32, u32)> for Image {
    type Output = Colour;

    #[inline]
    fn index(&self, index: (u32, u32)) -> &Self::Output {
        // Assert the index is within range before returning the individual pixel
        let index: usize = (index.0 + self.dims.0 * index.1) as usize;
        #[cfg(debug_assertions)]
        if index >= self.pixels.len() {
            panic!("Index {} is out-of-bounds for Image of size {}x{}", index, self.dims.0, self.dims.1);
        }
        &self.pixels[index]
    }
}
impl IndexMut<(u32, u32)> for Image {
    #[inline]
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output {
        // Assert the index is within range before returning the individual pixel
        let index: usize = (index.0 + self.dims.0 * index.1) as usize;
        #[cfg(debug_assertions)]
        if index >= self.pixels.len() {
            panic!("Index {} is out-of-bounds for Image of size {}x{}", index, self.dims.0, self.dims.1);
        }
        &mut self.pixels[index]
    }
}

impl Index<usize> for Image {
    type Output = [Colour];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        // Assert the index is within the number of rows before returning
        #[cfg(debug_assertions)]
        if index >= self.dims.1 as usize {
            panic!("Row index {} is out-of-bounds for Image of size {}x{}", index, self.dims.0, self.dims.1);
        }
        &self.pixels[self.dims.0 as usize * index..(self.dims.0 as usize + 1) * index]
    }
}
impl IndexMut<usize> for Image {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        // Assert the index is within the number of rows before returning
        #[cfg(debug_assertions)]
        if index >= self.dims.1 as usize {
            panic!("Row index {} is out-of-bounds for Image of size {}x{}", index, self.dims.0, self.dims.1);
        }
        &mut self.pixels[self.dims.0 as usize * index..(self.dims.0 as usize + 1) * index]
    }
}
impl Index<(usize, usize)> for Image {
    type Output = Colour;

    #[inline]
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        // Assert the index is within range before returning the individual pixel
        let index: usize = index.0 + self.dims.0 as usize * index.1;
        #[cfg(debug_assertions)]
        if index >= self.pixels.len() {
            panic!("Index {} is out-of-bounds for Image of size {}x{}", index, self.dims.0, self.dims.1);
        }
        &self.pixels[index]
    }
}
impl IndexMut<(usize, usize)> for Image {
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        // Assert the index is within range before returning the individual pixel
        let index: usize = index.0 + self.dims.0 as usize * index.1;
        #[cfg(debug_assertions)]
        if index >= self.pixels.len() {
            panic!("Index {} is out-of-bounds for Image of size {}x{}", index, self.dims.0, self.dims.1);
        }
        &mut self.pixels[index]
    }
}
impl AddAssign for Image {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        if self.dims != rhs.dims {
            panic!("Cannot add images of different dimensions ({}x{} VS {}x{})", self.dims.0, self.dims.1, rhs.dims.0, rhs.dims.1)
        }
        assert_eq!(self.pixels.len(), rhs.pixels.len());

        // Add the vectors
        for (pixel, rhs) in self.pixels.iter_mut().zip(rhs.pixels.into_iter()) {
            *pixel += rhs;
        }
    }
}


impl Serialize for Image {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Cast our internal buffer to a [`Vec<u8>`]
        let mut buffer: RgbaImage = RgbaImage::new(self.dims.0 as u32, self.dims.1 as u32);
        for y in 0..self.dims.1 {
            for x in 0..self.dims.0 {
                buffer[(x as u32, (self.dims.1 - 1 - y) as u32)] = self.pixels[(x + self.dims.0 * y) as usize].into();
            }
        }
        let mut image: Vec<u8> = Vec::new();
        buffer.write_with_encoder(PngEncoder::new(&mut image)).map_err(serde::ser::Error::custom)?;

        // Convert that to base64 and serialize _that_
        let b64_image: String = base64::prelude::BASE64_STANDARD.encode(&image);
        ImageInfo::Raw(ImageInfoRaw { data: b64_image }).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Image {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the image info
        let info = ImageInfo::deserialize(deserializer)?;

        // Either load the path, or the raw data
        match info {
            ImageInfo::Path(p) => {
                // First, mod the path to ensure that, if it's relative, it's relative to this file
                // TODO: Fixthis, but how do we get the parent path?
                let path: PathBuf = if p.path.is_relative() { p.path } else { p.path };

                // Then consider the format
                match p.fmt {
                    Some(fmt) => Image::from_path(fmt, path).map_err(serde::de::Error::custom),
                    None => Image::from_path_auto(path).map_err(serde::de::Error::custom),
                }
            },

            ImageInfo::Raw(r) => {
                // Decode the Base64 data
                let image: Vec<u8> = base64::prelude::BASE64_STANDARD.decode(&r.data).map_err(serde::de::Error::custom)?;
                Image::from_reader_auto(Cursor::new(image)).map_err(serde::de::Error::custom)
            },
        }
    }
}

// Iteration
impl Image {
    /// Returns a read-only iterator over all [`Colour`]s in this Image.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Colour> { self.into_iter() }

    /// Returns a mutating iterator over all [`Colour`]s in this Image.
    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Colour> { self.into_iter() }
}
impl<'a> IntoIterator for &'a Image {
    type Item = &'a Colour;
    type IntoIter = std::slice::Iter<'a, Colour>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.pixels.iter() }
}
impl<'a> IntoIterator for &'a mut Image {
    type Item = &'a mut Colour;
    type IntoIter = std::slice::IterMut<'a, Colour>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.pixels.iter_mut() }
}
impl IntoIterator for Image {
    type Item = Colour;
    type IntoIter = std::vec::IntoIter<Colour>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.pixels.into_iter() }
}
