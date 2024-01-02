use std::ffi::{OsStr, OsString};
use std::path::Path;
use thiserror::Error;

// Known ALPHA-g file extensions.
#[derive(Clone, Copy, Debug)]
enum Extension {
    Mid,
    Lz4,
    // If a new extension is added, remember to update the `TryFrom<&OsStr>`
    // implementation.
}

/// The error type returned when conversion from [`OsStr`] to a known ALPHA-g
/// file extension fails.
#[derive(Debug, Error)]
#[error("unknown conversion from `{extension:?}`")]
pub struct TryExtensionFromOsStrError {
    extension: OsString,
}

impl TryFrom<&OsStr> for Extension {
    type Error = TryExtensionFromOsStrError;

    fn try_from(extension: &OsStr) -> Result<Self, Self::Error> {
        match extension.to_str() {
            Some("mid") => Ok(Self::Mid),
            Some("lz4") => Ok(Self::Lz4),
            _ => Err(TryExtensionFromOsStrError {
                extension: extension.to_owned(),
            }),
        }
    }
}

/// The error type for I/O operations on ALPHA-g files
#[derive(Debug, Error)]
pub enum AlphaIOError {
    /// The error type for I/O operations of the Read, Write, Seek, and
    /// associated traits.
    #[error("io error")]
    IoError(#[from] std::io::Error),
    /// Unknown file extension.
    #[error("unknown file extension")]
    UnknownExtension(#[from] TryExtensionFromOsStrError),
}

/// Read the entire contents of a file (auto-detecting compression).
///
/// The compression algorithm is detected based on the file extension. This is a
/// convenience function for using [`std::fs::read`] and handling the known
/// compression algorithms used to store ALPHA-g data.
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, AlphaIOError> {
    let contents = std::fs::read(&path)?;

    match Extension::try_from(path.as_ref().extension().unwrap_or_default())? {
        Extension::Mid => Ok(contents),
        Extension::Lz4 => {
            let mut decoder = lz4::Decoder::new(&contents[..])?;
            let mut contents = Vec::new();
            std::io::copy(&mut decoder, &mut contents)?;
            Ok(contents)
        }
    }
}
