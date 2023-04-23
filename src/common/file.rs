//  FILE.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 11:42:51
//  Last edited:
//    23 Apr 2023, 13:17:29
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the [`File`] trait, which makes it more convenient to write
//!   file specifications.
// 

use std::any::type_name;
use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::errors::FileError;


/***** LIBRARY *****/
/// Defines the errors that may originate from the [`File`] trait.
#[derive(Debug)]
pub enum Error<E> {
    /// Failed to open a file.
    /// 
    /// The `what` should be the name of the thing we attempted to serialize.
    FileOpen{ what: &'static str, err: FileError },
    /// Failed to read a file.
    /// 
    /// The `what` should be the name of the thing we attempted to serialize.
    FileRead{ what: &'static str, err: FileError },
    /// Failed to parse a file.
    /// 
    /// The `what` should be the name of the thing we attempted to serialize.
    FileParse{ what: &'static str, path: PathBuf, err: Box<Self> },

    /// Failed to serialize ouselves using the backend serializer.
    /// 
    /// The `what` should be the name of the thing we attempted to serialize.
    Serialization{ what: &'static str, err: E },
}
impl<E: Display> Display for Error<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            FileOpen{ what, .. }        => write!(f, "Failed to open file to parse as `{what}`"),
            FileRead{ what, .. }        => write!(f, "Failed to read file to parse as `{what}`"),
            FileParse{ what, path, .. } => write!(f, "Failed to parse file '{}' as `{}`", path.display(), what),

            Serialization{ what, .. } => write!(f, "Failed to serialize `{what}`"),
        }
    }
}
impl<E: 'static + error::Error> error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            FileOpen{ err, .. }  => Some(err),
            FileRead{ err, .. }  => Some(err),
            FileParse{ err, .. } => Some(err),

            Serialization{ err, .. } => Some(err),
        }
    }
}

/// Defines the errors that may originate from the [`JsonFile`] trait.
pub type JsonError = Error<serde_json::Error>;



/// Defines convenience functions for reading/writing `serde` files to/from disk or other commonly used places.
pub trait File<'de>: Deserialize<'de> + Serialize {
    /// The associated error type for this File. This effectively determines the backend `serde` serializer/deserializer to use.
    type Err : error::Error;

    /// Attempts to parse this file from a string.
    /// 
    /// # Arguments
    /// - `raw`: The unparsed string from which we will attempt to read.
    /// 
    /// # Returns
    /// A new instance of `Self` with its contents loaded from the given string.
    /// 
    /// # Errors
    /// This function may error if we failed to parse `raw` as `Self`.
    fn from_string(raw: impl AsRef<str>) -> Result<Self, Error<Self::Err>> where Self: Sized;
    /// Attempts to parse this file from a reader.
    /// 
    /// # Arguments
    /// - `reader`: The [`Read`]-implementing reader to read from.
    /// 
    /// # Returns
    /// A new instance of `Self` with its contents loaded from the given reader.
    /// 
    /// # Errors
    /// This function may error if we failed to parse the contents of `reader` as `Self`.
    fn from_reader(reader: impl Read) -> Result<Self, Error<Self::Err>> where Self: Sized;
    /// Attempts to read this file from the given path on disk.
    /// 
    /// # Arguments
    /// - `path`: The path from which we will attempt to read.
    /// 
    /// # Returns
    /// A new instance of `Self` with its contents loaded from disk.
    /// 
    /// # Errors
    /// This function may error if we failed to load the file or parse it as `Self`.
    fn from_path(path: impl AsRef<Path>) -> Result<Self, Error<Self::Err>> where Self: Sized {
        let path: &Path = path.as_ref();

        // Attempt to open the file
        let mut handle: fs::File = match fs::File::open(path) {
            Ok(handle) => handle,
            Err(err)   => { return Err(Error::FileOpen{ what: type_name::<Self>(), err: FileError::Open { path: path.into(), err } }); },
        };

        // Read the file's contents into memory
        let mut raw: String = String::new();
        if let Err(err) = handle.read_to_string(&mut raw) { return Err(Error::FileRead{ what: type_name::<Self>(), err: FileError::Read{ path: path.into(), err } }); }

        // Parse using our own function
        match Self::from_string(&raw) {
            Ok(result) => Ok(result),
            Err(err)   => Err(Error::FileParse { what: type_name::<Self>(), path: path.into(), err: err }),
        }
    }

    /// Serializes ourselves to a string.
    /// 
    /// # Returns
    /// A new string containing our serialized contents.
    /// 
    /// # Errors
    /// This function may error if we failed to serialize ourselves.
    fn to_string(&self) -> Result<String, Error<Self::Err>>;
    /// Serializes ourselves and directory writes us to the given writer.
    /// 
    /// # Arguments
    /// - `writer`: The [`Write`]-implementing writer to write to.
    /// 
    /// # Errors
    /// This function may error if we failed to serialize ourselves or write to the writer.
    fn to_writer(&self, writer: impl Write) -> Result<(), Error<Self::Err>>;
    /// Writes this file to the given path on disk.
    /// 
    /// # Arguments
    /// - `path`: The path to which we will attempt to write.
    /// 
    /// # Errors
    /// This function fails if we fail to serialize or write the file. The latter may occur, for example, if we have insufficient permissions or any directory in the path does not exist.
    fn to_path(&self, path: impl AsRef<Path>) -> Result<(), Error<Self::Err>> {
        // Attempt to serialize ourselves to a file first
        let raw: String = self.to_string()?;

        // Open the file
        
    }
}



/// Marker trait that will automatically implement the [`File`] trait for a struct using `serde_json`.
pub trait JsonFile<'de>: Deserialize<'de> + Serialize {}
impl<'de, T: JsonFile<'de>> File<'de> for T {
    type Err = serde_json::Error;

    fn from_string(raw: impl AsRef<str>) -> Result<Self, Error<Self::Err>> where Self: Sized {
        
    }
    fn from_reader(reader: impl Read) -> Result<Self, Error<Self::Err>> where Self: Sized {
        
    }

    fn to_string(&self) -> Result<String, Error<Self::Err>> {
        
    }
    fn to_writer(&self, writer: impl Write) -> Result<(), Error<Self::Err>> {
        
    }
}
