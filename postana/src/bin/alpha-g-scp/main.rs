//! Wrapper for `scp` to make a local copy of the MIDAS files from specific
//! runs of the ALPHA-g experiment.

use clap::{ArgEnum, Parser};

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
}

#[derive(Clone, Copy, Debug, ArgEnum)]
enum Host {
    Lxplus,
    Alphagdaq,
}

#[derive(Clone, Copy, Debug, ArgEnum)]
enum Extension {
    Lz4,
}

fn main() {
    let args = Args::parse();
}
