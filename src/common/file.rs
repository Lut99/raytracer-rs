//  FILE.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 11:42:51
//  Last edited:
//    27 Apr 2023, 12:15:43
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
    /// Failed to parse ourselves from a string.
    /// 
    /// The `what` should be the name of the thing we attempted to parse.
    StringParse{ what: &'static str, err: E },
    /// Failed to parse ourselves from a given reader.
    /// 
    /// The `what` should be the name of the thing we attempted to serialize, and `reader` should be the name of the reader's type.
    ReaderParse{ what: &'static str, reader: &'static str, err: E },
    /// Failed to open a file.
    /// 
    /// The `what` should be the name of the thing we attempted to parse.
    FileOpen{ what: &'static str, err: FileError },
    /// Failed to read a file.
    /// 
    /// The `what` should be the name of the thing we attempted to parse.
    FileRead{ what: &'static str, err: FileError },
    /// Failed to parse a file.
    /// 
    /// The `what` should be the name of the thing we attempted to parse.
    FileParse{ what: &'static str, path: PathBuf, err: Box<Self> },

    /// Failed to serialize ourselves to a string.
    /// 
    /// The `what` should be the name of the thing we attempted to serialize.
    StringSerialize{ what: &'static str, err: E },
    /// Failed to serialize ourselves to a given writer.
    /// 
    /// The `what` should be the name of the thing we attempted to serialize, and `writer` should be the name of the writer's type.
    WriterSerialize{ what: &'static str, writer: &'static str, err: E },
    /// Failed to serialize ouselves using the backend serializer.
    /// 
    /// The `what` should be the name of the thing we attempted to serialize.
    FileSerialize{ what: &'static str, path: PathBuf, err: Box<Self> },
    /// Failed to create a file.
    /// 
    /// The `what` should be the name of the thing we attempted to serialize.
    FileCreate{ what: &'static str, err: FileError },
    /// Failed to write to a file.
    /// 
    /// The `what` should be the name of the thing we attempted to serialize.
    FileWrite{ what: &'static str, err: FileError },
}
impl<E: Display> Display for Error<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            StringParse{ what, .. }         => write!(f, "Failed to parse `{what}`"),
            ReaderParse{ what, reader, .. } => write!(f, "Failed to parse `{what}` from reader `{reader}`"),
            FileOpen{ what, .. }            => write!(f, "Failed to open file to parse as `{what}`"),
            FileRead{ what, .. }            => write!(f, "Failed to read `{what}` from file"),
            FileParse{ what, path, .. }     => write!(f, "Failed to parse `{}` from file '{}'", what, path.display()),

            StringSerialize{ what, .. }         => write!(f, "Failed to serialize `{what}`"),
            WriterSerialize{ what, writer, .. } => write!(f, "Failed to serialize `{what}` to writer `{writer}`"),
            FileSerialize{ what, path, .. }     => write!(f, "Failed to serialize `{}` to file '{}'", what, path.display()),
            FileCreate{ what, .. }              => write!(f, "Failed to create file to write `{what}`"),
            FileWrite{ what, .. }               => write!(f, "Failed to write `{what}` to file"),
        }
    }
}
impl<E: 'static + error::Error> error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            StringParse{ err, .. } => Some(err),
            ReaderParse{ err, .. } => Some(err),
            FileOpen{ err, .. }    => Some(err),
            FileRead{ err, .. }    => Some(err),
            FileParse{ err, .. }   => Some(err),

            StringSerialize{ err, .. } => Some(err),
            WriterSerialize{ err, .. } => Some(err),
            FileSerialize{ err, .. }   => Some(err),
            FileCreate{ err, .. }      => Some(err),
            FileWrite{ err, .. }       => Some(err),
        }
    }
}

/// Defines the errors that may originate from the [`JsonFile`] trait.
pub type JsonError = Error<serde_json::Error>;

/// Defines the errors that may originate from the [`YamlFile`] trait.
pub type YamlError = Error<serde_yaml::Error>;



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
    fn from_reader<R: Read>(reader: R) -> Result<Self, Error<Self::Err>> where Self: Sized;
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
            Err(err)   => Err(Error::FileParse { what: type_name::<Self>(), path: path.into(), err: Box::new(err) }),
        }
    }

    /// Serializes ourselves to a string.
    /// 
    /// # Arguments
    /// - `pretty`: Whether to write in pretty mode or not. Only relevant if the backend supports this difference.
    /// 
    /// # Returns
    /// A new string containing our serialized contents.
    /// 
    /// # Errors
    /// This function may error if we failed to serialize ourselves.
    fn to_string(&self, pretty: bool) -> Result<String, Error<Self::Err>>;
    /// Serializes ourselves and directory writes us to the given writer.
    /// 
    /// # Arguments
    /// - `writer`: The [`Write`]-implementing writer to write to.
    /// - `pretty`: Whether to write in pretty mode or not. Only relevant if the backend supports this difference.
    /// 
    /// # Errors
    /// This function may error if we failed to serialize ourselves or write to the writer.
    fn to_writer<W: Write>(&self, writer: W, pretty: bool) -> Result<(), Error<Self::Err>>;
    /// Writes this file to the given path on disk.
    /// 
    /// # Arguments
    /// - `path`: The path to which we will attempt to write.
    /// - `pretty`: Whether to write in pretty mode or not. Only relevant if the backend supports this difference.
    /// 
    /// # Errors
    /// This function fails if we fail to serialize or write the file. The latter may occur, for example, if we have insufficient permissions or any directory in the path does not exist.
    fn to_path(&self, path: impl AsRef<Path>, pretty: bool) -> Result<(), Error<Self::Err>> {
        let path: &Path = path.as_ref();

        // Attempt to serialize ourselves to a file first
        let raw: String = match self.to_string(pretty) {
            Ok(raw)  => raw,
            Err(err) => { return Err(Error::FileSerialize{ what: type_name::<Self>(), path: path.into(), err: Box::new(err) }); },
        };

        // Open the file
        let mut handle: fs::File = match fs::File::create(path) {
            Ok(handle) => handle,
            Err(err)   => { return Err(Error::FileCreate{ what: type_name::<Self>(), err: FileError::Create{ path: path.into(), err } }); },
        };

        // Write to it
        match write!(handle, "{raw}") {
            Ok(_)    => Ok(()),
            Err(err) => Err(Error::FileWrite{ what: type_name::<Self>(), err: FileError::Write{ path: path.into(), err } }),
        }
    }
}



