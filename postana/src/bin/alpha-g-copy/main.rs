//! Wrapper for `scp` to make a local copy of the MIDAS files from specific
//! runs of the ALPHA-g experiment.

use clap::{ArgEnum, Parser};
use glob::glob;
use std::{
    fmt, io,
    io::Write,
    process::{Command, Stdio},
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Run numbers for which you want to copy all MIDAS files locally
    #[clap(required = true)]
    run_numbers: Vec<u32>,
    /// Host from which the files will be copied
    #[clap(arg_enum, short, long)]
    source: Host,
    /// Path where the MIDAS files will be copied into
    #[clap(short, long)]
    output_path: Option<String>,
    /// Extension i.e. compression of remote files
    #[clap(arg_enum, short, long)]
    extension: Option<Extension>,
    /// Decompress the copied MIDAS file (requires --extension)
    #[clap(short, long, requires("extension"))]
    decompress: bool,
    /// User at remote host
    #[clap(short, long)]
    user: Option<String>,
}

/// Run `scp` and (if applicable) the appropriate decompression of the copied
/// files
fn main() {
    let args = Args::parse();

    let user = match args.user {
        Some(value) => value,
        None => get_from_stdin("Insert your username: "),
    };

    // Name of all the files we want to copy
    let file_names: Vec<String> = args
        .run_numbers
        .iter()
        .map(|n| file_name(*n, args.extension))
        .collect();

    // Format the remote path such that we can send a single scp command
    // That way people will be only asked their password once
    let mut remote = user + "@" + &args.source.to_string() + ":" + &midas_data_path(args.source);
    if file_names.len() == 1 {
        remote += &file_names[0];
    } else {
        remote = remote + "{" + &file_names.join(",") + "}";
    }

    let output_path = match args.output_path {
        None => String::from("./"),
        Some(path) => path,
    };

    let status = Command::new("scp")
        .arg(remote)
        .arg(&output_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("failed to run scp");

    if args.decompress && status.success() {
        // Arguments not passed through a shell.
        // Need to glob ourselves
        match args.extension.unwrap() {
            Extension::Lz4 => {
                /*
                Command::new("lz4")
                    .current_dir(output_path)
                    .arg("-d") // Decompress
                    .arg("-m") //Multiple input files
                    .arg("--rm") // Delete input files when done
                    .arg(&file_names.join(" "))
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .expect("failed to run lz4");
                    */
            }
        }
    }
}

/// Host from which the files will be copied
#[derive(Clone, Copy, Debug, ArgEnum)]
enum Host {
    Lxplus,
}
impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Host::Lxplus => write!(f, "lxplus.cern.ch"),
        }
    }
}

/// Compression of files in the remote host
#[derive(Clone, Copy, Debug, ArgEnum)]
enum Extension {
    Lz4,
}
impl fmt::Display for Extension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Extension::Lz4 => write!(f, ".lz4"),
        }
    }
}

/// Add `0`s to the beginning of the run number until it is at least 5
/// characters long.
fn format_run_number(run_number: u32) -> String {
    let mut run_number = run_number.to_string();
    while run_number.len() < 5 {
        run_number = "0".to_owned() + &run_number;
    }
    run_number
}

/// Get ALPHA-g file name given run number and extension.
///
/// All the files from a given run follow the format:
///
/// ```text
/// runXXXXXsub*.mid
/// ```
/// where `XXXXX` is the run number. Files can have a further extension (e.g.
/// `.lz4`) if they are compressed.
fn file_name(run_number: u32, ext: Option<Extension>) -> String {
    let ext = match ext {
        None => String::new(),
        Some(extension) => extension.to_string(),
    };
    String::from("run") + &format_run_number(run_number) + "sub*.mid" + &ext
}

/// Path to MIDAS files in a given host
fn midas_data_path(source: Host) -> String {
    match source {
        Host::Lxplus => String::from("/eos/experiment/ALPHAg/midasdata_old/"),
    }
}

/// Get line from standard input
fn get_from_stdin(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();

    let mut result = String::new();
    io::stdin()
        .read_line(&mut result)
        .expect("failed to read user");

    String::from(result.trim())
}
