use clap::ValueEnum;
use lz4::Decoder;
use std::fmt;
use std::fs::File;
use std::path::Path;

/// Extension i.e. compression of MIDAS files
#[derive(Clone, Copy, Debug, ValueEnum)]
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

/// Decompress `.lz4` file
pub fn decompress_lz4(source: &Path, destination: &Path) -> std::io::Result<()> {
    let input_file = File::open(source)?;
    let mut decoder = Decoder::new(input_file)?;
    let mut output_file = File::create(destination)?;
    std::io::copy(&mut decoder, &mut output_file)?;

    Ok(())
}

#[cfg(test)]
mod tests;