/// Marker trait that will automatically implement the [`File`] trait for a struct using `serde_json`.
pub trait JsonFile: for<'de> Deserialize<'de> + Serialize {}
impl<'de, T: JsonFile> File<'de> for T {
    type Err = serde_json::Error;

    fn from_string(raw: impl AsRef<str>) -> Result<Self, Error<Self::Err>> where Self: Sized {
        match serde_json::from_str(raw.as_ref()) {
            Ok(res)  => Ok(res),
            Err(err) => Err(Error::StringParse { what: type_name::<Self>(), err }),
        }
    }
    fn from_reader<R: Read>(reader: R) -> Result<Self, Error<Self::Err>> where Self: Sized {
        match serde_json::from_reader(reader) {
            Ok(res)  => Ok(res),
            Err(err) => Err(Error::ReaderParse { what: type_name::<Self>(), reader: type_name::<R>(), err }),
        }
    }

    fn to_string(&self, pretty: bool) -> Result<String, Error<Self::Err>> {
        if pretty {
            match serde_json::to_string_pretty(self) {
                Ok(raw)  => Ok(raw),
                Err(err) => Err(Error::StringSerialize{ what: type_name::<Self>(), err }),
            }
        } else {
            match serde_json::to_string(self) {
                Ok(raw)  => Ok(raw),
                Err(err) => Err(Error::StringSerialize{ what: type_name::<Self>(), err }),
            }
        }
    }
    fn to_writer<W: Write>(&self, writer: W, pretty: bool) -> Result<(), Error<Self::Err>> {
        if pretty {
            match serde_json::to_writer_pretty(writer, self) {
                Ok(_)    => Ok(()),
                Err(err) => Err(Error::WriterSerialize{ what: type_name::<Self>(), writer: type_name::<W>(), err }),
            }
        } else {
            match serde_json::to_writer(writer, self) {
                Ok(_)    => Ok(()),
                Err(err) => Err(Error::WriterSerialize{ what: type_name::<Self>(), writer: type_name::<W>(), err }),
            }
        }
    }
}

