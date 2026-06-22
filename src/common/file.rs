//  FILE.rs
//    by Lut99
//
//  Created:
//    23 Apr 2023, 11:42:51
//  Last edited:
//    28 Apr 2023, 10:30:09
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the [`File`] trait, which makes it more convenient to write
//!   file specifications.
//

use std::path::PathBuf;

use thiserror::Error;


/***** LIBRARY *****/
/// Defines the errors that may originate from the [`File`] trait.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to parse {what} from a string as JSON")]
    FromString {
        what: &'static str,
        #[source]
        err:  serde_json::Error,
    },
    #[error("Failed to serialize {what} to a string as JSON")]
    ToString {
        what: &'static str,
        #[source]
        err:  serde_json::Error,
    },
    #[error("Failed to read file {path:?} to parse {what} from JSON")]
    FromPath {
        what: &'static str,
        path: PathBuf,
        #[source]
        err:  std::io::Error,
    },
    #[error("Failed to write serialized JSON from {what} to path {path:?}")]
    ToPath {
        what: &'static str,
        path: PathBuf,
        #[source]
        err:  std::io::Error,
    },
}



/// Macro that implements a function to deserialize from a TOML-string.
macro_rules! impl_toml_from_string {
    () => {
        /// Deserializes this config from a TOML-string.
        ///
        /// # Arguments
        /// - `raw`: The string to deserialize from.
        ///
        /// # Returns
        /// A newly instantiated Self from the info in the string.
        ///
        /// # Errors
        /// This function can error if `raw` isn't valid TOML or not a valid instance of this.
        pub fn from_string(raw: impl AsRef<str>) -> Result<Self, crate::common::file::Error> {
            match serde_json::from_str(raw.as_ref()) {
                Ok(res) => Ok(res),
                Err(err) => Err(crate::common::file::Error::FromString { what: ::std::any::type_name::<Self>(), err }),
            }
        }
    };
}
/// Macro that implements a function to serialize to a TOML-string.
macro_rules! impl_toml_to_string {
    () => {
        /// Serializes this config to a TOML-string.
        ///
        /// # Returns
        /// A string representing Self but as string.
        ///
        /// # Errors
        /// This function can error if self couldn't be serialized somehow.
        pub fn to_string(&self) -> Result<String, crate::common::file::Error> {
            match serde_json::to_string_pretty(self) {
                Ok(raw) => Ok(raw),
                Err(err) => Err(crate::common::file::Error::ToString { what: ::std::any::type_name::<Self>(), err }),
            }
        }
    };
}
/// Macro that implements a function to deserialize from a TOML-file.
macro_rules! impl_toml_from_path {
    () => {
        /// Deserializes this config from a TOML file.
        ///
        /// # Arguments
        /// - `path`: The path pointing to the file to deserialize from.
        ///
        /// # Returns
        /// A newly instantiated Self from the info in the file.
        ///
        /// # Errors
        /// This function can error if `path` couldn't be read; isn't valid TOML; or not a valid
        /// instance of this.
        pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self, crate::common::file::Error> {
            // Read the file to a string
            let path: &std::path::Path = path.as_ref();
            let s: String = std::fs::read_to_string(path).map_err(|err| crate::common::file::Error::FromPath {
                what: ::std::any::type_name::<Self>(),
                path: path.into(),
                err,
            })?;

            // Now parse as string
            Self::from_string(s)
        }
    };
}
/// Macro that implements a function to serialize to a TOML-file.
macro_rules! impl_toml_to_path {
    () => {
        /// Serializes this config to a TOML file.
        ///
        /// # Arguments
        /// - `path`: The path pointing to the file to serialize to.
        ///
        /// # Errors
        /// This function can error if `path` couldn't be written to; or if `self` isn't
        /// serializable as TOML.
        pub fn to_path(&self, path: impl AsRef<std::path::Path>) -> Result<(), crate::common::file::Error> {
            // Serialize ourselves to a string.
            let s: String = self.to_string()?;

            // Write the string to the file
            let path: &std::path::Path = path.as_ref();
            std::fs::write(path, &s).map_err(|err| crate::common::file::Error::ToPath {
                what: ::std::any::type_name::<Self>(),
                path: path.into(),
                err,
            })
        }
    };
}
pub(crate) use impl_toml_from_path;
pub(crate) use impl_toml_from_string;
pub(crate) use impl_toml_to_path;
pub(crate) use impl_toml_to_string;
