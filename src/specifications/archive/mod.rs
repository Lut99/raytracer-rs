//  FILE.rs
//    by Lut99
//
//  Description:
//!   Defines a file format for embedding a scene file with all of its dependencies in a single
//!   archive.
//!
//!   This archive can be separately compressed.
//

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use log::debug;
use thiserror::Error;


/***** CONSTANTS *****/
/// Magic bytes for the Archive.
pub const MAGIC_BYTES: [u8; 4] = [0x15, 0x11, 0x19, 0x99];





/***** ERRORS *****/
#[derive(Debug, Error)]
#[error("Unknown file table entry type byte 0x{byte:02x}")]
pub struct FileTableEntryTypeError {
    byte: u8,
}



#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to open file {path:?}")]
    FileOpen {
        path: PathBuf,
        #[source]
        err:  std::io::Error,
    },
    #[error("{what}:{pos}: Expected a single byte with filetable entry type")]
    FileTableEntryTypeNotFound { what: String, pos: u64 },
    #[error("{what}:{pos}: Expected eight bytes of file address")]
    FileTableFileAddressNotFound { what: String, pos: u64 },
    #[error("{what}:4: Expected eight bytes with file count after magic bytes")]
    FileTableFileCountNotFound { what: String },
    #[error("{what}:4: File count is too high for your machine (your machine as address of {got} bytes)")]
    FileTableFileCountOverflow { what: String, got: usize },
    #[error("{what}:{pos}: Expected {expected} bytes of filename")]
    FileTableFilenameNotFound { what: String, pos: u64, expected: usize },
    #[error("{what}:{pos}: Filename is not valid UTF-8")]
    FileTableFilenameNotUtf8 {
        what: String,
        pos:  u64,
        #[source]
        err:  std::string::FromUtf8Error,
    },
    #[error("{what}:{pos}: Expected four bytes with filename length")]
    FileTableFilenameLenNotFound { what: String, pos: u64 },
    #[error("{what}:{pos}: Expected eight bytes of file length")]
    FileTableFileLenNotFound { what: String, pos: u64 },
    #[error("{what}:{pos}: Expected eight bytes of jump address")]
    FiletableJumpAddressNotFound { what: String, pos: u64 },
    #[error("{what}:0: Likely not an Archive (expected it to start with magic bytes)")]
    MagicBytesNotFound { what: String },
    #[error("Failed to read {what:?}")]
    Read {
        what: String,
        #[source]
        err:  std::io::Error,
    },
    #[error("Failed to seek in {what:?}")]
    Seek {
        what: String,
        #[source]
        err:  std::io::Error,
    },
    #[error("{what}:{pos}: {err}")]
    UnknownFileTableEntryType { what: String, pos: u64, err: FileTableEntryTypeError },
}





/***** ITERATORS *****/
/// Yielded by the iterators.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FileInfo<'a> {
    /// The name of the file.
    pub name: &'a str,
    /// The address of the file in the archive.
    pub addr: u64,
    /// The length of the file.
    pub len:  u64,
}



/// Iterate-by-reference iterator for the [`Archive`].
#[derive(Clone, Debug)]
pub struct Iter<'a>(std::collections::hash_map::Iter<'a, String, FileLoc>);
impl<'a> Iterator for Iter<'a> {
    type Item = FileInfo<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> { self.0.next().map(|(name, loc)| FileInfo { name, addr: loc.addr, len: loc.len }) }
}





/***** HELPERS *****/
/// Defines info we need to know about a file's location in the archive.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct FileLoc {
    addr: u64,
    len:  u64,
}





/***** SPEC *****/
/// Define file table entry node types.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum FileTableEntryType {
    /// It's a filename.
    Filename,
    /// It's a jump to another place in the file.
    Jump,
}
impl TryFrom<u8> for FileTableEntryType {
    type Error = FileTableEntryTypeError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Filename),
            1 => Ok(Self::Jump),
            byte => Err(FileTableEntryTypeError { byte }),
        }
    }
}
impl From<FileTableEntryType> for u8 {
    #[inline]
    fn from(value: FileTableEntryType) -> Self {
        match value {
            FileTableEntryType::Filename => 0,
            FileTableEntryType::Jump => 1,
        }
    }
}





/***** LIBRARY *****/
/// Defines a reader for the archive.
pub struct Archive<R> {
    // Source
    /// The handle we're reading from.
    reader: R,
    /// Source of the handle, for debugging.
    what:   String,

