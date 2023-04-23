//  ERRORS.rs
//    by Lut99
// 
//  Created:
//    23 Apr 2023, 12:00:31
//  Last edited:
//    23 Apr 2023, 12:09:52
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines common error types used across modules.
// 

use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::path::PathBuf;


/***** AUXILLARY *****/
/// Formatter returned by [`PrettyError::stack()`].
pub struct PrettyErrorFormatter<'e> {
    /// The error to format.
    err : &'e dyn Error,
}
impl<'e> Debug for PrettyErrorFormatter<'e> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        // Write this error first (with nice formatting)
        if f.alternate() {
            writeln!(f, "{:#?}", self.err)?;
        } else {
            writeln!(f, "{:?}", self.err)?;
        }

        // Do the recursive thing for any source
        if let Some(src) = self.err.source() {
            writeln!(f)?;
            writeln!(f, "Caused by:")?;
            write!(f, "{}", Self{ err: src })?;
        }

        // Done
        Ok(())
    }
}
impl<'e> Display for PrettyErrorFormatter<'e> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        // Write this error first (with nice formatting)
        if f.alternate() {
            writeln!(f, "{:#}", self.err)?;
        } else {
            writeln!(f, "{}", self.err)?;
        }

        // Do the recursive thing for any source
        if let Some(src) = self.err.source() {
            writeln!(f)?;
            writeln!(f, "Caused by:")?;
            write!(f, "{}", Self{ err: src })?;
        }

        // Done
        Ok(())
    }
}

/// Helper trait that allows us to easily print an error stack.
pub trait PrettyError: Error {
    /// Returns a serializer for this error that prints it and its sources.
    /// 
    /// # Returns
    /// A new [`PrettyErrorFormatter`] that implements [`Display`].
    fn stack(&self) -> PrettyErrorFormatter { PrettyErrorFormatter{ err: &self } }
}
impl<T: Error> PrettyError for T {}





/***** LIBRARY *****/
/// Defines errors relating to file reading/writing.
#[derive(Debug)]
pub enum FileError {
    /// Failed to open a file.
    Open{ path: PathBuf, err: std::io::Error },
    /// Failed to read a file.
    Read{ path: PathBuf, err: std::io::Error },
}
impl Display for FileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use FileError::*;
        match self {
            Open{ path, err } => write!(f, "Failed to open file '{}': {}", path.display(), err),
            Read{ path, err } => write!(f, "Failed to read file '{}': {}", path.display(), err),
        }
    }
}
impl Error for FileError {}