/// Macro that can implement [`File`] conveniently for us.
macro_rules! impl_file {
    ($s:ident, $backend:ident) => {
        impl<'de> crate::common::file::File<'de> for $s {
            type Err = $backend::Error;

            fn from_string(raw: impl AsRef<str>) -> Result<Self, crate::common::file::Error<Self::Err>> where Self: Sized {
                match $backend::from_str(raw.as_ref()) {
                    Ok(res)  => Ok(res),
                    Err(err) => Err(crate::common::file::Error::StringParse { what: ::std::any::type_name::<Self>(), err }),
                }
            }
            fn from_reader<R: ::std::io::Read>(reader: R) -> Result<Self, crate::common::file::Error<Self::Err>> where Self: Sized {
                match $backend::from_reader(reader) {
                    Ok(res)  => Ok(res),
                    Err(err) => Err(crate::common::file::Error::ReaderParse { what: ::std::any::type_name::<Self>(), reader: ::std::any::type_name::<R>(), err }),
                }
            }
        
            fn to_string(&self, _pretty: bool) -> Result<String, crate::common::file::Error<Self::Err>> {
                match $backend::to_string(self) {
                    Ok(raw)  => Ok(raw),
                    Err(err) => Err(crate::common::file::Error::StringSerialize{ what: ::std::any::type_name::<Self>(), err }),
                }
            }
            fn to_writer<W: ::std::io::Write>(&self, writer: W, _pretty: bool) -> Result<(), crate::common::file::Error<Self::Err>> {
                match $backend::to_writer(writer, self) {
                    Ok(_)    => Ok(()),
                    Err(err) => Err(crate::common::file::Error::WriterSerialize{ what: ::std::any::type_name::<Self>(), writer: ::std::any::type_name::<W>(), err }),
                }
            }
        }
    };

    ($s:ident, serde_json) => {
        impl<'de> File<'de> for $s {
            type Err = serde_json::Error;

            fn from_string(raw: impl AsRef<str>) -> Result<Self, Error<Self::Err>> where Self: Sized {
                match serde_json::from_str(raw.as_ref()) {
                    Ok(res)  => Ok(res),
                    Err(err) => Err(Error::StringParse { what: type_name::<Self>(), err }),
                }
            }
            fn from_reader<R: Read>(reader: R) -> Result<Self, Error<Self::Err>> where Self: Sized {
                match serde_json::from_reader(reader) {
                    Ok(res)  => Ok(res),
                    Err(err) => Err(Error::ReaderParse { what: type_name::<Self>(), reader: type_name::<R>(), err }),
                }
            }
        
            fn to_string(&self, pretty: bool) -> Result<String, Error<Self::Err>> {
                if pretty {
                    match serde_json::to_string_pretty(self) {
                        Ok(raw)  => Ok(raw),
                        Err(err) => Err(Error::StringSerialize{ what: type_name::<Self>(), err }),
                    }
                } else {
                    match serde_json::to_string(self) {
                        Ok(raw)  => Ok(raw),
                        Err(err) => Err(Error::StringSerialize{ what: type_name::<Self>(), err }),
                    }
                }
            }
            fn to_writer<W: Write>(&self, writer: W, pretty: bool) -> Result<(), Error<Self::Err>> {
                if pretty {
                    match serde_json::to_writer_pretty(writer, self) {
                        Ok(_)    => Ok(()),
                        Err(err) => Err(Error::WriterSerialize{ what: type_name::<Self>(), writer: type_name::<W>(), err }),
                    }
                } else {
                    match serde_json::to_writer(writer, self) {
                        Ok(_)    => Ok(()),
                        Err(err) => Err(Error::WriterSerialize{ what: type_name::<Self>(), writer: type_name::<W>(), err }),
                    }
                }
            }
        }
    };
}
pub(crate) use impl_file;