    // In-memory
    /// The filetable we've loaded that maps entries to where to find them.
    filetable: HashMap<String, FileLoc>,
}

// Constructors
impl<R: Read + Seek> Archive<R> {
    /// Constructor for the Archive that consumes a given reader.
    ///
    /// Note that the reader must be [`Seek`]able in addition to just [`Read`]able. It will always
    /// read from the start of the reader.
    ///
    /// # Arguments
    /// - `reader`: The [`Seek`]able [`Read`]er to read the archive from.
    /// - `what`: A debug description of what we're reading; e.g., a file path or a name of the
    ///   buffer.
    ///
    /// # Returns
    /// A new Archive that can be used to open and load files.
    ///
    /// # Errors
    /// This function can error if we failed to read from the `reader` or if the `reader` was not a
    /// valid Archive.
    pub fn open(mut reader: R, what: String) -> Result<Self, Error> {
        if let Err(err) = reader.seek(SeekFrom::Start(0)) {
            return Err(Error::Seek { what, err });
        }
        let mut fourbytes: [u8; 4] = [0; 4];
        let mut eightbytes: [u8; 8] = [0; 8];

        // First, expect the magic bytes
        if reader.read(&mut fourbytes).map_err(|err| Error::Read { what: what.clone(), err })? < 4 {
            return Err(Error::MagicBytesNotFound { what });
        }
        if fourbytes != MAGIC_BYTES {
            return Err(Error::MagicBytesNotFound { what });
        }

        // Then read how many files there are
        if reader.read(&mut eightbytes).map_err(|err| Error::Read { what: what.clone(), err })? < 8 {
            return Err(Error::FileTableFileCountNotFound { what });
        }
        let filecount: u64 = u64::from_le_bytes(eightbytes);
        let filecount: usize = if filecount <= usize::MAX as u64 {
            filecount as usize
        } else {
            return Err(Error::FileTableFileCountOverflow { what, got: std::mem::size_of::<usize>() });
        };

        // Read the filetable...
        let mut filetable: HashMap<String, FileLoc> = HashMap::with_capacity(filecount);
        let mut pos: u64 = 12;
        while filetable.len() < filecount {
            // Read entry type byte
            let mut entry: u8 = 0;
            if reader.read(std::slice::from_mut(&mut entry)).map_err(|err| Error::Read { what: what.clone(), err })? < 1 {
                return Err(Error::FileTableEntryTypeNotFound { what, pos });
            }
            let entry = FileTableEntryType::try_from(entry).map_err(|err| Error::UnknownFileTableEntryType { what: what.clone(), pos, err })?;
            pos += 1;

            // Match the entry type
            match entry {
                FileTableEntryType::Filename => {
                    // Read the four bytes of file name length
                    if reader.read(&mut fourbytes).map_err(|err| Error::Read { what: what.clone(), err })? < 4 {
                        return Err(Error::FileTableFilenameLenNotFound { what, pos });
                    }
                    let filename_len: usize = u32::from_le_bytes(fourbytes) as usize;
                    pos += 4;

                    // Then read those many bytes as filename length
                    let mut filename: Vec<u8> = vec![0; filename_len];
                    if reader.read(&mut filename).map_err(|err| Error::Read { what: what.clone(), err })? < filename_len {
                        return Err(Error::FileTableFilenameNotFound { what, pos, expected: filename_len });
                    }
                    let filename: String =
                        String::from_utf8(filename).map_err(|err| Error::FileTableFilenameNotUtf8 { what: what.clone(), pos, err })?;
                    pos += filename_len as u64;

                    // Read the address of where to find the file
                    if reader.read(&mut eightbytes).map_err(|err| Error::Read { what: what.clone(), err })? < 8 {
                        return Err(Error::FileTableFileAddressNotFound { what, pos });
                    }
                    let fileaddr: u64 = u64::from_le_bytes(eightbytes);
                    pos += 8;

                    // Finally the length of the file
                    if reader.read(&mut eightbytes).map_err(|err| Error::Read { what: what.clone(), err })? < 8 {
                        return Err(Error::FileTableFileLenNotFound { what, pos });
                    }
                    let filelen: u64 = u64::from_le_bytes(eightbytes);
                    pos += 8;

                    // Insert it
                    filetable.insert(filename, FileLoc { addr: fileaddr, len: filelen });
                },

                FileTableEntryType::Jump => {
                    // Read the next address' entry
                    if reader.read(&mut eightbytes).map_err(|err| Error::Read { what: what.clone(), err })? < 8 {
                        return Err(Error::FiletableJumpAddressNotFound { what, pos });
                    }
                    let jumpaddr: u64 = u64::from_le_bytes(eightbytes);
                    reader.seek(SeekFrom::Start(jumpaddr)).map_err(|err| Error::Seek { what: what.clone(), err })?;
                    pos = jumpaddr;
                },
            }
        }

        // Build self
        Ok(Self { reader, what, filetable })
    }
}
impl Archive<File> {
    /// Constructor for the Archive that opens it from a path.
    ///
    /// The source is automatically set to the file's path.
    ///
    /// # Arguments
    /// - `path`: The path to the file to open.
    ///
    /// # Returns
    /// A new Archive that can be used to open and load files in the given file.
    ///
    /// # Errors
    /// This function errors if it failed to find the file or if [`Archive::new()`] would fail.
    #[inline]
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        fn _from_path(path: &Path) -> Result<Archive<File>, Error> {
            // Open the file normally
            debug!("Opening file {path:?} as an Archive file");
            let handle = File::open(path).map_err(|err| Error::FileOpen { path: path.into(), err })?;

            // Open as a reader
            Archive::open(handle, path.to_string_lossy().into())
        }
        _from_path(path.as_ref())
    }
}

