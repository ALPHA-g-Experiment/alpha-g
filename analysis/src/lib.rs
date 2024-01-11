use midasio::file::{initial_timestamp_unchecked, run_number_unchecked, TryFileViewFromBytesError};
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::path::{Path, PathBuf};
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
    /// MIDAS file format error.
    #[error("midas file format error")]
    MidasFileFormatError(#[from] TryFileViewFromBytesError),
    /// Bad run number.
    #[error("bad run number in `{}` (expected `{expected}`, found `{found}`)", .path.display())]
    BadRunNumber {
        path: PathBuf,
        expected: u32,
        found: u32,
    },
    /// Duplicate files by their initial timestamp.
    #[error("duplicate initial timestamp in `{}` and `{}`", .path1.display(), .path2.display())]
    DuplicateInitialTimestamp { path1: PathBuf, path2: PathBuf },
}

/// Read the entire contents of a file (auto-detecting compression).
///
/// The compression algorithm is detected based on the file extension. This is a
/// convenience function for using [`std::fs::read`] and handling the known
/// compression algorithms used to store ALPHA-g data.
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, AlphaIOError> {
    match Extension::try_from(path.as_ref().extension().unwrap_or_default())? {
        Extension::Mid => Ok(std::fs::read(&path)?),
        Extension::Lz4 => {
            let file = std::fs::File::open(&path)?;
            let mut decoder = lz4::Decoder::new(file)?;
            let mut contents = Vec::new();
            std::io::copy(&mut decoder, &mut contents)?;
            Ok(contents)
        }
    }
}

/// Sort all the files of an individual run by their initial ODB dump timestamp.
///
/// Returns an error if:
/// - Not all files correspond to the same run number.
/// - Two files have the same initial timestamp.
///
/// # Panics
///
/// Panics if the input iterator is empty.
pub fn sort_run_files<P: AsRef<Path>>(
    files: impl IntoIterator<Item = P>,
) -> Result<(u32, Vec<P>), AlphaIOError> {
    let mut files = files
        .into_iter()
        .map(|path| {
            let mut file = std::fs::File::open(&path)?;
            // The first 12 bytes contain both the run number and the initial
            // timestamp.
            let mut buffer = [0; 12];
            match Extension::try_from(path.as_ref().extension().unwrap_or_default())? {
                Extension::Mid => {
                    file.read_exact(&mut buffer)?;
                }
                Extension::Lz4 => {
                    let mut decoder = lz4::Decoder::new(&mut file)?;
                    decoder.read_exact(&mut buffer)?;
                }
            }

            let run_number = run_number_unchecked(&buffer)?;
            let initial_timestamp = initial_timestamp_unchecked(&buffer)?;

            Ok((run_number, initial_timestamp, path))
        })
        .collect::<Result<Vec<_>, AlphaIOError>>()?;

    assert!(!files.is_empty());
    let expected_run_number = files[0].0;
    for (run_number, _, path) in &files {
        if *run_number != expected_run_number {
            return Err(AlphaIOError::BadRunNumber {
                path: path.as_ref().to_owned(),
                expected: expected_run_number,
                found: *run_number,
            });
        }
    }

    files.sort_unstable_by_key(|(_, initial_timestamp, _)| *initial_timestamp);
    // These are sorted, so it is enough to check for consecutive duplicates.
    for window in files.windows(2) {
        if window[0].1 == window[1].1 {
            return Err(AlphaIOError::DuplicateInitialTimestamp {
                path1: window[0].2.as_ref().to_owned(),
                path2: window[1].2.as_ref().to_owned(),
            });
        }
    }

    Ok((
        expected_run_number,
        files.into_iter().map(|(_, _, path)| path).collect(),
    ))
}
