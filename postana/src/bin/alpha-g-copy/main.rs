//! Make local copies of the MIDAS files from specific runs of the ALPHA-g
//! experiment.

use crate::host::Host;
use clap::{ArgEnum, Parser};
use glob::glob;
use std::{
    ffi::OsString,
    fmt,
    path::{Path, PathBuf},
    process::Command,
};

/// Hosts for ALPHA-g MIDAS files
mod host;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Run numbers for which you want to copy all MIDAS files locally
    #[clap(required = true)]
    run_numbers: Vec<u32>,
    /// User at remote host
    #[clap(short, long)]
    user: String,
    /// Host from which the files will be copied
    #[clap(arg_enum, short, long)]
    source: Host,
    /// Path where the MIDAS files will be copied into
    #[clap(short, long, default_value="./", parse(try_from_str=is_directory))]
    output_path: PathBuf,
    /// Extension i.e. compression of remote files
    #[clap(arg_enum, short, long)]
    extension: Option<Extension>,
    /// Decompress the copied MIDAS file (requires --extension)
    #[clap(short, long, requires("extension"))]
    decompress: bool,
}

/// Extension i.e. compression of MIDAS files
#[derive(Clone, Copy, Debug, ArgEnum)]
pub enum Extension {
    Lz4,
}
impl fmt::Display for Extension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Extension::Lz4 => write!(f, ".lz4"),
        }
    }
}

/// Copy and (if applicable) run the appropriate decompression of the MIDAS
/// files
fn main() {
    let args = Args::parse();

    // Pattern of all files we want to copy (just filename, no paths)
    let patterns: Vec<String> = args
        .run_numbers
        .into_iter()
        .map(|n| {
            args.source.filename_pattern(n)
                + &args.extension.map_or(String::new(), |e| e.to_string())
        })
        .collect();

    // Format the remote path such that we can send a single scp command
    // That way people will be only asked their password once
    let mut remote = args.user
        + "@"
        + &args.source.to_string()
        + ":"
        + args.source.data_path().to_str().unwrap();
    if patterns.len() == 1 {
        remote += &patterns[0];
    } else {
        remote = remote + "{" + &patterns.join(",") + "}";
    }

    Command::new("scp")
        .arg(remote)
        .arg(&args.output_path)
        .status()
        .expect("failed to run scp");

    // Don't check status. Decompress whatever was successful
    if args.decompress {
        // Arguments not passed through a shell. Need to glob ourselves.
        // This is just the filenames (no path to them).
        // Need to run the decompression from the output_path
        let file_names = local_files(&args.output_path, &patterns);

        match args.extension.unwrap() {
            Extension::Lz4 => {
                Command::new("lz4")
                    .current_dir(args.output_path)
                    .arg("-d") // Decompress
                    .arg("-m") //Multiple input files
                    .arg("--rm") // Delete input files when done
                    .args(file_names)
                    .status()
                    .expect("failed to run lz4");
            }
        }
    }
}

/// Find all files in a path that match a set of patterns
fn local_files(path: &Path, patterns: &[String]) -> Vec<OsString> {
    let patterns: Vec<PathBuf> = patterns.iter().map(|s| path.join(s)).collect();

    let mut local_files = Vec::new();
    for pattern in patterns {
        for entry in glob(pattern.to_str().unwrap()).unwrap() {
            local_files.push(entry.unwrap());
        }
    }

    local_files
        .into_iter()
        .map(|p| p.file_name().unwrap().to_os_string())
        .collect()
}

/// Parse `--output-path` flag as valid directory
fn is_directory(s: &str) -> Result<PathBuf, String> {
    let path: PathBuf = s.into();
    if path.is_dir() {
        Ok(path)
    } else {
        Err(String::from("path is not pointing at a directory on disk"))
    }
}

#[cfg(test)]
mod tests;