// Properties
impl<R> Archive<R> {
    /// Returns an iterator over file metadata.
    ///
    /// # Returns
    /// An [`Iter`]ator yielding [`FileInfo`] structs describing the file.
    #[inline]
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter { self.into_iter() }



    /// Returns the number of files in this archive.
    #[inline]
    pub fn filecount(&self) -> usize { self.filetable.len() }

    /// Returns whether there are any files in this archive.
    #[inline]
    pub fn is_empty(&self) -> bool { self.filecount() == 0 }
}

// Iterators
impl<'a, R> IntoIterator for &'a Archive<R> {
    type Item = FileInfo<'a>;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { Iter(self.filetable.iter()) }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_archive_open() {
        let empty = b"\x15\x11\x19\x99\x00\x00\x00\x00\x00\x00\x00\x00";
        let archive = Archive::open(Cursor::new(empty.as_slice()), "test1".into()).unwrap();
        assert!(archive.is_empty());

        let one = b"\x15\x11\x19\x99\x01\x00\x00\x00\x00\x00\x00\x00\x00\x0d\x00\x00\x00Hello, world!\x2e\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let archive = Archive::open(Cursor::new(one.as_slice()), "test2".into()).unwrap();
        assert_eq!(archive.filecount(), 1);
        assert_eq!(archive.iter().collect::<Vec<FileInfo>>(), vec![FileInfo { name: "Hello, world!", addr: 46, len: 0 }]);

        let two = b"\x15\x11\x19\x99\x02\x00\x00\x00\x00\x00\x00\x00\x00\x0d\x00\x00\x00Hello, world!\x2e\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x0b\x00\x00\x00Ciao, world\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let archive = Archive::open(Cursor::new(two.as_slice()), "test3".into()).unwrap();
        assert_eq!(archive.filecount(), 2);
        let mut files = archive.iter().collect::<Vec<FileInfo>>();
        files.sort_by(|i1, i2| i1.name.cmp(&i2.name));
        assert_eq!(files, vec![FileInfo { name: "Ciao, world", addr: 0, len: 0 }, FileInfo { name: "Hello, world!", addr: 46, len: 0 },]);

        let two_split = b"\x15\x11\x19\x99\x02\x00\x00\x00\x00\x00\x00\x00\x00\x0d\x00\x00\x00Hello, world!\x2e\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x3e\x00\x00\x00\x00\x00\x00\x00GARBAGE\x00\x0b\x00\x00\x00Ciao, world\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let archive = Archive::open(Cursor::new(two_split.as_slice()), "test4".into()).unwrap();
        assert_eq!(archive.filecount(), 2);
        let mut files = archive.iter().collect::<Vec<FileInfo>>();
        files.sort_by(|i1, i2| i1.name.cmp(&i2.name));
        assert_eq!(files, vec![FileInfo { name: "Ciao, world", addr: 0, len: 0 }, FileInfo { name: "Hello, world!", addr: 46, len: 0 },]);
    }
}
